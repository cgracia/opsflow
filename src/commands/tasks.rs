use colored::Colorize;

use crate::config::PraxisConfig;
use crate::todoist::TodoistClient;

pub enum TasksCommand {
    List,
    Add {
        content: String,
        due: Option<String>,
        priority: Option<u8>,
    },
    Done {
        id: String,
    },
}

pub fn run(
    config: &PraxisConfig,
    cmd: TasksCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = TodoistClient::new(config)?;

    match cmd {
        TasksCommand::List => {
            let filter = config
                .todoist_pull_filter
                .as_deref()
                .unwrap_or("today | overdue");
            let tasks = client.fetch_tasks(Some(filter))?;

            if tasks.is_empty() {
                println!("No tasks matching filter: {}", filter);
                return Ok(());
            }

            println!("{}", format!("Tasks ({})", tasks.len()).bold());
            println!();

            let mut overdue: Vec<_> = tasks.iter().filter(|t| t.is_overdue()).collect();
            let mut today: Vec<_> = tasks.iter().filter(|t| t.is_due_today()).collect();
            let mut other: Vec<_> = tasks
                .iter()
                .filter(|t| !t.is_overdue() && !t.is_due_today())
                .collect();

            overdue.sort_by(|a, b| b.priority.cmp(&a.priority));
            today.sort_by(|a, b| b.priority.cmp(&a.priority));
            other.sort_by(|a, b| b.priority.cmp(&a.priority));

            if !overdue.is_empty() {
                println!("  {} Overdue", "⚠".red());
                for t in &overdue {
                    let prio = priority_label(t.priority);
                    println!("    [{}] {} {} ({})", t.id.dimmed(), prio, t.content, t.due.as_ref().map(|d| d.date.as_str()).unwrap_or(""));
                }
                println!();
            }

            if !today.is_empty() {
                println!("  {} Due today", "●".yellow());
                for t in &today {
                    let prio = priority_label(t.priority);
                    println!("    [{}] {} {}", t.id.dimmed(), prio, t.content);
                }
                println!();
            }

            if !other.is_empty() {
                println!("  {} Upcoming", "○".cyan());
                for t in &other {
                    let prio = priority_label(t.priority);
                    let due = t.due.as_ref().map(|d| d.date.as_str()).unwrap_or("no date");
                    println!("    [{}] {} {} ({})", t.id.dimmed(), prio, t.content, due);
                }
            }
        }

        TasksCommand::Add { content, due, priority } => {
            // Resolve default project if configured
            let project_id = if let Some(ref proj_name) = config.todoist_project {
                client.resolve_project_id(proj_name).unwrap_or(None)
            } else {
                None
            };

            let task = client.create_task(
                &content,
                due.as_deref(),
                priority,
                project_id.as_deref(),
            )?;
            println!("{} Created task [{}]: {}", "✓".green(), task.id.dimmed(), task.content);
        }

        TasksCommand::Done { id } => {
            client.close_task(&id)?;
            println!("{} Completed task {}", "✓".green(), id.dimmed());
        }
    }

    Ok(())
}

fn priority_label(p: u8) -> colored::ColoredString {
    match p {
        4 => "P1".red().bold(),
        3 => "P2".yellow(),
        2 => "P3".normal(),
        _ => "P4".dimmed(),
    }
}
