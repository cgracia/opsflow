/// Ingest Claude Code JSONL session files.
///
/// IMPORTANT: Claude Code JSONL token data is fundamentally unreliable due to
/// a known Anthropic bug (anthropics/claude-code#22686). Input tokens are
/// 100-174x too low (streaming placeholders never updated), and output tokens
/// exclude thinking tokens. Cache fields are accurate.
///
/// All token/cost data from this source is marked as unreliable.
use crate::db;
use crate::ingest::pricing::PricingDb;
use crate::ingest::{IngestResult, RunRecord, SessionRecord};
use duckdb::Connection;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

const DATA_QUALITY: &str = "claude_code_jsonl_unreliable";

#[derive(Debug, Deserialize)]
struct JsonlEntry {
    #[serde(default)]
    uuid: Option<String>,
    #[serde(rename = "sessionId", default)]
    session_id: Option<String>,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(rename = "gitBranch", default)]
    git_branch: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    message: Option<JsonlMessage>,
}

#[derive(Debug, Deserialize)]
struct JsonlMessage {
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    usage: Option<JsonlUsage>,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct JsonlUsage {
    #[serde(default)]
    input_tokens: i64,
    #[serde(default)]
    output_tokens: i64,
    #[serde(default)]
    cache_creation_input_tokens: i64,
    #[serde(default)]
    cache_read_input_tokens: i64,
}

pub fn ingest(
    conn: &Connection,
    pricing: &PricingDb,
    claude_code_dir: &Path,
) -> Result<IngestResult, Box<dyn std::error::Error>> {
    if !claude_code_dir.exists() {
        return Ok(IngestResult {
            source: "claude-code".to_string(),
            sessions_added: 0,
            runs_added: 0,
            files_skipped: 0,
        });
    }

    let mut sessions_added = 0;
    let mut runs_added = 0;
    let mut files_skipped = 0;

    let pattern = format!("{}/**/*.jsonl", claude_code_dir.display());
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

        if let Ok(Some(prev)) = db::check_ingested(conn, "claude-code", &path_str) {
            if prev == mtime {
                files_skipped += 1;
                continue;
            }
        }

        match parse_jsonl_file(&path, pricing) {
            Ok((session, runs)) => {
                db::insert_session(conn, &session)?;
                sessions_added += 1;
                let count = runs.len() as i32;
                for run in &runs {
                    db::insert_run(conn, run)?;
                }
                db::record_ingestion(conn, "claude-code", &path_str, &mtime, count)?;
                runs_added += runs.len();
            }
            Err(e) => {
                eprintln!("warning: failed to parse {}: {}", path.display(), e);
            }
        }
    }

    Ok(IngestResult {
        source: "claude-code".to_string(),
        sessions_added,
        runs_added,
        files_skipped,
    })
}

