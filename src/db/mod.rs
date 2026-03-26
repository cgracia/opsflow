use duckdb::{params, Connection};
use std::path::Path;

pub use crate::ingest::{RunRecord, SessionRecord};

/// Open (or create) the praxis DuckDB database and apply schema migrations.
pub fn init_db(praxis_dir: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(praxis_dir)?;
    let db_path = praxis_dir.join("praxis.db");
    let conn = Connection::open(&db_path)?;
    apply_schema(&conn)?;
    Ok(conn)
}

fn apply_schema(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS sessions (
            id          VARCHAR PRIMARY KEY,
            source      VARCHAR NOT NULL,
            project     VARCHAR,
            title       VARCHAR,
            model       VARCHAR,
            started_at  VARCHAR,
            ended_at    VARCHAR,
            git_branch  VARCHAR,
            working_dir VARCHAR,
            ingested_at VARCHAR NOT NULL
        );

        CREATE TABLE IF NOT EXISTS runs (
            id                  VARCHAR PRIMARY KEY,
            session_id          VARCHAR,
            source              VARCHAR NOT NULL,
            model               VARCHAR,
            timestamp           VARCHAR NOT NULL,
            role                VARCHAR,
            input_tokens        BIGINT DEFAULT 0,
            output_tokens       BIGINT DEFAULT 0,
            cache_read_tokens   BIGINT DEFAULT 0,
            cache_write_tokens  BIGINT DEFAULT 0,
            tokens_estimated    BOOLEAN DEFAULT false,
            data_quality        VARCHAR DEFAULT 'reliable',
            cost_usd            DOUBLE DEFAULT 0.0,
            cost_estimated      BOOLEAN DEFAULT true,
            duration_ms         BIGINT,
            project             VARCHAR,
            git_branch          VARCHAR,
            working_dir         VARCHAR,
            command             VARCHAR,
            source_path         VARCHAR,
            ingested_at         VARCHAR NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ingestion_log (
            source          VARCHAR NOT NULL,
            source_path     VARCHAR NOT NULL,
            file_modified   VARCHAR NOT NULL,
            ingested_at     VARCHAR NOT NULL,
            records_added   INTEGER DEFAULT 0,
            PRIMARY KEY (source, source_path)
        );

        -- Phase 3: Control Plane tables

        -- Tracks last output time per workflow (updated by signal collector).
        CREATE TABLE IF NOT EXISTS workflow_runs (
            workflow_name   VARCHAR PRIMARY KEY,
            last_output_at  VARCHAR,
            output_path     VARCHAR
        );

        -- Signals: new/modified files detected in workflow output directories.
        CREATE TABLE IF NOT EXISTS signals (
            id               VARCHAR PRIMARY KEY,
            workflow_name    VARCHAR NOT NULL,
            file_path        VARCHAR NOT NULL,
            detected_at      VARCHAR NOT NULL,
            file_modified    VARCHAR NOT NULL,
            content_preview  VARCHAR,
            priority         VARCHAR,
            triage_reason    VARCHAR,
            suggested_action VARCHAR,
            triaged_at       VARCHAR,
            acknowledged     BOOLEAN DEFAULT false,
            acknowledged_at  VARCHAR
        );

        -- Tracks the last scan time per output directory (for incremental collection).
        CREATE TABLE IF NOT EXISTS collection_state (
            directory        VARCHAR PRIMARY KEY,
            last_scanned     VARCHAR NOT NULL,
            last_file_mtime  VARCHAR
        );

        -- Observability: one row per triage LLM call.
        CREATE TABLE IF NOT EXISTS triage_runs (
            id                 VARCHAR PRIMARY KEY,
            timestamp          VARCHAR NOT NULL,
            model              VARCHAR,
            signals_triaged    INTEGER,
            tokens_input       BIGINT,
            tokens_output      BIGINT,
            cache_read_tokens  BIGINT,
            cache_write_tokens BIGINT,
            cost_usd           DOUBLE,
            duration_ms        BIGINT
        );
    ",
    )?;

    // Migrations: add columns that may not exist in older databases.
    // DuckDB supports ALTER TABLE ... ADD COLUMN IF NOT EXISTS.
    conn.execute_batch(
        "ALTER TABLE triage_runs ADD COLUMN IF NOT EXISTS cache_read_tokens  BIGINT;
         ALTER TABLE triage_runs ADD COLUMN IF NOT EXISTS cache_write_tokens BIGINT;",
    )?;

    Ok(())
}

