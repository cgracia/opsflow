use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::observability::RunMetadata;

pub struct RunPaths {
    pub artifact: PathBuf,
    pub metadata: PathBuf,
}

fn run_dir(praxis_dir: &PathBuf) -> PathBuf {
    praxis_dir.join("runs")
}

fn run_prefix(timestamp: &DateTime<Utc>, short_id: &str) -> String {
    format!("{}-{}", timestamp.format("%Y%m%d-%H%M%S"), short_id)
}

pub fn save_run(
    praxis_dir: &PathBuf,
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

fn build_frontmatter(meta: &RunMetadata) -> String {
    format!(
        "run_id: {}\ncommand: {}\nmodel: {}\ntimestamp: {}\nduration_ms: {}\ntokens_input: {}\ntokens_output: {}\ntokens_input_estimated: {}\ntokens_output_estimated: {}",
        meta.run_id,
        meta.command,
        meta.model,
        meta.timestamp.to_rfc3339(),
        meta.duration_ms,
        meta.tokens_input,
        meta.tokens_output,
        meta.tokens_input_estimated,
        meta.tokens_output_estimated,
    )
}
