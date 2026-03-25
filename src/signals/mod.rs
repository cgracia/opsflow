pub mod triage;

use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::registry::{expand_tilde, WorkflowEntry};

/// A signal collected from a workflow output directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub id: String,
    pub workflow_name: String,
    pub file_path: String,
    pub detected_at: String,
    pub file_modified: String,
    pub content_preview: Option<String>,
    pub priority: Option<String>,
    pub triage_reason: Option<String>,
    pub suggested_action: Option<String>,
    pub triaged_at: Option<String>,
    pub acknowledged: bool,
}

/// Options for filtering signals queries.
pub struct SignalsFilter {
    pub untriaged_only: bool,
    pub workflow: Option<String>,
}

/// Collect new signals from all writes_to directories in the workflow registry.
/// Skips files already present in the signals table (deduped by file_path + file_modified).
pub fn collect_signals(
    conn: &Connection,
    workflows: &[WorkflowEntry],
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut total_collected = 0;
    let now = Utc::now().to_rfc3339();

    for entry in workflows {
        if !entry.workflow.is_enabled() {
            continue;
        }
        let dirs = entry.workflow.writes_to.as_deref().unwrap_or(&[]);
        for dir_str in dirs {
            let dir = expand_tilde(dir_str);
            if !dir.exists() {
                continue;
            }

            // Get the last scan time for this directory
            let last_scanned: Option<DateTime<Utc>> = {
                let mut stmt = conn.prepare(
                    "SELECT last_scanned FROM collection_state WHERE directory = ?",
                )?;
                let mut rows = stmt.query(params![dir.to_string_lossy().as_ref()])?;
                if let Some(row) = rows.next()? {
                    let ts: String = row.get(0)?;
                    ts.parse::<DateTime<Utc>>().ok()
                } else {
                    None
                }
            };

            let mut newest_mtime: Option<DateTime<Utc>> = None;

            if let Ok(read_dir) = fs::read_dir(&dir) {
                for file_entry in read_dir.flatten() {
                    let path = file_entry.path();
                    if !path.is_file() {
                        continue;
                    }
                    let Ok(meta) = file_entry.metadata() else {
                        continue;
                    };
                    let Ok(mtime_sys) = meta.modified() else {
                        continue;
                    };
                    let file_modified: DateTime<Utc> = mtime_sys.into();

                    // Only process files newer than our last scan
                    if let Some(last) = last_scanned {
                        if file_modified <= last {
                            continue;
                        }
                    }

                    let file_path = path.to_string_lossy().to_string();
                    let file_modified_str = file_modified.to_rfc3339();

                    // Check for exact duplicate (same path + mtime already collected)
                    let already_exists: bool = {
                        let mut stmt = conn.prepare(
                            "SELECT COUNT(*) FROM signals WHERE file_path = ? AND file_modified = ?",
                        )?;
                        let mut rows = stmt.query(params![file_path, file_modified_str])?;
                        let count: i64 = rows.next()?.map(|r| r.get(0).unwrap_or(0)).unwrap_or(0);
                        count > 0
                    };
                    if already_exists {
                        continue;
                    }

                    let content_preview = read_preview(&path, 500);
                    let signal_id = Uuid::new_v4().to_string();

                    conn.execute(
                        "INSERT INTO signals
                             (id, workflow_name, file_path, detected_at, file_modified,
                              content_preview, priority, triage_reason, suggested_action,
                              triaged_at, acknowledged, acknowledged_at)
                         VALUES (?, ?, ?, ?, ?, ?, NULL, NULL, NULL, NULL, false, NULL)",
                        params![
                            signal_id,
                            entry.name,
                            file_path,
                            now,
                            file_modified_str,
                            content_preview
                        ],
                    )?;

                    // Update workflow_runs to track latest output
                    conn.execute(
                        "INSERT INTO workflow_runs (workflow_name, last_output_at, output_path)
                         VALUES (?, ?, ?)
                         ON CONFLICT (workflow_name) DO UPDATE SET
                             last_output_at = excluded.last_output_at,
                             output_path    = excluded.output_path",
                        params![entry.name, file_modified_str, file_path],
                    )?;

                    total_collected += 1;

                    if newest_mtime.map_or(true, |prev| file_modified > prev) {
                        newest_mtime = Some(file_modified);
                    }
                }
            }

            // Update collection_state for this directory
            let scan_ts = newest_mtime
                .map(|t| t.to_rfc3339())
                .unwrap_or_else(|| now.clone());
            let dir_key = dir.to_string_lossy().to_string();
            conn.execute(
                "INSERT INTO collection_state (directory, last_scanned, last_file_mtime)
                 VALUES (?, ?, ?)
                 ON CONFLICT (directory) DO UPDATE SET
                     last_scanned    = excluded.last_scanned,
                     last_file_mtime = excluded.last_file_mtime",
                params![dir_key, now, scan_ts],
            )?;
        }
    }

    Ok(total_collected)
}