/// Returns the stored file_modified timestamp if this path has already been ingested.
pub fn check_ingested(
    conn: &Connection,
    source: &str,
    source_path: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut stmt = conn
        .prepare("SELECT file_modified FROM ingestion_log WHERE source = ? AND source_path = ?")?;
    let mut rows = stmt.query(params![source, source_path])?;
    if let Some(row) = rows.next()? {
        let val: String = row.get(0)?;
        Ok(Some(val))
    } else {
        Ok(None)
    }
}

/// Record a completed ingestion in the log (upsert by source + source_path).
pub fn record_ingestion(
    conn: &Connection,
    source: &str,
    source_path: &str,
    file_modified: &str,
    records_added: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO ingestion_log (source, source_path, file_modified, ingested_at, records_added)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT (source, source_path) DO UPDATE SET
             file_modified = excluded.file_modified,
             ingested_at   = excluded.ingested_at,
             records_added = excluded.records_added",
        params![source, source_path, file_modified, now, records_added],
    )?;
    Ok(())
}

/// Insert a session record. Skips on conflict (idempotent).
pub fn insert_session(
    conn: &Connection,
    s: &SessionRecord,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO sessions
             (id, source, project, title, model, started_at, ended_at, git_branch, working_dir, ingested_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT (id) DO NOTHING",
        params![
            s.id,
            s.source,
            s.project,
            s.title,
            s.model,
            s.started_at,
            s.ended_at,
            s.git_branch,
            s.working_dir,
            now
        ],
    )?;
    Ok(())
}

/// Insert a run record. Upserts on conflict so re-ingestion can refresh mutable logs.
pub fn insert_run(conn: &Connection, r: &RunRecord) -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO runs
             (id, session_id, source, model, timestamp, role,
              input_tokens, output_tokens, cache_read_tokens, cache_write_tokens,
              tokens_estimated, data_quality, cost_usd, cost_estimated,
              duration_ms, project, git_branch, working_dir, command, source_path, ingested_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT (id) DO UPDATE SET
             session_id         = excluded.session_id,
             source             = excluded.source,
             model              = excluded.model,
             timestamp          = excluded.timestamp,
             role               = excluded.role,
             input_tokens       = excluded.input_tokens,
             output_tokens      = excluded.output_tokens,
             cache_read_tokens  = excluded.cache_read_tokens,
             cache_write_tokens = excluded.cache_write_tokens,
             tokens_estimated   = excluded.tokens_estimated,
             data_quality       = excluded.data_quality,
             cost_usd           = excluded.cost_usd,
             cost_estimated     = excluded.cost_estimated,
             duration_ms        = excluded.duration_ms,
             project            = excluded.project,
             git_branch         = excluded.git_branch,
             working_dir        = excluded.working_dir,
             command            = excluded.command,
             source_path        = excluded.source_path,
             ingested_at        = excluded.ingested_at",
        params![
            r.id,
            r.session_id,
            r.source,
            r.model,
            r.timestamp,
            r.role,
            r.input_tokens,
            r.output_tokens,
            r.cache_read_tokens,
            r.cache_write_tokens,
            r.tokens_estimated,
            r.data_quality,
            r.cost_usd,
            r.cost_estimated,
            r.duration_ms,
            r.project,
            r.git_branch,
            r.working_dir,
            r.command,
            r.source_path,
            now
        ],
    )?;
    Ok(())
}

