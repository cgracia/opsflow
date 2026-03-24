use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, TimeZone, Utc};

use crate::observability::RunMetadata;

pub struct RunPaths {
    pub artifact: PathBuf,
    pub metadata: PathBuf,
}

fn run_dir(praxis_dir: &Path) -> PathBuf {
    praxis_dir.join("runs")
}

fn run_prefix(timestamp: &DateTime<Utc>, short_id: &str) -> String {
    format!("{}-{}", timestamp.format("%Y%m%d-%H%M%S"), short_id)
}

pub fn save_run(
    praxis_dir: &Path,
    timestamp: &DateTime<Utc>,
    short_id: &str,
    command: &str,
    problem: &str,
    output: &str,
    metadata: &RunMetadata,
) -> Result<RunPaths, String> {
    let runs_dir = run_dir(praxis_dir);
    fs::create_dir_all(&runs_dir)
        .map_err(|e| format!("error: Failed to create runs directory: {}", e))?;

    let prefix = run_prefix(timestamp, short_id);
    let artifact_path = runs_dir.join(format!("{}-{}.md", prefix, command));
    let metadata_path = runs_dir.join(format!("{}-{}.meta.json", prefix, command));

    // Build markdown artifact
    let frontmatter = build_frontmatter(metadata);
    let artifact_content = format!(
        "---\n{}\n---\n\n# {}\n\n{}",
        frontmatter, problem, output
    );

    fs::write(&artifact_path, &artifact_content)
        .map_err(|e| format!("error: Failed to write artifact: {}", e))?;

    // Write metadata JSON
    let meta_json = serde_json::to_string_pretty(metadata)
        .map_err(|e| format!("error: Failed to serialize metadata: {}", e))?;

    fs::write(&metadata_path, &meta_json)
        .map_err(|e| format!("error: Failed to write metadata: {}", e))?;

    Ok(RunPaths {
        artifact: artifact_path,
        metadata: metadata_path,
    })
}

/// Wrap a string in YAML double-quotes, escaping backslashes and double-quotes.
fn yaml_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn build_frontmatter(meta: &RunMetadata) -> String {
    format!(
        "run_id: {}\ncommand: {}\nmodel: {}\ntimestamp: {}\nduration_ms: {}\ntokens_input: {}\ntokens_output: {}\ntokens_input_estimated: {}\ntokens_output_estimated: {}",
        yaml_quote(&meta.run_id),
        yaml_quote(&meta.command),
        yaml_quote(&meta.model),
        yaml_quote(&meta.timestamp.to_rfc3339()),
        meta.duration_ms,
        meta.tokens_input,
        meta.tokens_output,
        meta.tokens_input_estimated,
        meta.tokens_output_estimated,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::build_metadata;

    fn make_meta(model: &str) -> RunMetadata {
        let ts = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
        build_metadata("test-run-id", ts, "think", "input", "output", model, Some(10), Some(20), 100)
    }

    // --- yaml_quote ---

    #[test]
    fn yaml_quote_plain_string() {
        assert_eq!(yaml_quote("hello"), "\"hello\"");
    }

    #[test]
    fn yaml_quote_colon_in_value() {
        // "llama3:8b" is common Ollama syntax and must not break YAML
        assert_eq!(yaml_quote("llama3:8b"), "\"llama3:8b\"");
    }

    #[test]
    fn yaml_quote_escapes_backslash() {
        assert_eq!(yaml_quote("a\\b"), "\"a\\\\b\"");
    }

    #[test]
    fn yaml_quote_escapes_double_quote() {
        assert_eq!(yaml_quote("say \"hi\""), "\"say \\\"hi\\\"\"");
    }

    // --- run_prefix ---

    #[test]
    fn run_prefix_format() {
        let ts = Utc.with_ymd_and_hms(2024, 6, 15, 12, 34, 56).unwrap();
        assert_eq!(run_prefix(&ts, "abc12345"), "20240615-123456-abc12345");
    }

    // --- build_frontmatter ---

    #[test]
    fn frontmatter_quotes_model_with_colon() {
        let meta = make_meta("llama3:8b");
        let fm = build_frontmatter(&meta);
        assert!(fm.contains("model: \"llama3:8b\""));
    }

    #[test]
    fn frontmatter_contains_all_fields() {
        let meta = make_meta("llama3");
        let fm = build_frontmatter(&meta);
        assert!(fm.contains("run_id:"));
        assert!(fm.contains("command:"));
        assert!(fm.contains("model:"));
        assert!(fm.contains("timestamp:"));
        assert!(fm.contains("duration_ms:"));
        assert!(fm.contains("tokens_input:"));
        assert!(fm.contains("tokens_output:"));
        assert!(fm.contains("tokens_input_estimated:"));
        assert!(fm.contains("tokens_output_estimated:"));
    }

    #[test]
    fn frontmatter_numeric_fields_unquoted() {
        let meta = make_meta("llama3");
        let fm = build_frontmatter(&meta);
        assert!(fm.contains("duration_ms: 100"));
        assert!(fm.contains("tokens_input: 10"));
        assert!(fm.contains("tokens_output: 20"));
    }

    // --- save_run integration ---

    #[test]
    fn save_run_creates_files() {
        let dir = tempfile::tempdir().unwrap();
        let ts = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
        let meta = make_meta("llama3");

        let paths = save_run(dir.path(), &ts, "abcd1234", "think", "my problem", "my output", &meta)
            .unwrap();

        assert!(paths.artifact.exists());
        assert!(paths.metadata.exists());
    }

    #[test]
    fn save_run_artifact_contains_problem_and_output() {
        let dir = tempfile::tempdir().unwrap();
        let ts = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
        let meta = make_meta("llama3");

        let paths = save_run(dir.path(), &ts, "abcd1234", "think", "my problem", "my output", &meta)
            .unwrap();

        let content = fs::read_to_string(&paths.artifact).unwrap();
        assert!(content.contains("my problem"));
        assert!(content.contains("my output"));
        assert!(content.starts_with("---\n"));
    }

    #[test]
    fn save_run_metadata_is_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let ts = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
        let meta = make_meta("llama3");

        let paths = save_run(dir.path(), &ts, "abcd1234", "think", "problem", "output", &meta)
            .unwrap();

        let json_str = fs::read_to_string(&paths.metadata).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["model"], "llama3");
        assert_eq!(parsed["command"], "think");
    }
}
