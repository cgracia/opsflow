/// Ingest praxis-native `.meta.json` run files from `~/.praxis/runs/`.
use crate::db;
use crate::ingest::{IngestResult, RunRecord};
use crate::ingest::pricing::PricingDb;
use duckdb::Connection;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct MetaJson {
    run_id: String,
    timestamp: String,
    command: String,
    model: String,
    #[serde(default)]
    tokens_input: i64,
    #[serde(default)]
    tokens_output: i64,
    #[serde(default)]
    tokens_input_estimated: bool,
    #[serde(default)]
    tokens_output_estimated: bool,
    #[serde(default)]
    cache_read_tokens: i64,
    #[serde(default)]
    cache_write_tokens: i64,
    #[serde(default)]
    duration_ms: Option<i64>,
}

pub fn ingest(
    conn: &Connection,
    pricing: &PricingDb,
    praxis_dir: &Path,
) -> Result<IngestResult, Box<dyn std::error::Error>> {
    let runs_dir = praxis_dir.join("runs");
    if !runs_dir.exists() {
        return Ok(IngestResult {
            source: "praxis".to_string(),
            sessions_added: 0,
            runs_added: 0,
            files_skipped: 0,
        });
    }

    let mut runs_added = 0;
    let mut files_skipped = 0;

    let pattern = runs_dir.join("*.meta.json");
    let pattern_str = pattern.to_string_lossy();

    for entry in glob::glob(&pattern_str)? {
        let path = match entry {
            Ok(p) => p,
            Err(e) => {
                eprintln!("warning: glob error: {}", e);
                continue;
            }
        };

        let path_str = path.to_string_lossy().to_string();

        // Check file modification time for incremental ingestion
        let mtime = file_mtime_str(&path);

        if let Ok(Some(prev_mtime)) = db::check_ingested(conn, "praxis", &path_str) {
            if prev_mtime == mtime {
                files_skipped += 1;
                continue;
            }
        }

        match parse_meta_file(&path, pricing) {
            Ok(run) => {
                db::insert_run(conn, &run)?;
                db::record_ingestion(conn, "praxis", &path_str, &mtime, 1)?;
                runs_added += 1;
            }
            Err(e) => {
                eprintln!("warning: failed to parse {}: {}", path.display(), e);
            }
        }
    }

    Ok(IngestResult {
        source: "praxis".to_string(),
        sessions_added: 0,
        runs_added,
        files_skipped,
    })
}

fn parse_meta_file(
    path: &Path,
    pricing: &PricingDb,
) -> Result<RunRecord, Box<dyn std::error::Error>> {
    let raw = std::fs::read_to_string(path)?;
    let meta: MetaJson = serde_json::from_str(&raw)?;

    let is_estimated = meta.tokens_input_estimated || meta.tokens_output_estimated;
    let (cost_usd, cost_est) = pricing.estimate_cost(
        &meta.model,
        meta.tokens_input,
        meta.tokens_output,
        meta.cache_read_tokens,
        meta.cache_write_tokens,
    );

    Ok(RunRecord {
        id: meta.run_id.clone(),
        session_id: None,
        source: "praxis".to_string(),
        model: Some(meta.model),
        timestamp: meta.timestamp,
        role: None,
        input_tokens: meta.tokens_input,
        output_tokens: meta.tokens_output,
        cache_read_tokens: meta.cache_read_tokens,
        cache_write_tokens: meta.cache_write_tokens,
        tokens_estimated: is_estimated,
        data_quality: if is_estimated {
            "estimated".to_string()
        } else {
            "reliable".to_string()
        },
        cost_usd,
        cost_estimated: cost_est || is_estimated,
        duration_ms: meta.duration_ms,
        project: None,
        git_branch: None,
        working_dir: None,
        command: Some(meta.command),
        source_path: Some(path.to_string_lossy().to_string()),
    })
}

fn file_mtime_str(path: &Path) -> String {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| {
            let secs = t
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            secs.to_string()
        })
        .unwrap_or_else(|_| "0".to_string())
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
    fn test_parse_sample_meta() {
        let dir = TempDir::new().unwrap();
        let meta_json = r#"{
            "run_id": "abc123",
            "timestamp": "2026-03-25T10:00:00Z",
            "command": "think",
            "model": "qwen3.5:9B",
            "tokens_input": 312,
            "tokens_output": 189,
            "tokens_input_estimated": false,
            "tokens_output_estimated": false,
            "duration_ms": 4231,
            "praxis_version": "0.1.0"
        }"#;

        let path = dir.path().join("20260325-100000-abc123-think.meta.json");
        std::fs::write(&path, meta_json).unwrap();

        let pricing = empty_pricing();
        let run = parse_meta_file(&path, &pricing).unwrap();

        assert_eq!(run.id, "abc123");
        assert_eq!(run.source, "praxis");
        assert_eq!(run.model.unwrap(), "qwen3.5:9B");
        assert_eq!(run.input_tokens, 312);
        assert_eq!(run.output_tokens, 189);
        assert!(!run.tokens_estimated);
        assert_eq!(run.data_quality, "reliable");
        assert_eq!(run.duration_ms, Some(4231));
        assert_eq!(run.command.unwrap(), "think");
    }

    #[test]
    fn test_parse_estimated_meta() {
        let dir = TempDir::new().unwrap();
        let meta_json = r#"{
            "run_id": "def456",
            "timestamp": "2026-03-25T11:00:00Z",
            "command": "think",
            "model": "qwen3.5:9B",
            "tokens_input": 100,
            "tokens_output": 50,
            "tokens_input_estimated": true,
            "tokens_output_estimated": false,
            "duration_ms": 2000,
            "praxis_version": "0.1.0"
        }"#;

        let path = dir.path().join("20260325-110000-def456-think.meta.json");
        std::fs::write(&path, meta_json).unwrap();

        let pricing = empty_pricing();
        let run = parse_meta_file(&path, &pricing).unwrap();

        assert!(run.tokens_estimated);
        assert_eq!(run.data_quality, "estimated");
        assert!(run.cost_estimated);
    }

    #[test]
    fn test_ingest_full_flow() {
        let dir = TempDir::new().unwrap();
        let praxis_dir = dir.path();
        let runs_dir = praxis_dir.join("runs");
        std::fs::create_dir_all(&runs_dir).unwrap();

        let meta_json = r#"{"run_id":"xyz789","timestamp":"2026-03-25T12:00:00Z","command":"think","model":"qwen3.5:9B","tokens_input":100,"tokens_output":50,"tokens_input_estimated":false,"tokens_output_estimated":false,"duration_ms":1000,"praxis_version":"0.1.0"}"#;
        std::fs::write(runs_dir.join("20260325-xyz789-think.meta.json"), meta_json).unwrap();

        let conn = init_db(praxis_dir).unwrap();
        let pricing = empty_pricing();
        let result = ingest(&conn, &pricing, praxis_dir).unwrap();

        assert_eq!(result.source, "praxis");
        assert_eq!(result.runs_added, 1);
        assert_eq!(result.files_skipped, 0);

        // Second ingest: should skip the same file
        let result2 = ingest(&conn, &pricing, praxis_dir).unwrap();
        assert_eq!(result2.runs_added, 0);
        assert_eq!(result2.files_skipped, 1);
    }
}
