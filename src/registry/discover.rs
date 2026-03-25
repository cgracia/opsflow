use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::Workflow;

/// A workflow skeleton produced by auto-discovery.
#[derive(Debug, Clone)]
pub struct DiscoveredWorkflow {
    pub name: String,
    pub workflow: Workflow,
}

/// Scan well-known locations for OpenCode commands and systemd timers,
/// and write a draft `workflows.toml` to `praxis_dir`.
/// Returns the list of discovered workflows.
pub fn run_discover(praxis_dir: &Path) -> Result<Vec<DiscoveredWorkflow>, Box<dyn std::error::Error>> {
    let mut discovered: HashMap<String, DiscoveredWorkflow> = HashMap::new();

    // 1. Scan local .opencode/commands/ (relative to CWD)
    let local_opencode = PathBuf::from(".opencode/commands");
    if local_opencode.exists() {
        for entry in scan_opencode_commands(&local_opencode) {
            discovered.entry(entry.name.clone()).or_insert(entry);
        }
    }

    // 2. Scan global ~/.config/opencode/commands/
    if let Some(home) = dirs_next::home_dir() {
        let global_opencode = home.join(".config").join("opencode").join("commands");
        if global_opencode.exists() {
            for entry in scan_opencode_commands(&global_opencode) {
                discovered.entry(entry.name.clone()).or_insert(entry);
            }
        }
    }

    // 3. Scan ~/.config/systemd/user/*.timer
    if let Some(home) = dirs_next::home_dir() {
        let timer_dir = home.join(".config").join("systemd").join("user");
        if timer_dir.exists() {
            for entry in scan_systemd_timers(&timer_dir) {
                discovered.entry(entry.name.clone()).or_insert(entry);
            }
        }
    }

    let mut list: Vec<DiscoveredWorkflow> = discovered.into_values().collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));

    if !list.is_empty() {
        write_draft_toml(praxis_dir, &list)?;
    }

    Ok(list)
}

fn scan_opencode_commands(dir: &Path) -> Vec<DiscoveredWorkflow> {
    let mut results = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return results;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let description = extract_description_from_markdown(&path)
            .unwrap_or_else(|| "[auto-discovered, please review]".to_string());
        results.push(DiscoveredWorkflow {
            name: name.clone(),
            workflow: Workflow {
                description,
                trigger: "manual".to_string(),
                schedule: None,
                source_command: "opencode".to_string(),
                command_path: path.to_string_lossy().to_string(),
                reads_from: None,
                writes_to: None,
                output_format: Some("markdown".to_string()),
                tags: None,
                expected_frequency_hours: None,
                enabled: Some(true),
            },
        });
    }
    results
}

fn scan_systemd_timers(timer_dir: &Path) -> Vec<DiscoveredWorkflow> {
    let mut results = Vec::new();
    let Ok(entries) = fs::read_dir(timer_dir) else {
        return results;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("timer") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let schedule = extract_systemd_schedule(&path);
        let freq_hours = schedule
            .as_deref()
            .and_then(guess_frequency_hours_from_schedule);
        results.push(DiscoveredWorkflow {
            name: name.clone(),
            workflow: Workflow {
                description: "[auto-discovered, please review]".to_string(),
                trigger: "scheduled".to_string(),
                schedule,
                source_command: "systemd".to_string(),
                command_path: path.to_string_lossy().to_string(),
                reads_from: None,
                writes_to: None,
                output_format: Some("markdown".to_string()),
                tags: None,
                expected_frequency_hours: freq_hours,
                enabled: Some(true),
            },
        });
    }
    results
}

/// Extract description from OpenCode command markdown.
/// Looks for YAML frontmatter `description:` field first, then falls back to
/// the first non-empty, non-heading line.
pub fn extract_description_from_markdown(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    // Check for YAML frontmatter
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let frontmatter = &content[3..3 + end];
            for line in frontmatter.lines() {
                if let Some(rest) = line.strip_prefix("description:") {
                    let desc = rest.trim().trim_matches('"').trim_matches('\'');
                    if !desc.is_empty() {
                        return Some(desc.to_string());
                    }
                }
            }
        }
    }
    // Fall back: first non-empty, non-YAML-fence, non-heading line
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("---") || trimmed.starts_with('#') {
            continue;
        }
        return Some(trimmed.to_string());
    }
    None
}

/// Parse an `OnCalendar=` or `OnBootSec=` entry from a systemd timer file.
fn extract_systemd_schedule(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Some(val) = line.strip_prefix("OnCalendar=") {
            return Some(val.trim().to_string());
        }
    }
    None
}

