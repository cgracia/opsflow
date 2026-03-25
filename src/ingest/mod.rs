pub mod ccusage;
pub mod claude_code;
pub mod opencode;
pub mod praxis_native;
pub mod pricing;

use crate::config::PraxisConfig;
use colored::Colorize;
use duckdb::Connection;

/// A normalized session record for the `sessions` table.
#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub id: String,
    pub source: String,
    pub project: Option<String>,
    pub title: Option<String>,
    pub model: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub git_branch: Option<String>,
    pub working_dir: Option<String>,
}

/// A normalized run record for the `runs` table.
#[derive(Debug, Clone)]
pub struct RunRecord {
    pub id: String,
    pub session_id: Option<String>,
    pub source: String,
    pub model: Option<String>,
    pub timestamp: String, // ISO 8601
    pub role: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub tokens_estimated: bool,
    pub data_quality: String, // "reliable" | "claude_code_jsonl_unreliable" | "estimated"
    pub cost_usd: f64,
    pub cost_estimated: bool,
    pub duration_ms: Option<i64>,
    pub project: Option<String>,
    pub git_branch: Option<String>,
    pub working_dir: Option<String>,
    pub command: Option<String>,
    pub source_path: Option<String>,
}

/// Result of ingesting one source.
pub struct IngestResult {
    pub source: String,
    pub sessions_added: usize,
    pub runs_added: usize,
    pub files_skipped: usize,
}

/// Orchestrate ingestion across all enabled sources.
///
/// `source_filter` — if Some, only ingest that source ("opencode", "claude-code", "praxis", "ccusage").
pub fn run_ingest(
    config: &PraxisConfig,
    conn: &Connection,
    source_filter: Option<&str>,
    opencode_dir_override: Option<&std::path::Path>,
    claude_code_dir_override: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pricing = pricing::PricingDb::load(&config.praxis_dir)?;

    let mut results: Vec<IngestResult> = Vec::new();

    let run_source = |name: &str| -> bool {
        source_filter.map_or(true, |f| f == name)
    };

    // Praxis native
    if run_source("praxis") {
        let r = praxis_native::ingest(conn, &pricing, &config.praxis_dir)?;
        results.push(r);
    }

    // OpenCode
    if run_source("opencode") {
        let oc_dir = opencode_dir_override
            .map(|p| p.to_path_buf())
            .or_else(|| config.resolved_opencode_dir());
        if let Some(dir) = oc_dir {
            let r = opencode::ingest(conn, &pricing, &dir)?;
            results.push(r);
        } else if source_filter.is_some() {
            eprintln!("warning: opencode data directory not found; set PRAXIS_OPENCODE_DIR or configure opencode_dir");
        }
    }

    // Claude Code
    if run_source("claude-code") {
        let cc_dir = claude_code_dir_override
            .map(|p| p.to_path_buf())
            .or_else(|| config.resolved_claude_code_dir());
        if let Some(dir) = cc_dir {
            let r = claude_code::ingest(conn, &pricing, &dir)?;
            results.push(r);
        } else if source_filter.is_some() {
            eprintln!("warning: claude-code directory not found; set PRAXIS_CLAUDE_CODE_DIR or configure claude_code_dir");
        }
    }

    // ccusage import
    if run_source("ccusage") {
        let imports_dir = config.praxis_dir.join("imports");
        if imports_dir.exists() || source_filter.is_some() {
            let r = ccusage::ingest(conn, &imports_dir)?;
            results.push(r);
        }
    }

    print_summary(&results);
    Ok(())
}

fn print_summary(results: &[IngestResult]) {
    println!();
    let total_runs: usize = results.iter().map(|r| r.runs_added).sum();
    let total_sessions: usize = results.iter().map(|r| r.sessions_added).sum();

    if results.is_empty() || (total_runs == 0 && total_sessions == 0) {
        println!("{}", "No new data ingested — everything is up to date.".dimmed());
    } else {
        println!("{}", "Ingestion complete:".bold());
        for r in results {
            if r.runs_added > 0 || r.sessions_added > 0 {
                println!(
                    "  {}: {} runs, {} sessions ({} files skipped)",
                    r.source.cyan(),
                    r.runs_added.to_string().bold(),
                    r.sessions_added,
                    r.files_skipped
                );
            }
        }
        println!(
            "\n  Total: {} new runs across {} sessions",
            total_runs.to_string().bold().green(),
            total_sessions
        );
    }
    println!();
}
