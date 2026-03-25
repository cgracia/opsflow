use colored::Colorize;

use crate::config::PraxisConfig;
use crate::db;
use crate::signals::{get_signals, SignalsFilter};

pub struct SignalsOptions {
    pub untriaged_only: bool,
    pub workflow: Option<String>,
}

pub fn run(
    config: &PraxisConfig,
    opts: SignalsOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db::init_db(&config.praxis_dir)?;

    let filter = SignalsFilter {
        untriaged_only: opts.untriaged_only,
        workflow: opts.workflow.clone(),
    };

    let signals = get_signals(&conn, &filter)?;

    if signals.is_empty() {
        if opts.untriaged_only {
            println!("No untriaged signals. Run {} to collect new outputs.", "praxis collect".cyan());
        } else {
            println!("No signals found.");
        }
        return Ok(());
    }

    let label = if opts.untriaged_only {
        format!("Untriaged signals ({})", signals.len())
    } else {
        format!("Signals ({})", signals.len())
    };
    println!("{}", label.bold());
    println!();

    for sig in &signals {
        let priority_str = match sig.priority.as_deref() {
            Some("ACT_NOW") => "ACT_NOW".red().bold().to_string(),
            Some("REVIEW_TODAY") => "REVIEW_TODAY".yellow().to_string(),
            Some("REVIEW_WEEK") => "REVIEW_WEEK".cyan().to_string(),
            Some("BACKGROUND") => "BACKGROUND".dimmed().to_string(),
            Some("IGNORE") => "IGNORE".dimmed().to_string(),
            Some(other) => other.to_string(),
            None => "untriaged".dimmed().to_string(),
        };

        println!("  [{}] {} — {}", priority_str, sig.workflow_name.cyan(), sig.file_path);
        println!("    Modified: {}", sig.file_modified);

        if let Some(reason) = &sig.triage_reason {
            println!("    Reason: {}", reason);
        }
        if let Some(action) = &sig.suggested_action {
            println!("    Action: {}", action.bold());
        }
        println!();
    }

    Ok(())
}