/// Query signals from the database with optional filters.
pub fn get_signals(
    conn: &Connection,
    filter: &SignalsFilter,
) -> Result<Vec<Signal>, Box<dyn std::error::Error>> {
    let mut conditions = Vec::new();
    if filter.untriaged_only {
        conditions.push("priority IS NULL".to_string());
    }
    if let Some(ref wf) = filter.workflow {
        conditions.push(format!("workflow_name = '{}'", wf.replace('\'', "''")));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT id, workflow_name, file_path, detected_at, file_modified,
                content_preview, priority, triage_reason, suggested_action,
                triaged_at, acknowledged
         FROM signals
         {} ORDER BY file_modified DESC",
        where_clause
    );

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([])?;
    let mut signals = Vec::new();

    while let Some(row) = rows.next()? {
        signals.push(Signal {
            id: row.get(0)?,
            workflow_name: row.get(1)?,
            file_path: row.get(2)?,
            detected_at: row.get(3)?,
            file_modified: row.get(4)?,
            content_preview: row.get(5)?,
            priority: row.get(6)?,
            triage_reason: row.get(7)?,
            suggested_action: row.get(8)?,
            triaged_at: row.get(9)?,
            acknowledged: row.get(10)?,
        });
    }
    Ok(signals)
}

/// Read the first `max_chars` characters of a file as a content preview.
fn read_preview(path: &Path, max_chars: usize) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    if content.chars().count() <= max_chars {
        Some(content)
    } else {
        let truncated: String = content.chars().take(max_chars).collect();
        Some(format!("{}…", truncated))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;
    use tempfile::TempDir;

    fn temp_conn() -> (TempDir, Connection) {
        let dir = TempDir::new().unwrap();
        let conn = crate::db::init_db(dir.path()).unwrap();
        (dir, conn)
    }

    fn make_workflow_entry(name: &str, writes_to: &str) -> WorkflowEntry {
        use crate::registry::Workflow;
        WorkflowEntry {
            name: name.to_string(),
            workflow: Workflow {
                description: "Test workflow".to_string(),
                trigger: "scheduled".to_string(),
                schedule: Some("daily 07:00".to_string()),
                source_command: "opencode".to_string(),
                command_path: ".opencode/commands/test.md".to_string(),
                reads_from: None,
                writes_to: Some(vec![writes_to.to_string()]),
                output_format: Some("markdown".to_string()),
                tags: None,
                expected_frequency_hours: Some(24),
                enabled: Some(true),
            },
            last_output_at: None,
            stale: false,
        }
    }

    #[test]
    fn test_collect_signals_empty_dir() {
        let (_db_dir, conn) = temp_conn();
        let output_dir = TempDir::new().unwrap();
        let entry = make_workflow_entry("test-wf", output_dir.path().to_str().unwrap());
        let count = collect_signals(&conn, &[entry]).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_collect_signals_new_file() {
        let (_db_dir, conn) = temp_conn();
        let output_dir = TempDir::new().unwrap();
        let file = output_dir.path().join("report.md");
        let mut f = fs::File::create(&file).unwrap();
        f.write_all(b"# Daily Report\nAll is well.").unwrap();

        let entry = make_workflow_entry("test-wf", output_dir.path().to_str().unwrap());
        let count = collect_signals(&conn, &[entry]).unwrap();
        assert_eq!(count, 1);

        let signals = get_signals(
            &conn,
            &SignalsFilter { untriaged_only: false, workflow: None },
        )
        .unwrap();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].workflow_name, "test-wf");
        assert!(signals[0].content_preview.as_deref().unwrap().contains("Daily Report"));
    }

    #[test]
    fn test_collect_signals_deduplication() {
        let (_db_dir, conn) = temp_conn();
        let output_dir = TempDir::new().unwrap();
        let file = output_dir.path().join("report.md");
        fs::write(&file, b"# Report").unwrap();

        let entry = make_workflow_entry("test-wf", output_dir.path().to_str().unwrap());
        let count1 = collect_signals(&conn, &[entry.clone()]).unwrap();
        let count2 = collect_signals(&conn, &[entry]).unwrap();
        assert_eq!(count1, 1);
        assert_eq!(count2, 0); // deduped
    }

    #[test]
    fn test_get_signals_untriaged_filter() {
        let (_db_dir, conn) = temp_conn();
        let output_dir = TempDir::new().unwrap();
        fs::write(output_dir.path().join("r.md"), b"report").unwrap();

        let entry = make_workflow_entry("wf", output_dir.path().to_str().unwrap());
        collect_signals(&conn, &[entry]).unwrap();

        // All signals start untriaged
        let untriaged = get_signals(
            &conn,
            &SignalsFilter { untriaged_only: true, workflow: None },
        )
        .unwrap();
        assert_eq!(untriaged.len(), 1);

        // Mark as triaged
        conn.execute(
            "UPDATE signals SET priority = 'BACKGROUND', triaged_at = ? WHERE 1=1",
            params![Utc::now().to_rfc3339()],
        )
        .unwrap();

        let untriaged_after = get_signals(
            &conn,
            &SignalsFilter { untriaged_only: true, workflow: None },
        )
        .unwrap();
        assert_eq!(untriaged_after.len(), 0);
    }

    #[test]
    fn test_read_preview_truncation() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("long.md");
        let content = "x".repeat(600);
        fs::write(&path, &content).unwrap();
        let preview = read_preview(&path, 500).unwrap();
        assert!(preview.len() <= 510); // 500 chars + "…"
        assert!(preview.ends_with('…'));
    }
}
