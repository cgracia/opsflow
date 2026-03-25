/// Import ccusage `--json` output as a data source.
///
/// Instead of reimplementing Claude Code or OpenCode log parsing, users can
/// pipe ccusage output into Praxis:
///
///   ccusage session --json > ~/.praxis/imports/ccusage-claude-code.json
///   praxis ingest --source ccusage
///
/// Files are read from `{praxis_dir}/imports/ccusage-*.json`.
use crate::db;
use crate::ingest::{IngestResult, RunRecord, SessionRecord};
use duckdb::Connection;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct CcusageReport {
    #[serde(rename = "type")]
    report_type: Option<String>,
    data: Vec<CcusageSession>,
}

#[derive(Debug, Deserialize)]
struct CcusageSession {
    session: String,
    #[serde(default)]
    models: Vec<String>,
    #[serde(rename = "inputTokens", default)]
    input_tokens: i64,
    #[serde(rename = "outputTokens", default)]
    output_tokens: i64,
    #[serde(rename = "cacheCreationTokens", default)]
    cache_creation_tokens: i64,
    #[serde(rename = "cacheReadTokens", default)]
    cache_read_tokens: i64,
    #[serde(rename = "costUSD", default)]
    cost_usd: f64,
    #[serde(rename = "lastActivity", default)]
    last_activity: Option<String>,
}