/// Guess `expected_frequency_hours` from a systemd OnCalendar value.
fn guess_frequency_hours_from_schedule(schedule: &str) -> Option<u64> {
    let s = schedule.to_lowercase();
    if s.contains("hourly") || s == "*:0/1" {
        Some(1)
    } else if s.contains("daily") || s.starts_with("*-*-* ") {
        Some(24)
    } else if s.contains("weekly") || s.contains("mon") {
        Some(168)
    } else {
        None
    }
}

fn write_draft_toml(
    praxis_dir: &Path,
    workflows: &[DiscoveredWorkflow],
) -> Result<(), Box<dyn std::error::Error>> {
    let target = praxis_dir.join("workflows.toml");

    // Don't clobber an existing file — write to a draft path instead
    let (path, label) = if target.exists() {
        (praxis_dir.join("workflows.draft.toml"), "draft")
    } else {
        (target, "workflows.toml")
    };

    let mut out = String::new();
    out.push_str("# Auto-generated by `praxis discover`\n");
    out.push_str("# Review and fill in reads_from, writes_to, tags, and expected_frequency_hours.\n\n");

    for dw in workflows {
        let w = &dw.workflow;
        out.push_str(&format!("[workflows.{}]\n", dw.name));
        out.push_str(&format!("description = {:?}\n", w.description));
        out.push_str(&format!("trigger = {:?}\n", w.trigger));
        if let Some(sched) = &w.schedule {
            out.push_str(&format!("schedule = {:?}\n", sched));
        }
        out.push_str(&format!("source_command = {:?}\n", w.source_command));
        out.push_str(&format!("command_path = {:?}\n", w.command_path));
        out.push_str("reads_from = []                  # TODO: fill in\n");
        out.push_str("writes_to = []                   # TODO: fill in\n");
        out.push_str("output_format = \"markdown\"\n");
        out.push_str("tags = []                        # TODO: fill in\n");
        if let Some(freq) = w.expected_frequency_hours {
            out.push_str(&format!("expected_frequency_hours = {}\n", freq));
        } else {
            out.push_str("# expected_frequency_hours = 24   # TODO: fill in if scheduled\n");
        }
        out.push_str("enabled = true\n\n");
    }

    fs::write(&path, out)?;
    println!(
        "Wrote {} to {}",
        label,
        path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    #[test]
    fn test_extract_description_frontmatter() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cmd.md");
        fs::write(
            &path,
            "---\ndescription: Summarise customer tickets\n---\n\n# Body\n",
        )
        .unwrap();
        let desc = extract_description_from_markdown(&path).unwrap();
        assert_eq!(desc, "Summarise customer tickets");
    }

    #[test]
    fn test_extract_description_fallback() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cmd.md");
        fs::write(&path, "# Heading\n\nThis is the first paragraph.\n").unwrap();
        let desc = extract_description_from_markdown(&path).unwrap();
        assert_eq!(desc, "This is the first paragraph.");
    }

    #[test]
    fn test_extract_description_fixture() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/opencode-commands/customer-tickets.md");
        if path.exists() {
            let desc = extract_description_from_markdown(&path).unwrap();
            assert_eq!(desc, "Summarise new customer tickets and flag P1s");
        }
    }

    #[test]
    fn test_scan_opencode_commands() {
        let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/opencode-commands");
        if fixture_dir.exists() {
            let results = scan_opencode_commands(&fixture_dir);
            assert_eq!(results.len(), 2);
            let names: Vec<&str> = results.iter().map(|r| r.name.as_str()).collect();
            assert!(names.contains(&"customer-tickets"));
            assert!(names.contains(&"performance-review"));
        }
    }

    #[test]
    fn test_guess_frequency_daily() {
        assert_eq!(guess_frequency_hours_from_schedule("daily"), Some(24));
        assert_eq!(guess_frequency_hours_from_schedule("*-*-* 07:00:00"), Some(24));
        assert_eq!(guess_frequency_hours_from_schedule("weekly"), Some(168));
    }

    #[test]
    fn test_write_draft_toml() {
        let dir = TempDir::new().unwrap();
        let workflows = vec![DiscoveredWorkflow {
            name: "test-cmd".to_string(),
            workflow: Workflow {
                description: "A test command".to_string(),
                trigger: "manual".to_string(),
                schedule: None,
                source_command: "opencode".to_string(),
                command_path: ".opencode/commands/test-cmd.md".to_string(),
                reads_from: None,
                writes_to: None,
                output_format: Some("markdown".to_string()),
                tags: None,
                expected_frequency_hours: None,
                enabled: Some(true),
            },
        }];
        write_draft_toml(dir.path(), &workflows).unwrap();
        let content = fs::read_to_string(dir.path().join("workflows.toml")).unwrap();
        assert!(content.contains("[workflows.test-cmd]"));
        assert!(content.contains("description = \"A test command\""));
    }

}