/// Get the total number of runs in the database.
pub fn run_count(conn: &Connection) -> Result<i64, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM runs")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_conn() -> (TempDir, Connection) {
        let dir = TempDir::new().unwrap();
        let conn = init_db(dir.path()).unwrap();
        (dir, conn)
    }

    #[test]
    fn test_schema_creation() {
        let (_dir, conn) = temp_conn();
        // Should be able to query all three tables
        conn.execute_batch("SELECT 1 FROM sessions LIMIT 0")
            .unwrap();
        conn.execute_batch("SELECT 1 FROM runs LIMIT 0").unwrap();
        conn.execute_batch("SELECT 1 FROM ingestion_log LIMIT 0")
            .unwrap();
    }

    #[test]
    fn test_ingestion_log_round_trip() {
        let (_dir, conn) = temp_conn();

        assert!(check_ingested(&conn, "opencode", "/some/path.json")
            .unwrap()
            .is_none());

        record_ingestion(
            &conn,
            "opencode",
            "/some/path.json",
            "2026-03-25T10:00:00Z",
            5,
        )
        .unwrap();

        let result = check_ingested(&conn, "opencode", "/some/path.json").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "2026-03-25T10:00:00Z");
    }

    #[test]
    fn test_ingestion_log_upsert() {
        let (_dir, conn) = temp_conn();

        record_ingestion(&conn, "opencode", "/path.json", "2026-03-24T10:00:00Z", 3).unwrap();
        record_ingestion(&conn, "opencode", "/path.json", "2026-03-25T10:00:00Z", 7).unwrap();

        let result = check_ingested(&conn, "opencode", "/path.json").unwrap();
        assert_eq!(result.unwrap(), "2026-03-25T10:00:00Z");
    }

    #[test]
    fn test_insert_session() {
        let (_dir, conn) = temp_conn();

        let session = SessionRecord {
            id: "sess-001".to_string(),
            source: "opencode".to_string(),
            project: Some("myapp".to_string()),
            title: Some("Refactor auth".to_string()),
            model: Some("claude-sonnet-4".to_string()),
            started_at: Some("2026-03-25T09:00:00Z".to_string()),
            ended_at: Some("2026-03-25T09:30:00Z".to_string()),
            git_branch: Some("main".to_string()),
            working_dir: Some("/home/user/myapp".to_string()),
        };
        insert_session(&conn, &session).unwrap();

        // Duplicate should be silently skipped
        insert_session(&conn, &session).unwrap();

        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM sessions WHERE id = 'sess-001'")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        let count: i64 = rows.next().unwrap().unwrap().get(0).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_run() {
        let (_dir, conn) = temp_conn();

        let run = RunRecord {
            id: "run-001".to_string(),
            session_id: Some("sess-001".to_string()),
            source: "opencode".to_string(),
            model: Some("claude-sonnet-4".to_string()),
            timestamp: "2026-03-25T09:15:00Z".to_string(),
            role: Some("assistant".to_string()),
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 200,
            cache_write_tokens: 100,
            tokens_estimated: false,
            data_quality: "reliable".to_string(),
            cost_usd: 0.025,
            cost_estimated: false,
            duration_ms: None,
            project: Some("myapp".to_string()),
            git_branch: Some("main".to_string()),
            working_dir: Some("/home/user/myapp".to_string()),
            command: None,
            source_path: Some("/data/opencode/message/sess-001/msg-001.json".to_string()),
        };
        insert_run(&conn, &run).unwrap();

        assert_eq!(run_count(&conn).unwrap(), 1);

        // Duplicate is upserted
        let mut updated_run = run.clone();
        updated_run.output_tokens = 750;
        updated_run.project = Some("myapp-v2".to_string());
        insert_run(&conn, &updated_run).unwrap();
        assert_eq!(run_count(&conn).unwrap(), 1);

        let mut stmt = conn
            .prepare("SELECT output_tokens, project FROM runs WHERE id = 'run-001'")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        let row = rows.next().unwrap().unwrap();
        let output_tokens: i64 = row.get(0).unwrap();
        let project: Option<String> = row.get(1).unwrap();
        assert_eq!(output_tokens, 750);
        assert_eq!(project.as_deref(), Some("myapp-v2"));
    }
}
