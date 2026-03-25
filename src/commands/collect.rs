use colored::Colorize;

use crate::config::PraxisConfig;
use crate::db;
use crate::registry;
use crate::signals;

pub fn run(config: &PraxisConfig) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db::init_db(&config.praxis_dir)?;
    let entries = registry::load_workflows(&config.praxis_dir)?;

    if entries.is_empty() {
        println!("No workflows registered. Run {} first.", "praxis discover".cyan());
        return Ok(());
    }

    let enabled_count = entries.iter().filter(|e| e.workflow.is_enabled()).count();
    println!(
        "{}",
        format!(
            "Collecting from {} workflows ({} enabled)…",
            entries.len(),
            enabled_count
        )
        .dimmed()
    );

    let count = signals::collect_signals(&conn, &entries)?;

    if count == 0 {
        println!("No new signals. All output directories are up to date.");
    } else {
        println!(
            "{} {} new signal{}.",
            "✓".green(),
            count,
            if count == 1 { "" } else { "s" }
        );
        println!(
            "Run {} to classify them.",
            "praxis triage".cyan()
        );
    }

    Ok(())
}
