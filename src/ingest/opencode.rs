/// Ingest OpenCode session and message logs.
///
/// OpenCode stores data under:
///   {opencode_dir}/storage/session/{projectHash}/{sessionID}.json
///   {opencode_dir}/storage/message/{sessionID}/msg_{messageID}.json
use crate::db;
use crate::ingest::pricing::PricingDb;
use crate::ingest::{IngestResult, RunRecord, SessionRecord};
use duckdb::Connection;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct OcSession {
    id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(rename = "projectID", default)]
    project_id: Option<String>,
    /// "provider/model" string
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    time: Option<OcTime>,
}

#[derive(Debug, Deserialize)]
struct OcTime {
    #[serde(default)]
    created: Option<i64>,
    #[serde(default)]
    updated: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct OcMessage {
    id: String,
    #[serde(rename = "sessionID")]
    session_id: String,
    #[serde(default)]
    role: Option<String>,
    #[serde(default, deserialize_with = "deserialize_model")]
    model: Option<String>,
    #[serde(default)]
    input_tokens: i64,
    #[serde(default)]
    output_tokens: i64,
    #[serde(default)]
    cache_creation_input_tokens: i64,
    #[serde(default)]
    cache_read_input_tokens: i64,
    /// Always 0 in OpenCode — we calculate cost from tokens.
    #[serde(rename = "cost", default)]
    _cost: f64,
    #[serde(default, alias = "time", deserialize_with = "deserialize_time_created")]
    time_created: Option<i64>,
}

pub fn ingest(
    conn: &Connection,
    pricing: &PricingDb,
    opencode_dir: &Path,
) -> Result<IngestResult, Box<dyn std::error::Error>> {
    let storage = opencode_dir.join("storage");
    let session_dir = storage.join("session");
    let message_dir = storage.join("message");

    let mut sessions_added = 0;
    let mut runs_added = 0;
    let mut files_skipped = 0;
    let mut session_projects: HashMap<String, Option<String>> = HashMap::new();

    // Ingest session files
    if session_dir.exists() {
        let pattern = format!("{}/**/*.json", session_dir.display());
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

            if let Ok(Some(prev)) = db::check_ingested(conn, "opencode_session", &path_str) {
                if prev == mtime {
                    files_skipped += 1;
                    continue;
                }
            }

            match parse_session_file(&path) {
                Ok(session) => {
                    session_projects.insert(session.id.clone(), session.project.clone());
                    db::insert_session(conn, &session)?;
                    db::record_ingestion(conn, "opencode_session", &path_str, &mtime, 1)?;
                    sessions_added += 1;
                }
                Err(e) => {
                    eprintln!("warning: failed to parse session {}: {}", path.display(), e);
                }
            }
        }
    }

    // Ingest message files
    if message_dir.exists() {
        let pattern = format!("{}/**/*.json", message_dir.display());
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

            if let Ok(Some(prev)) = db::check_ingested(conn, "opencode", &path_str) {
                if prev == mtime {
                    files_skipped += 1;
                    continue;
                }
            }

            match parse_message_file(&path, pricing) {
                Ok(mut run) => {
                    if let Some(session_id) = &run.session_id {
                        run.project = session_projects
                            .get(session_id)
                            .cloned()
                            .flatten()
                            .or_else(|| lookup_session_project(conn, session_id));
                    }
                    db::insert_run(conn, &run)?;
                    db::record_ingestion(conn, "opencode", &path_str, &mtime, 1)?;
                    runs_added += 1;
                }
                Err(e) => {
                    eprintln!("warning: failed to parse message {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(IngestResult {
        source: "opencode".to_string(),
        sessions_added,
        runs_added,
        files_skipped,
    })
}

fn parse_session_file(path: &Path) -> Result<SessionRecord, Box<dyn std::error::Error>> {
    let raw = std::fs::read_to_string(path)?;
    let s: OcSession = serde_json::from_str(&raw)?;

    let started_at = s
        .time
        .as_ref()
        .and_then(|t| t.created)
        .map(unix_secs_to_iso);
    let ended_at = s
        .time
        .as_ref()
        .and_then(|t| t.updated)
        .map(unix_secs_to_iso);

    Ok(SessionRecord {
        id: s.id,
        source: "opencode".to_string(),
        project: s.project_id,
        title: s.title,
        model: s.model,
        started_at,
        ended_at,
        git_branch: None,
        working_dir: None,
    })
}

fn parse_message_file(
    path: &Path,
    pricing: &PricingDb,
) -> Result<RunRecord, Box<dyn std::error::Error>> {
    let raw = std::fs::read_to_string(path)?;
    let msg: OcMessage = serde_json::from_str(&raw)?;

    let timestamp = msg
        .time_created
        .map(unix_secs_to_iso)
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());

    let model = msg.model.as_deref().unwrap_or("unknown");
    let (cost_usd, cost_estimated) = pricing.estimate_cost(
        model,
        msg.input_tokens,
        msg.output_tokens,
        msg.cache_read_input_tokens,
        msg.cache_creation_input_tokens,
    );

    Ok(RunRecord {
        id: format!("oc-{}", msg.id),
        session_id: Some(msg.session_id),
        source: "opencode".to_string(),
        model: msg.model,
        timestamp,
        role: msg.role,
        input_tokens: msg.input_tokens,
        output_tokens: msg.output_tokens,
        cache_read_tokens: msg.cache_read_input_tokens,
        cache_write_tokens: msg.cache_creation_input_tokens,
        tokens_estimated: false, // OpenCode token counts are reliable
        data_quality: "reliable".to_string(),
        cost_usd,
        cost_estimated,
        duration_ms: None,
        project: None,
        git_branch: None,
        working_dir: None,
        command: None,
        source_path: Some(path.to_string_lossy().to_string()),
    })
}

fn deserialize_model<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(None);
    };

    match value {
        Value::String(s) => Ok(Some(s)),
        Value::Object(map) => {
            let provider = map
                .get("providerID")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty());
            let model = map
                .get("modelID")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty());

            Ok(match (provider, model) {
                (Some(provider), Some(model)) => Some(format!("{provider}/{model}")),
                (_, Some(model)) => Some(model.to_string()),
                _ => None,
            })
        }
        _ => Ok(None),
    }
}

