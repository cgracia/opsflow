use crate::config::PraxisConfig;
use crate::db;
use crate::registry;
use crate::signals::{self, triage};

use super::status;

pub fn run(config: &PraxisConfig) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db::init_db(&config.praxis_dir)?;
    let mut entries = registry::load_workflows(&config.praxis_dir)?;

    // Step 1: Collect
    if !entries.is_empty() {
        let count = signals::collect_signals(&conn, &entries)?;
        if count > 0 {
            eprintln!("  collected {} new signal{}", count, if count == 1 { "" } else { "s" });
        }
    }

    // Step 2: Triage (only if there are untriaged signals)
    let untriaged = signals::get_signals(
        &conn,
        &signals::SignalsFilter { untriaged_only: true, workflow: None },
    )?;
    if !untriaged.is_empty() {
        match triage::run_triage(&conn, config, &entries) {
            Ok(result) => {
                if result.items_triaged > 0 {
                    eprintln!("  triaged {} item{}", result.items_triaged, if result.items_triaged == 1 { "" } else { "s" });
                }
            }
            Err(e) => {
                eprintln!("  warning: triage failed: {}", e);
                eprintln!("  continuing to status…");
            }
        }
    }

    // Step 3: Status
    registry::check_staleness(&mut entries, &conn)?;
    status::run(config, status::StatusOptions { urgent_only: false, json: false })
}
