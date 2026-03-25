pub mod discover;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};

/// A single workflow definition from workflows.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub description: String,
    /// manual | scheduled | on-demand
    pub trigger: String,
    /// Human-readable schedule string, e.g. "daily 07:00"
    pub schedule: Option<String>,
    /// opencode | systemd | script | cron
    pub source_command: String,
    pub command_path: String,
    pub reads_from: Option<Vec<String>>,
    pub writes_to: Option<Vec<String>>,
    pub output_format: Option<String>,
    pub tags: Option<Vec<String>>,
    /// Used to detect staleness: how often (hours) this workflow is expected to run.
    pub expected_frequency_hours: Option<u64>,
    pub enabled: Option<bool>,
}

impl Workflow {
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }
}

/// A workflow entry enriched with runtime health information.
#[derive(Debug, Clone)]
pub struct WorkflowEntry {
    pub name: String,
    pub workflow: Workflow,
    /// Most recent output file timestamp (from DB or filesystem scan).
    pub last_output_at: Option<DateTime<Utc>>,
    /// True if enabled and past expected_frequency_hours since last output.
    pub stale: bool,
}

#[derive(Deserialize)]
struct WorkflowsFile {
    #[serde(default)]
    workflows: HashMap<String, Workflow>,
}

/// Load all workflow entries from `<praxis_dir>/workflows.toml`.
/// Returns an empty list if the file does not exist.
pub fn load_workflows(praxis_dir: &Path) -> Result<Vec<WorkflowEntry>, Box<dyn std::error::Error>> {
    let path = praxis_dir.join("workflows.toml");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = fs::read_to_string(&path)?;
    let file: WorkflowsFile = toml::from_str(&contents)
        .map_err(|e| format!("failed to parse workflows.toml: {}", e))?;

    let mut entries: Vec<WorkflowEntry> = file
        .workflows
        .into_iter()
        .map(|(name, workflow)| WorkflowEntry {
            name,
            workflow,
            last_output_at: None,
            stale: false,
        })
        .collect();

    // Stable sort by name for consistent output
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

/// Enrich workflow entries with last_output_at and staleness.
/// Checks DB first (workflow_runs table), then falls back to scanning writes_to directories.
pub fn check_staleness(
    entries: &mut Vec<WorkflowEntry>,
    conn: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in entries.iter_mut() {
        // Try DB first
        let db_ts: Option<String> = {
            let mut stmt =
                conn.prepare("SELECT last_output_at FROM workflow_runs WHERE workflow_name = ?")?;
            let mut rows = stmt.query(params![entry.name])?;
            if let Some(row) = rows.next()? {
                row.get(0)?
            } else {
                None
            }
        };

        let last_output_at: Option<DateTime<Utc>> = if let Some(ts) = db_ts {
            ts.parse::<DateTime<Utc>>().ok()
        } else {
            scan_latest_mtime(&entry.workflow)
        };

        entry.last_output_at = last_output_at;

        if let Some(freq_hours) = entry.workflow.expected_frequency_hours {
            if entry.workflow.is_enabled() {
                entry.stale = match last_output_at {
                    None => true,
                    Some(last) => {
                        let age = Utc::now() - last;
                        age.num_hours() as u64 > freq_hours
                    }
                };
            }
        }
    }
    Ok(())
}

/// Scan writes_to directories for the most recently modified file.
fn scan_latest_mtime(workflow: &Workflow) -> Option<DateTime<Utc>> {
    let dirs = workflow.writes_to.as_deref().unwrap_or(&[]);
    let mut latest: Option<DateTime<Utc>> = None;
    for dir_str in dirs {
        let dir = expand_tilde(dir_str);
        if let Ok(read_dir) = fs::read_dir(&dir) {
            for entry in read_dir.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        if let Ok(mtime) = meta.modified() {
                            let mtime_utc: DateTime<Utc> = mtime.into();
                            if latest.map_or(true, |prev| mtime_utc > prev) {
                                latest = Some(mtime_utc);
                            }
                        }
                    }
                }
            }
        }
    }
    latest
}

/// Expand a leading `~/` to the user's home directory.
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs_next::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn temp_workflows_toml(content: &str) -> TempDir {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("workflows.toml");
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        dir
    }

    #[test]
    fn test_load_workflows_missing_file() {
        let dir = TempDir::new().unwrap();
        let entries = load_workflows(dir.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_load_workflows_parses_toml() {
        let toml_content = r#"
[workflows.customer-tickets]
description = "Summarise new customer tickets and flag P1s"
trigger = "scheduled"
schedule = "daily 07:00"
source_command = "opencode"
command_path = ".opencode/commands/customer-tickets.md"
reads_from = ["jira-mirror", "mcp-rag"]
writes_to = ["/tmp/reports/tickets/"]
output_format = "markdown"
tags = ["support", "triage", "daily"]
expected_frequency_hours = 24
enabled = true

[workflows.manual-review]
description = "Manual quarterly review"
trigger = "manual"
source_command = "opencode"
command_path = ".opencode/commands/review.md"
enabled = true
"#;
        let dir = temp_workflows_toml(toml_content);
        let entries = load_workflows(dir.path()).unwrap();
        assert_eq!(entries.len(), 2);
        // Entries are sorted by name
        assert_eq!(entries[0].name, "customer-tickets");
        assert_eq!(entries[1].name, "manual-review");
        assert_eq!(
            entries[0].workflow.description,
            "Summarise new customer tickets and flag P1s"
        );
        assert_eq!(entries[0].workflow.expected_frequency_hours, Some(24));
        assert!(entries[0].workflow.is_enabled());
        let tags = entries[0].workflow.tags.as_deref().unwrap();
        assert!(tags.contains(&"support".to_string()));
    }

    #[test]
    fn test_load_workflows_fixture_file() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/workflows.toml");
        if fixture.exists() {
            let entries = load_workflows(fixture.parent().unwrap()).unwrap();
            assert!(entries.len() >= 3);
        }
    }

    #[test]
    fn test_staleness_no_frequency() {
        let toml_content = r#"
[workflows.manual-task]
description = "No frequency set"
trigger = "manual"
source_command = "opencode"
command_path = ".opencode/commands/task.md"
"#;
        let dir = temp_workflows_toml(toml_content);
        let mut entries = load_workflows(dir.path()).unwrap();
        let conn_dir = TempDir::new().unwrap();
        let conn = crate::db::init_db(conn_dir.path()).unwrap();
        check_staleness(&mut entries, &conn).unwrap();
        // No expected_frequency_hours → never stale
        assert!(!entries[0].stale);
    }

    #[test]
    fn test_staleness_never_ran() {
        let toml_content = r#"
[workflows.daily-report]
description = "Daily report"
trigger = "scheduled"
schedule = "daily 07:00"
source_command = "opencode"
command_path = ".opencode/commands/report.md"
writes_to = ["/tmp/nonexistent-praxis-test-dir-xyz/"]
expected_frequency_hours = 24
enabled = true
"#;
        let dir = temp_workflows_toml(toml_content);
        let mut entries = load_workflows(dir.path()).unwrap();
        let conn_dir = TempDir::new().unwrap();
        let conn = crate::db::init_db(conn_dir.path()).unwrap();
        check_staleness(&mut entries, &conn).unwrap();
        // No output ever → stale
        assert!(entries[0].stale);
    }

    #[test]
    fn test_expand_tilde() {
        let result = expand_tilde("~/foo/bar");
        let s = result.to_string_lossy();
        assert!(!s.starts_with('~'));
        assert!(s.ends_with("foo/bar"));
    }
}
