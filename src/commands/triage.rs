use colored::Colorize;

use crate::config::PraxisConfig;
use crate::db;
use crate::registry;
use crate::signals::triage;

pub fn run(config: &PraxisConfig) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db::init_db(&config.praxis_dir)?;
    let entries = registry::load_workflows(&config.praxis_dir)?;

    println!("{}", "Running triage…".dimmed());

    let result = triage::run_triage(&conn, config, &entries)?;

    if result.items_triaged == 0 {
        println!("No untriaged signals. Run {} first.", "praxis collect".cyan());
        return Ok(());
    }

    println!(
        "{} Triaged {} item{} using {}.",
        "✓".green(),
        result.items_triaged,
        if result.items_triaged == 1 { "" } else { "s" },
        result.model
    );

    if let (Some(input), Some(output)) = (result.input_tokens, result.output_tokens) {
        let cache_note = match result.cache_read_tokens {
            Some(n) if n > 0 => format!("  ({} cache hit)", n),
            _ => String::new(),
        };
        println!(
            "  Tokens: {} in / {} out{}  ({:.0}ms)",
            input, output, cache_note, result.duration_ms
        );
    }

    println!("Run {} to see the prioritised dashboard.", "praxis status".cyan());

    Ok(())
}