pub fn ingest(
    conn: &Connection,
    imports_dir: &Path,
) -> Result<IngestResult, Box<dyn std::error::Error>> {
    if !imports_dir.exists() {
        return Ok(IngestResult {
            source: "ccusage".to_string(),
            sessions_added: 0,
            runs_added: 0,
            files_skipped: 0,
        });
    }

    let mut sessions_added = 0;
    let mut runs_added = 0;
    let mut files_skipped = 0;

    let pattern = format!("{}/ccusage-*.json", imports_dir.display());
    for entry in glob::glob(&pattern)? {
        let path = match entry {
            Ok(p) => p,
            Err(e) => {
                eprintln!("warning: glob error: {}", e);
                continue;
            }
        };

        let path_str = path.to_string_lossy().to_string();
        let mtime = file_mtime_str(&path);

        if let Ok(Some(prev)) = db::check_ingested(conn, "ccusage", &path_str) {
            if prev == mtime {
                files_skipped += 1;
                continue;
            }
        }

        match parse_ccusage_file(&path) {
            Ok((sessions, runs)) => {
                let count = runs.len() as i32;
                for s in &sessions {
                    db::insert_session(conn, s)?;
                }
                for r in &runs {
                    db::insert_run(conn, r)?;
                }
                db::record_ingestion(conn, "ccusage", &path_str, &mtime, count)?;
                sessions_added += sessions.len();
                runs_added += runs.len();

                // Move processed file to imports/processed/
                let processed_dir = path.parent().unwrap().join("processed");
                let _ = std::fs::create_dir_all(&processed_dir);
                let dest = processed_dir.join(path.file_name().unwrap());
                let _ = std::fs::rename(&path, dest);
            }
            Err(e) => {
                eprintln!(
                    "warning: failed to parse ccusage file {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }

    Ok(IngestResult {
        source: "ccusage".to_string(),
        sessions_added,
        runs_added,
        files_skipped,
    })
}

fn parse_ccusage_file(
    path: &Path,
) -> Result<(Vec<SessionRecord>, Vec<RunRecord>), Box<dyn std::error::Error>> {
    let raw = std::fs::read_to_string(path)?;
    let report: CcusageReport = serde_json::from_str(&raw)?;

    // Infer source from filename:
    //   ccusage-claude-code.json → claude-code
    //   ccusage-opencode.json    → opencode
    //   otherwise                → ccusage
    let filename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("ccusage");
    let source = if filename.contains("opencode") {
        "opencode"
    } else if filename.contains("claude") {
        "claude-code"
    } else {
        "ccusage"
    };

    let mut sessions: Vec<SessionRecord> = Vec::new();
    let mut runs: Vec<RunRecord> = Vec::new();

    for entry in &report.data {
        let primary_model = entry.models.first().cloned();
        let timestamp = entry
            .last_activity
            .as_deref()
            .map(|d| format!("{}T00:00:00Z", d))
            .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());

        let session_id = format!("ccusage-{}", entry.session);

        sessions.push(SessionRecord {
            id: session_id.clone(),
            source: source.to_string(),
            project: None,
            title: None,
            model: primary_model.clone(),
            started_at: Some(timestamp.clone()),
            ended_at: Some(timestamp.clone()),
            git_branch: None,
            working_dir: None,
        });

        // ccusage data for Claude Code inherits unreliable token counts
        let is_claude_code = source == "claude-code";
        let data_quality = if is_claude_code {
            "claude_code_jsonl_unreliable"
        } else {
            "reliable"
        };

        runs.push(RunRecord {
            id: session_id.clone(),
            session_id: Some(session_id),
            source: source.to_string(),
            model: primary_model,
            timestamp,
            role: None,
            input_tokens: entry.input_tokens,
            output_tokens: entry.output_tokens,
            cache_read_tokens: entry.cache_read_tokens,
            cache_write_tokens: entry.cache_creation_tokens,
            tokens_estimated: is_claude_code,
            data_quality: data_quality.to_string(),
            cost_usd: entry.cost_usd,
            cost_estimated: false, // ccusage calculated this with LiteLLM pricing
            duration_ms: None,
            project: None,
            git_branch: None,
            working_dir: None,
            command: None,
            source_path: Some(path.to_string_lossy().to_string()),
        });
    }

    Ok((sessions, runs))
}

fn file_mtime_str(path: &Path) -> String {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string()
        })
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;
    use tempfile::TempDir;

    fn sample_ccusage_json(filename_hint: &str) -> (&'static str, String) {
        let json = r#"{
            "type": "session",
            "data": [
                {
                    "session": "sess-001",
                    "models": ["claude-opus-4-20250514"],
                    "inputTokens": 4512,
                    "outputTokens": 350846,
                    "cacheCreationTokens": 512,
                    "cacheReadTokens": 1024,
                    "totalTokens": 356894,
                    "costUSD": 156.40,
                    "lastActivity": "2025-05-24"
                }
            ],
            "summary": {}
        }"#;
        (json, filename_hint.to_string())
    }

    #[test]
    fn test_parse_ccusage_claude_code() {
        let dir = TempDir::new().unwrap();
        let (json, _) = sample_ccusage_json("ccusage-claude-code");
        let path = dir.path().join("ccusage-claude-code.json");
        std::fs::write(&path, json).unwrap();

        let (sessions, runs) = parse_ccusage_file(&path).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(runs.len(), 1);

        let run = &runs[0];
        assert_eq!(run.source, "claude-code");
        assert_eq!(run.input_tokens, 4512);
        assert_eq!(run.output_tokens, 350846);
        assert_eq!(run.cost_usd, 156.40);
        assert!(!run.cost_estimated); // ccusage calculated cost
        assert!(run.tokens_estimated); // but token counts are unreliable (Claude Code)
        assert_eq!(run.data_quality, "claude_code_jsonl_unreliable");
    }

    #[test]
    fn test_parse_ccusage_opencode() {
        let dir = TempDir::new().unwrap();
        let json = r#"{"type":"session","data":[{"session":"oc-sess","models":["anthropic/claude-sonnet-4"],"inputTokens":1000,"outputTokens":500,"cacheCreationTokens":0,"cacheReadTokens":0,"costUSD":0.05,"lastActivity":"2026-03-25"}],"summary":{}}"#;
        let path = dir.path().join("ccusage-opencode.json");
        std::fs::write(&path, json).unwrap();

        let (_, runs) = parse_ccusage_file(&path).unwrap();
        let run = &runs[0];
        assert_eq!(run.source, "opencode");
        assert!(!run.tokens_estimated); // OpenCode counts are reliable
        assert_eq!(run.data_quality, "reliable");
    }

    #[test]
    fn test_ingest_moves_to_processed() {
        let dir = TempDir::new().unwrap();
        let imports_dir = dir.path().join("imports");
        std::fs::create_dir_all(&imports_dir).unwrap();

        let (json, _) = sample_ccusage_json("ccusage-claude-code");
        let src = imports_dir.join("ccusage-claude-code.json");
        std::fs::write(&src, json).unwrap();

        let praxis_dir = dir.path().join("praxis");
        let conn = init_db(&praxis_dir).unwrap();

        let result = ingest(&conn, &imports_dir).unwrap();
        assert_eq!(result.runs_added, 1);

        // File should be moved to processed/
        assert!(!src.exists());
        assert!(imports_dir
            .join("processed/ccusage-claude-code.json")
            .exists());
    }
}