fn deserialize_time_created<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(extract_created_timestamp(value.as_ref()))
}

fn extract_created_timestamp(value: Option<&Value>) -> Option<i64> {
    let raw = match value? {
        Value::Number(n) => n.as_i64(),
        Value::Object(map) => map.get("created").and_then(Value::as_i64),
        _ => None,
    }?;

    // Newer OpenCode payloads store timestamps in milliseconds.
    if raw > 10_000_000_000 {
        Some(raw / 1000)
    } else {
        Some(raw)
    }
}

fn unix_secs_to_iso(secs: i64) -> String {
    use chrono::{DateTime, Utc};
    DateTime::<Utc>::from(std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs as u64))
        .to_rfc3339()
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

fn lookup_session_project(conn: &Connection, session_id: &str) -> Option<String> {
    let mut stmt = conn
        .prepare("SELECT project FROM sessions WHERE id = ? LIMIT 1")
        .ok()?;
    let mut rows = stmt.query([session_id]).ok()?;
    rows.next().ok().flatten().and_then(|row| row.get(0).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;
    use crate::ingest::pricing::PricingDb;
    use tempfile::TempDir;

    fn empty_pricing() -> PricingDb {
        PricingDb::empty()
    }

    #[test]
    fn test_parse_session_json() {
        let dir = TempDir::new().unwrap();
        let json = r#"{
            "id": "sess-abc",
            "title": "Refactor auth service",
            "projectID": "proj-hash-123",
            "model": "anthropic/claude-sonnet-4-20250514",
            "time": {"created": 1742896800, "updated": 1742900400}
        }"#;
        let path = dir.path().join("sess-abc.json");
        std::fs::write(&path, json).unwrap();

        let session = parse_session_file(&path).unwrap();
        assert_eq!(session.id, "sess-abc");
        assert_eq!(session.source, "opencode");
        assert_eq!(session.title.unwrap(), "Refactor auth service");
        assert_eq!(session.project.unwrap(), "proj-hash-123");
        assert!(session.started_at.is_some());
        assert!(session.ended_at.is_some());
    }

    #[test]
    fn test_parse_message_json() {
        let dir = TempDir::new().unwrap();
        let json = r#"{
            "id": "msg-001",
            "sessionID": "sess-abc",
            "role": "assistant",
            "model": "anthropic/claude-sonnet-4-20250514",
            "input_tokens": 1500,
            "output_tokens": 300,
            "cache_creation_input_tokens": 100,
            "cache_read_input_tokens": 800,
            "cost": 99.0,
            "time_created": 1742896900
        }"#;
        let path = dir.path().join("msg_msg-001.json");
        std::fs::write(&path, json).unwrap();

        let pricing = empty_pricing();
        let run = parse_message_file(&path, &pricing).unwrap();

        assert_eq!(run.id, "oc-msg-001");
        assert_eq!(run.session_id.unwrap(), "sess-abc");
        assert_eq!(run.source, "opencode");
        assert_eq!(run.input_tokens, 1500);
        assert_eq!(run.output_tokens, 300);
        assert_eq!(run.cache_write_tokens, 100);
        assert_eq!(run.cache_read_tokens, 800);
        assert!(!run.tokens_estimated);
        assert_eq!(run.data_quality, "reliable");
        assert_eq!(run.cost_usd, 0.0);
        assert!(run.cost_estimated);
        assert_eq!(run.role.unwrap(), "assistant");
    }

    #[test]
    fn test_parse_message_json_with_structured_model_and_time() {
        let dir = TempDir::new().unwrap();
        let json = r#"{
            "id": "msg-002",
            "sessionID": "sess-abc",
            "role": "user",
            "model": {
                "providerID": "er",
                "modelID": "claude-4-6-opus"
            },
            "time": {
                "created": 1771839318125
            }
        }"#;
        let path = dir.path().join("msg_msg-002.json");
        std::fs::write(&path, json).unwrap();

        let pricing = empty_pricing();
        let run = parse_message_file(&path, &pricing).unwrap();

        assert_eq!(run.id, "oc-msg-002");
        assert_eq!(run.session_id.unwrap(), "sess-abc");
        assert_eq!(run.model.as_deref(), Some("er/claude-4-6-opus"));
        assert_eq!(run.role.as_deref(), Some("user"));
        assert_eq!(run.timestamp, "2026-02-23T09:35:18+00:00");
    }

    #[test]
    fn test_ingest_full_flow() {
        let dir = TempDir::new().unwrap();
        let opencode_dir = dir.path().join("opencode");
        let storage = opencode_dir.join("storage");
        let session_dir = storage.join("session").join("proj-hash");
        let message_dir = storage.join("message").join("sess-abc");
        std::fs::create_dir_all(&session_dir).unwrap();
        std::fs::create_dir_all(&message_dir).unwrap();

        let session_json = r#"{"id":"sess-abc","title":"Test","projectID":"proj-hash","model":"anthropic/claude-sonnet-4","time":{"created":1742896800,"updated":1742900400}}"#;
        std::fs::write(session_dir.join("sess-abc.json"), session_json).unwrap();

        let msg_json = r#"{"id":"msg-001","sessionID":"sess-abc","role":"assistant","model":"anthropic/claude-sonnet-4","input_tokens":100,"output_tokens":50,"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"cost":42.5,"time_created":1742896900}"#;
        std::fs::write(message_dir.join("msg_msg-001.json"), msg_json).unwrap();

        let praxis_dir = dir.path().join("praxis");
        let conn = init_db(&praxis_dir).unwrap();
        let pricing = empty_pricing();

        let result = ingest(&conn, &pricing, &opencode_dir).unwrap();
        assert_eq!(result.source, "opencode");
        assert_eq!(result.sessions_added, 1);
        assert_eq!(result.runs_added, 1);

        // Second ingest: skip already-ingested files
        let result2 = ingest(&conn, &pricing, &opencode_dir).unwrap();
        assert_eq!(result2.runs_added, 0);
        assert_eq!(result2.files_skipped, 2); // session + message

        let mut stmt = conn
            .prepare("SELECT cost_usd, cost_estimated FROM runs WHERE id = 'oc-msg-001'")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        let row = rows.next().unwrap().unwrap();
        let cost_usd: f64 = row.get(0).unwrap();
        let cost_estimated: bool = row.get(1).unwrap();
        assert_eq!(cost_usd, 0.0);
        assert!(cost_estimated);
    }

    #[test]
    fn test_ingest_sets_run_project_from_session() {
        let dir = TempDir::new().unwrap();
        let opencode_dir = dir.path().join("opencode");
        let storage = opencode_dir.join("storage");
        let session_dir = storage.join("session").join("proj-hash");
        let message_dir = storage.join("message").join("sess-abc");
        std::fs::create_dir_all(&session_dir).unwrap();
        std::fs::create_dir_all(&message_dir).unwrap();

        let session_json = r#"{"id":"sess-abc","title":"Test","projectID":"proj-hash","model":"anthropic/claude-sonnet-4","time":{"created":1742896800,"updated":1742900400}}"#;
        std::fs::write(session_dir.join("sess-abc.json"), session_json).unwrap();

        let msg_json = r#"{"id":"msg-001","sessionID":"sess-abc","role":"assistant","model":"anthropic/claude-sonnet-4","input_tokens":100,"output_tokens":50,"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"cost":0,"time_created":1742896900}"#;
        std::fs::write(message_dir.join("msg_msg-001.json"), msg_json).unwrap();

        let praxis_dir = dir.path().join("praxis");
        let conn = init_db(&praxis_dir).unwrap();
        let pricing = empty_pricing();

        ingest(&conn, &pricing, &opencode_dir).unwrap();

        let mut stmt = conn
            .prepare("SELECT project FROM runs WHERE id = 'oc-msg-001'")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        let project: Option<String> = rows.next().unwrap().unwrap().get(0).unwrap();
        assert_eq!(project.as_deref(), Some("proj-hash"));
    }
}