fn parse_jsonl_file(
    path: &Path,
    pricing: &PricingDb,
) -> Result<(SessionRecord, Vec<RunRecord>), Box<dyn std::error::Error>> {
    let raw = std::fs::read_to_string(path)?;

    // Deduplicate by uuid — keep the last occurrence of each uuid
    // (later entries may have updated fields for the same message)
    let mut ordered_uuids: Vec<String> = Vec::new();
    let mut by_uuid: HashMap<String, JsonlEntry> = HashMap::new();

    // Also track session metadata from the first entry
    let mut session_id: Option<String> = None;
    let mut git_branch: Option<String> = None;
    let mut cwd: Option<String> = None;

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let entry: JsonlEntry = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(_) => continue, // Skip malformed lines
        };

        // Capture session metadata from first entry
        if session_id.is_none() {
            session_id = entry.session_id.clone();
            git_branch = entry.git_branch.clone();
            cwd = entry.cwd.clone();
        }

        let uuid = entry
            .uuid
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        if !by_uuid.contains_key(&uuid) {
            ordered_uuids.push(uuid.clone());
        }
        by_uuid.insert(uuid, entry);
    }

    let sid = session_id.clone().unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    // Derive project name from path (the directory above the .jsonl file)
    let project = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    let session = SessionRecord {
        id: sid.clone(),
        source: "claude-code".to_string(),
        project: project.clone(),
        title: None, // Claude Code JSONL has no session title
        model: None,
        started_at: None,
        ended_at: None,
        git_branch: git_branch.clone(),
        working_dir: cwd.clone(),
    };

    let mut runs: Vec<RunRecord> = Vec::new();

    for uuid in &ordered_uuids {
        let entry = &by_uuid[uuid];
        let msg = match &entry.message {
            Some(m) => m,
            None => continue,
        };

        // Only process assistant messages that have usage data
        let role = msg.role.as_deref().unwrap_or("user");
        if role != "assistant" {
            continue;
        }
        let usage = msg.usage.as_ref();

        let input_tokens = usage.map(|u| u.input_tokens).unwrap_or(0);
        let output_tokens = usage.map(|u| u.output_tokens).unwrap_or(0);
        let cache_write = usage.map(|u| u.cache_creation_input_tokens).unwrap_or(0);
        let cache_read = usage.map(|u| u.cache_read_input_tokens).unwrap_or(0);

        let model = msg.model.as_deref().unwrap_or("unknown");
        let (cost_usd, _) = pricing.estimate_cost(model, input_tokens, output_tokens, cache_read, cache_write);

        let timestamp = entry
            .timestamp
            .clone()
            .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());

        runs.push(RunRecord {
            id: format!("cc-{}", uuid),
            session_id: Some(sid.clone()),
            source: "claude-code".to_string(),
            model: msg.model.clone(),
            timestamp,
            role: Some(role.to_string()),
            input_tokens,
            output_tokens,
            cache_read_tokens: cache_read,
            cache_write_tokens: cache_write,
            // Always mark as unreliable per the known bug
            tokens_estimated: true,
            data_quality: DATA_QUALITY.to_string(),
            cost_usd,
            cost_estimated: true, // cost based on unreliable token counts
            duration_ms: None,
            project: project.clone(),
            git_branch: git_branch.clone(),
            working_dir: cwd.clone(),
            command: None,
            source_path: Some(path.to_string_lossy().to_string()),
        });
    }

    Ok((session, runs))
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
    use crate::ingest::pricing::PricingDb;
    use tempfile::TempDir;

    fn empty_pricing() -> PricingDb {
        PricingDb::empty()
    }

    fn sample_jsonl() -> &'static str {
        r#"{"parentUuid":null,"sessionId":"sess-cc-001","version":"1.0","gitBranch":"main","cwd":"/home/user/myapp","message":{"role":"user","content":[{"type":"text","text":"Help me refactor auth"}]},"uuid":"uuid-001","timestamp":"2026-03-25T10:00:00Z"}
{"parentUuid":"uuid-001","sessionId":"sess-cc-001","version":"1.0","gitBranch":"main","cwd":"/home/user/myapp","message":{"role":"assistant","model":"claude-sonnet-4-20250514","content":[{"type":"text","text":"Sure, here is..."}],"usage":{"input_tokens":5,"output_tokens":200,"cache_creation_input_tokens":100,"cache_read_input_tokens":500}},"uuid":"uuid-002","timestamp":"2026-03-25T10:01:00Z"}
{"parentUuid":"uuid-001","sessionId":"sess-cc-001","version":"1.0","gitBranch":"main","cwd":"/home/user/myapp","message":{"role":"assistant","model":"claude-sonnet-4-20250514","content":[{"type":"text","text":"Updated response"}],"usage":{"input_tokens":5,"output_tokens":210,"cache_creation_input_tokens":100,"cache_read_input_tokens":500}},"uuid":"uuid-002","timestamp":"2026-03-25T10:01:05Z"}"#
    }

    #[test]
    fn test_parse_jsonl() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("sess-cc-001.jsonl");
        std::fs::write(&path, sample_jsonl()).unwrap();

        let pricing = empty_pricing();
        let (session, _runs) = parse_jsonl_file(&path, &pricing).unwrap();

        assert_eq!(session.id, "sess-cc-001");
        assert_eq!(session.source, "claude-code");
        assert_eq!(session.git_branch.as_deref(), Some("main"));
        assert_eq!(session.working_dir.as_deref(), Some("/home/user/myapp"));
    }

    #[test]
    fn test_deduplication_keeps_last() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("sess-cc-001.jsonl");
        std::fs::write(&path, sample_jsonl()).unwrap();

        let pricing = empty_pricing();
        let (_, runs) = parse_jsonl_file(&path, &pricing).unwrap();

        // uuid-002 appears twice; only one run should be produced (last wins)
        assert_eq!(runs.len(), 1);
        // The last occurrence had output_tokens=210
        assert_eq!(runs[0].output_tokens, 210);
    }

    #[test]
    fn test_always_marked_unreliable() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("sess-cc-001.jsonl");
        std::fs::write(&path, sample_jsonl()).unwrap();

        let pricing = empty_pricing();
        let (_, runs) = parse_jsonl_file(&path, &pricing).unwrap();

        for run in &runs {
            assert!(run.tokens_estimated, "tokens_estimated must always be true for Claude Code");
            assert!(run.cost_estimated, "cost_estimated must always be true for Claude Code");
            assert_eq!(run.data_quality, DATA_QUALITY);
        }
    }

    #[test]
    fn test_cache_fields_preserved() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("sess-cc-001.jsonl");
        std::fs::write(&path, sample_jsonl()).unwrap();

        let pricing = empty_pricing();
        let (_, runs) = parse_jsonl_file(&path, &pricing).unwrap();

        assert_eq!(runs[0].cache_write_tokens, 100);
        assert_eq!(runs[0].cache_read_tokens, 500);
    }

    #[test]
    fn test_ingest_full_flow() {
        let dir = TempDir::new().unwrap();
        let cc_dir = dir.path().join("cc");
        let project_dir = cc_dir.join("myapp");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::write(project_dir.join("sess-cc-001.jsonl"), sample_jsonl()).unwrap();

        let praxis_dir = dir.path().join("praxis");
        let conn = crate::db::init_db(&praxis_dir).unwrap();
        let pricing = empty_pricing();

        let result = ingest(&conn, &pricing, &cc_dir).unwrap();
        assert_eq!(result.source, "claude-code");
        assert_eq!(result.sessions_added, 1);
        assert_eq!(result.runs_added, 1);

        // Second run: skip
        let result2 = ingest(&conn, &pricing, &cc_dir).unwrap();
        assert_eq!(result2.files_skipped, 1);
    }
}
