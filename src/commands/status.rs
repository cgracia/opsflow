use colored::Colorize;
use serde::Serialize;

use crate::config::PraxisConfig;
use crate::db;
use crate::registry;
use crate::signals::{get_signals, SignalsFilter};
use crate::todoist::TodoistClient;

pub struct StatusOptions {
    pub urgent_only: bool,
    pub json: bool,
}

#[derive(Serialize)]
struct StatusJson {
    act_now: Vec<SignalJson>,
    review_today: Vec<SignalJson>,
    background: Vec<SignalJson>,
    todoist: TodoistSummaryJson,
    workflows: WorkflowsHealthJson,
}

#[derive(Serialize)]
struct SignalJson {
    id: String,
    workflow: String,
    file: String,
    reason: Option<String>,
    suggested_action: Option<String>,
}

#[derive(Serialize)]
struct TodoistSummaryJson {
    active: usize,
    due_today: usize,
    overdue: usize,
}

#[derive(Serialize)]
struct WorkflowsHealthJson {
    total: usize,
    healthy: usize,
    stale: Vec<String>,
}

pub fn run(
    config: &PraxisConfig,
    opts: StatusOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db::init_db(&config.praxis_dir)?;
    let mut workflow_entries = registry::load_workflows(&config.praxis_dir)?;
    registry::check_staleness(&mut workflow_entries, &conn)?;

    // Signals by priority
    let all_signals = get_signals(&conn, &SignalsFilter { untriaged_only: false, workflow: None })?;

    let act_now: Vec<_> = all_signals
        .iter()
        .filter(|s| s.priority.as_deref() == Some("ACT_NOW"))
        .collect();
    let review_today: Vec<_> = all_signals
        .iter()
        .filter(|s| s.priority.as_deref() == Some("REVIEW_TODAY"))
        .collect();
    let review_week: Vec<_> = all_signals
        .iter()
        .filter(|s| s.priority.as_deref() == Some("REVIEW_WEEK"))
        .collect();
    let background: Vec<_> = all_signals
        .iter()
        .filter(|s| s.priority.as_deref() == Some("BACKGROUND"))
        .collect();
    let untriaged: Vec<_> = all_signals
        .iter()
        .filter(|s| s.priority.is_none())
        .collect();

    // Todoist tasks (optional — gracefully skip if not configured)
    let todoist_tasks = if config.todoist_api_key.is_some() {
        match TodoistClient::new(config) {
            Ok(client) => {
                let filter = config
                    .todoist_pull_filter
                    .as_deref()
                    .unwrap_or("today | overdue");
                client.fetch_tasks(Some(filter)).unwrap_or_default()
            }
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    let tasks_due_today: Vec<_> = todoist_tasks.iter().filter(|t| t.is_due_today()).collect();
    let tasks_overdue: Vec<_> = todoist_tasks.iter().filter(|t| t.is_overdue()).collect();

    // Workflow health
    let stale_workflows: Vec<_> = workflow_entries.iter().filter(|e| e.stale).collect();
    let healthy_count = workflow_entries.len() - stale_workflows.len();

    if opts.json {
        let data = StatusJson {
            act_now: act_now
                .iter()
                .map(|s| SignalJson {
                    id: s.id.clone(),
                    workflow: s.workflow_name.clone(),
                    file: s.file_path.clone(),
                    reason: s.triage_reason.clone(),
                    suggested_action: s.suggested_action.clone(),
                })
                .collect(),
            review_today: review_today
                .iter()
                .map(|s| SignalJson {
                    id: s.id.clone(),
                    workflow: s.workflow_name.clone(),
                    file: s.file_path.clone(),
                    reason: s.triage_reason.clone(),
                    suggested_action: s.suggested_action.clone(),
                })
                .collect(),
            background: background
                .iter()
                .chain(review_week.iter())
                .map(|s| SignalJson {
                    id: s.id.clone(),
                    workflow: s.workflow_name.clone(),
                    file: s.file_path.clone(),
                    reason: s.triage_reason.clone(),
                    suggested_action: s.suggested_action.clone(),
                })
                .collect(),
            todoist: TodoistSummaryJson {
                active: todoist_tasks.len(),
                due_today: tasks_due_today.len(),
                overdue: tasks_overdue.len(),
            },
            workflows: WorkflowsHealthJson {
                total: workflow_entries.len(),
                healthy: healthy_count,
                stale: stale_workflows.iter().map(|e| e.name.clone()).collect(),
            },
        };
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    let sep = "─".repeat(65);
    let today = chrono::Utc::now().format("%Y-%m-%d");

    println!(
        "{}\n{}\n{}",
        sep,
        format!("Praxis — Morning Status ({})", today).bold(),
        sep
    );

    // --- ACT NOW ---
    if !act_now.is_empty() || (!opts.urgent_only && !act_now.is_empty()) {
        print_section_header("🔴", "ACT NOW", act_now.len(), "red");
        for sig in &act_now {
            let desc = sig.triage_reason.as_deref().unwrap_or("—");
            println!(
                "  • [{}] {}",
                sig.workflow_name.red().bold(),
                desc
            );
            if let Some(action) = &sig.suggested_action {
                println!("    → {}", action.bold());
            }
        }
        println!();
    }

    if opts.urgent_only {
        if act_now.is_empty() {
            println!("  No urgent items.");
        }
        println!("{}", sep);
        return Ok(());
    }

    // --- REVIEW TODAY ---
    let review_today_total = review_today.len() + tasks_due_today.len() + tasks_overdue.len();
    if review_today_total > 0 {
        print_section_header("🟡", "REVIEW TODAY", review_today_total, "yellow");
        for sig in &review_today {
            let desc = sig.triage_reason.as_deref().unwrap_or("—");
            println!("  • [{}] {}", sig.workflow_name.yellow(), desc);
        }
        if !tasks_overdue.is_empty() {
            for t in &tasks_overdue {
                println!(
                    "  • {} {} {}",
                    "Todoist (overdue):".red(),
                    t.content,
                    format!("[overdue {}]", t.due.as_ref().map(|d| d.date.as_str()).unwrap_or("")).dimmed()
                );
            }
        }
        if !tasks_due_today.is_empty() {
            for t in &tasks_due_today {
                println!("  • {} {}", "Todoist:".yellow(), t.content);
            }
        }
        println!();
    }

    // --- BACKGROUND ---
    let bg_total = background.len() + review_week.len();
    if bg_total > 0 {
        print_section_header("🟢", "BACKGROUND", bg_total, "green");
        let combined: Vec<_> = background.iter().chain(review_week.iter()).collect();
        for sig in combined.iter().take(5) {
            let desc = sig.triage_reason.as_deref().unwrap_or("routine output");
            println!("  • [{}] {}", sig.workflow_name.dimmed(), desc);
        }
        if combined.len() > 5 {
            println!("  … and {} more", combined.len() - 5);
        }
        println!();
    }

    // --- UNTRIAGED ---
    if !untriaged.is_empty() {
        println!(
            "  {} {} signal{} not yet triaged. Run {} to classify.",
            "⚠".yellow(),
            untriaged.len(),
            if untriaged.len() == 1 { "" } else { "s" },
            "praxis triage".cyan()
        );
        println!();
    }

    // --- TODOIST summary ---
    if !todoist_tasks.is_empty() {
        println!(
            "{}",
            format!(
                "📋 TODOIST ({} active tasks, {} due today, {} overdue)",
                todoist_tasks.len(),
                tasks_due_today.len(),
                tasks_overdue.len()
            )
            .bold()
        );
        println!();
    } else if config.todoist_api_key.is_some() {
        println!("{}", "📋 TODOIST  (no tasks matching filter)".dimmed());
        println!();
    }

    // --- WORKFLOWS health ---
    if !workflow_entries.is_empty() {
        println!(
            "{}",
            format!(
                "⚙️  WORKFLOWS ({} registered, {} healthy, {} stale)",
                workflow_entries.len(),
                healthy_count,
                stale_workflows.len()
            )
            .bold()
        );
        for entry in &stale_workflows {
            let age_str = entry
                .last_output_at
                .map(|ts| {
                    let hours = (chrono::Utc::now() - ts).num_hours();
                    if hours < 48 {
                        format!("last ran {}h ago", hours)
                    } else {
                        format!("last ran {}d ago", hours / 24)
                    }
                })
                .unwrap_or_else(|| "never ran".to_string());
            let freq_str = entry
                .workflow
                .expected_frequency_hours
                .map(|h| if h >= 168 { "expected: weekly".to_string() } else { format!("expected: every {}h", h) })
                .unwrap_or_default();
            println!(
                "  • {} {}: {} ({})",
                "⚠️".yellow(),
                entry.name.yellow(),
                age_str,
                freq_str
            );
        }
        println!();
    }

    // --- AI usage hint (from Phase 2 stats if available) ---
    print_ai_usage_hint(&conn);

    println!("{}", sep);

    Ok(())
}

fn print_section_header(icon: &str, label: &str, count: usize, _color: &str) {
    println!("{} {} ({})", icon, label.bold(), count);
}

/// Print a brief AI usage line if there's notable spend in the last 24h.
fn print_ai_usage_hint(conn: &duckdb::Connection) {
    // Sum cost for last 24h vs 7-day daily average
    let sql_24h = "SELECT COALESCE(SUM(cost_usd), 0.0) FROM runs
                   WHERE timestamp >= (CURRENT_TIMESTAMP - INTERVAL '1 day')";
    let sql_7d_avg = "SELECT COALESCE(AVG(daily_cost), 0.0) FROM (
                          SELECT DATE_TRUNC('day', CAST(timestamp AS TIMESTAMP)) AS day,
                                 SUM(cost_usd) AS daily_cost
                          FROM runs
                          WHERE timestamp >= (CURRENT_TIMESTAMP - INTERVAL '7 days')
                          GROUP BY 1
                      ) t";

    let cost_24h: f64 = conn
        .prepare(sql_24h)
        .and_then(|mut s| {
            let mut rows = s.query([])?;
            match rows.next()? {
                Some(row) => Ok(Some(row.get::<_, f64>(0)?)),
                None => Ok(None),
            }
        })
        .ok()
        .flatten()
        .unwrap_or(0.0);

    let avg_7d: f64 = conn
        .prepare(sql_7d_avg)
        .and_then(|mut s| {
            let mut rows = s.query([])?;
            match rows.next()? {
                Some(row) => Ok(Some(row.get::<_, f64>(0)?)),
                None => Ok(None),
            }
        })
        .ok()
        .flatten()
        .unwrap_or(0.0);

    if cost_24h > 0.01 {
        let msg = if avg_7d > 0.001 && cost_24h > avg_7d * 1.5 {
            format!(
                "💰 AI spend yesterday: ${:.2} — {:.0}x your 7-day average",
                cost_24h,
                cost_24h / avg_7d
            )
        } else {
            format!("💰 AI spend yesterday: ${:.2}", cost_24h)
        };
        println!("{}", msg.dimmed());
        println!();
    }
}
