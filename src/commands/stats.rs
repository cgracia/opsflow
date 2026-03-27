use crate::config::PraxisConfig;
use crate::db::init_db;
use colored::Colorize;
use duckdb::Connection;

pub struct StatsOptions {
    pub by: Option<String>, // "day" | "model" | "project" | "source"
    pub since: Option<String>,
    pub until: Option<String>,
    pub json: bool,
}

pub fn run(config: &PraxisConfig, opts: StatsOptions) -> Result<(), Box<dyn std::error::Error>> {
    let conn = init_db(&config.praxis_dir)?;

    match opts.by.as_deref() {
        None => print_summary(&conn, &opts),
        Some("day") => print_by_day(&conn, &opts),
        Some("model") => print_by_group(&conn, &opts, "model"),
        Some("project") => print_by_group(&conn, &opts, "project"),
        Some("source") => print_by_group(&conn, &opts, "source"),
        Some(other) => {
            eprintln!("error: unknown --by value '{}'; valid: day, model, project, source", other);
            std::process::exit(1);
        }
    }
}

struct SummaryStats {
    total_runs: i64,
    total_input: i64,
    total_output: i64,
    estimated_cost: f64,
    has_unreliable: bool,
}

fn build_where_clause(opts: &StatsOptions) -> String {
    let mut conditions: Vec<String> = Vec::new();

    if let Some(ref s) = opts.since {
        if let Some(ts) = parse_date(s) {
            conditions.push(format!("timestamp >= '{}'", ts));
        }
    }
    if let Some(ref u) = opts.until {
        if let Some(ts) = parse_date_end(u) {
            conditions.push(format!("timestamp < '{}'", ts));
        }
    }

    if conditions.is_empty() {
        // Default: last 30 days
        use chrono::{Duration, Utc};
        let since = Utc::now() - Duration::days(30);
        conditions.push(format!(
            "timestamp >= '{}'",
            since.format("%Y-%m-%dT%H:%M:%SZ")
        ));
    }

    format!("WHERE {}", conditions.join(" AND "))
}

fn query_summary(
    conn: &Connection,
    opts: &StatsOptions,
) -> Result<SummaryStats, Box<dyn std::error::Error>> {
    let where_clause = build_where_clause(opts);
    let sql = format!(
        "SELECT
             COUNT(*)             AS total_runs,
             SUM(input_tokens)    AS total_input,
             SUM(output_tokens)   AS total_output,
             SUM(cost_usd)        AS estimated_cost,
             SUM(CASE WHEN data_quality LIKE '%unreliable%' THEN 1 ELSE 0 END) AS unreliable_count
         FROM runs
         {}",
        where_clause
    );

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        Ok(SummaryStats {
            total_runs: row.get::<_, i64>(0).unwrap_or(0),
            total_input: row.get::<_, i64>(1).unwrap_or(0),
            total_output: row.get::<_, i64>(2).unwrap_or(0),
            estimated_cost: row.get::<_, f64>(3).unwrap_or(0.0),
            has_unreliable: row.get::<_, i64>(4).unwrap_or(0) > 0,
        })
    } else {
        Ok(SummaryStats {
            total_runs: 0,
            total_input: 0,
            total_output: 0,
            estimated_cost: 0.0,
            has_unreliable: false,
        })
    }
}

fn print_summary(conn: &Connection, opts: &StatsOptions) -> Result<(), Box<dyn std::error::Error>> {
    let stats = query_summary(conn, opts)?;
    let where_clause = build_where_clause(opts);

    // Source breakdown
    let source_rows = query_by(&conn, &where_clause, "source", 5)?;
    // Top models
    let model_rows = query_by(&conn, &where_clause, "model", 5)?;
    // Top projects
    let project_rows = query_by(&conn, &where_clause, "project", 5)?;

    let period_label = period_desc(opts);

    if opts.json {
        let obj = serde_json::json!({
            "period": period_label,
            "total_runs": stats.total_runs,
            "total_input_tokens": stats.total_input,
            "total_output_tokens": stats.total_output,
            "estimated_cost_usd": stats.estimated_cost,
            "has_unreliable_data": stats.has_unreliable,
            "by_source": source_rows.iter().map(|r| serde_json::json!({"name": r.0, "runs": r.1, "cost_usd": r.2})).collect::<Vec<_>>(),
            "top_models": model_rows.iter().map(|r| serde_json::json!({"name": r.0, "runs": r.1, "cost_usd": r.2})).collect::<Vec<_>>(),
            "top_projects": project_rows.iter().map(|r| serde_json::json!({"name": r.0, "runs": r.1, "cost_usd": r.2})).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&obj)?);
        return Ok(());
    }

    println!();
    println!(
        "{}",
        format!("Praxis — AI Usage Summary ({})", period_label).bold()
    );
    println!("{}", "─".repeat(50).dimmed());

    println!(
        "  {:20} {}",
        "Total runs:".dimmed(),
        fmt_count(stats.total_runs).bold()
    );
    println!(
        "  {:20} {} in / {} out",
        "Total tokens:".dimmed(),
        fmt_tokens_m(stats.total_input).bold(),
        fmt_tokens_m(stats.total_output).bold()
    );

    let cost_str = if stats.has_unreliable {
        format!("~${:.2} (includes unreliable data)", stats.estimated_cost)
            .yellow()
            .to_string()
    } else {
        format!("${:.2}", stats.estimated_cost).green().to_string()
    };
    println!("  {:20} {}", "Estimated cost:".dimmed(), cost_str);

    if !source_rows.is_empty() {
        let sources_str = source_rows
            .iter()
            .map(|(name, count, _)| format!("{} ({})", name, count))
            .collect::<Vec<_>>()
            .join(", ");
        println!("  {:20} {}", "Sources:".dimmed(), sources_str);
    }

    if !model_rows.is_empty() {
        let models_str = model_rows
            .iter()
            .take(3)
            .map(|(name, count, _)| format!("{} ({})", name, count))
            .collect::<Vec<_>>()
            .join(", ");
        println!("  {:20} {}", "Top models:".dimmed(), models_str);
    }

    if !project_rows.is_empty() {
        let proj_str: Vec<_> = project_rows
            .iter()
            .filter(|(name, _, _)| !name.is_empty())
            .take(3)
            .map(|(name, count, _)| format!("{} ({})", name, count))
            .collect();
        if !proj_str.is_empty() {
            println!("  {:20} {}", "Top projects:".dimmed(), proj_str.join(", "));
        }
    }

    println!("{}", "─".repeat(50).dimmed());
    if stats.has_unreliable {
        println!(
            "{}",
            "  Note: ~ = Claude Code token/cost data is unreliable (Anthropic bug #22686)"
                .yellow()
                .dimmed()
        );
    }
    println!();

    Ok(())
}

fn print_by_day(conn: &Connection, opts: &StatsOptions) -> Result<(), Box<dyn std::error::Error>> {
    let where_clause = build_where_clause(opts);
    let sql = format!(
        "SELECT
             LEFT(timestamp, 10)  AS day,
             COUNT(*)             AS runs,
             SUM(input_tokens)    AS tokens_in,
             SUM(output_tokens)   AS tokens_out,
             SUM(cost_usd)        AS cost,
             SUM(CASE WHEN cost_estimated THEN 1 ELSE 0 END) > 0 AS any_estimated,
             -- Top model by run count for the day
             (SELECT model FROM runs r2
              WHERE LEFT(r2.timestamp, 10) = LEFT(r.timestamp, 10)
              GROUP BY model ORDER BY COUNT(*) DESC LIMIT 1) AS top_model
         FROM runs r
         {}
         GROUP BY day
         ORDER BY day DESC
         LIMIT 60",
        where_clause
    );

    if opts.json {
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        let mut data: Vec<serde_json::Value> = Vec::new();
        while let Some(row) = rows.next()? {
            data.push(serde_json::json!({
                "date": row.get::<_, String>(0)?,
                "runs": row.get::<_, i64>(1)?,
                "tokens_in": row.get::<_, i64>(2)?,
                "tokens_out": row.get::<_, i64>(3)?,
                "cost_usd": row.get::<_, f64>(4)?,
                "cost_estimated": row.get::<_, bool>(5)?,
                "top_model": row.get::<_, Option<String>>(6)?,
            }));
        }
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    println!();
    println!(
        "{:<12} {:>6} {:>12} {:>12} {:>12} {}",
        "Date".bold(),
        "Runs".bold(),
        "Tokens In".bold(),
        "Tokens Out".bold(),
        "Cost (est.)".bold(),
        "Top Model".bold(),
    );
    println!("{}", "─".repeat(80).dimmed());

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let day: String = row.get(0)?;
        let runs: i64 = row.get(1)?;
        let tokens_in: i64 = row.get(2)?;
        let tokens_out: i64 = row.get(3)?;
        let cost: f64 = row.get(4)?;
        let any_estimated: bool = row.get(5)?;
        let top_model: Option<String> = row.get(6)?;

        let cost_str = if cost == 0.0 {
            "—".to_string()
        } else if any_estimated {
            format!("~${:.2}", cost)
        } else {
            format!("${:.2}", cost)
        };

        println!(
            "{:<12} {:>6} {:>12} {:>12} {:>12} {}",
            day,
            runs,
            fmt_tokens_m(tokens_in),
            fmt_tokens_m(tokens_out),
            cost_str,
            top_model.as_deref().unwrap_or("—"),
        );
    }
    println!("{}", "─".repeat(80).dimmed());
    println!();

    Ok(())
}

fn print_by_group(
    conn: &Connection,
    opts: &StatsOptions,
    group: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let where_clause = build_where_clause(opts);
    let rows = query_by(conn, &where_clause, group, 50)?;

    if opts.json {
        let data: Vec<serde_json::Value> = rows
            .iter()
            .map(|(name, count, cost)| {
                serde_json::json!({"name": name, "runs": count, "cost_usd": cost})
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    println!();
    println!(
        "{:<40} {:>8} {:>12}",
        group.to_uppercase().bold(),
        "Runs".bold(),
        "Cost (est.)".bold()
    );
    println!("{}", "─".repeat(62).dimmed());

    for (name, count, cost) in &rows {
        let display_name = if name.is_empty() { "(none)" } else { name.as_str() };
        let cost_str = if *cost == 0.0 {
            "—".to_string()
        } else {
            format!("${:.4}", cost)
        };
        println!("{:<40} {:>8} {:>12}", display_name, count, cost_str);
    }
    println!("{}", "─".repeat(62).dimmed());
    println!();

    Ok(())
}

fn query_by(
    conn: &Connection,
    where_clause: &str,
    group: &str,
    limit: usize,
) -> Result<Vec<(String, i64, f64)>, Box<dyn std::error::Error>> {
    let sql = format!(
        "SELECT COALESCE({}, '') AS grp, COUNT(*) AS cnt, SUM(cost_usd) AS cost
         FROM runs
         {}
         GROUP BY grp
         ORDER BY cnt DESC
         LIMIT {}",
        group, where_clause, limit
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        let name: String = row.get(0)?;
        let count: i64 = row.get(1)?;
        let cost: f64 = row.get(2).unwrap_or(0.0);
        result.push((name, count, cost));
    }
    Ok(result)
}

fn period_desc(opts: &StatsOptions) -> String {
    match (&opts.since, &opts.until) {
        (Some(s), Some(u)) => format!("{} to {}", s, u),
        (Some(s), None) => format!("since {}", s),
        (None, Some(u)) => format!("until {}", u),
        (None, None) => "last 30 days".to_string(),
    }
}

fn parse_date(s: &str) -> Option<String> {
    use chrono::{Duration, Utc};

    if let Some(days_str) = s.strip_suffix('d') {
        if let Ok(days) = days_str.parse::<i64>() {
            let dt = Utc::now() - Duration::days(days);
            return Some(dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());
        }
    }
    if s.len() == 10 {
        return Some(format!("{}T00:00:00Z", s));
    }
    None
}

/// Parse an until date — use the start of the *next* day for strict `<` comparison.
fn parse_date_end(s: &str) -> Option<String> {
    use chrono::{Duration, NaiveDate, TimeZone, Utc};

    if s.len() == 10 {
        if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            let next = d + Duration::days(1);
            let dt = Utc.from_utc_datetime(&next.and_hms_opt(0, 0, 0)?);
            return Some(dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());
        }
    }
    None
}

fn fmt_tokens_m(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn fmt_count(n: i64) -> String {
    // Add thousands separator
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{init_db, insert_run};
    use crate::ingest::RunRecord;
    use tempfile::TempDir;

    fn make_run(id: &str, source: &str, model: &str, ts: &str, project: &str) -> RunRecord {
        RunRecord {
            id: id.to_string(),
            session_id: None,
            source: source.to_string(),
            model: Some(model.to_string()),
            timestamp: ts.to_string(),
            role: Some("assistant".to_string()),
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            tokens_estimated: false,
            data_quality: "reliable".to_string(),
            cost_usd: 0.05,
            cost_estimated: false,
            duration_ms: Some(2000),
            project: if project.is_empty() { None } else { Some(project.to_string()) },
            git_branch: None,
            working_dir: None,
            command: None,
            source_path: None,
        }
    }

    #[test]
    fn test_summary_stats() {
        let dir = TempDir::new().unwrap();
        let conn = init_db(dir.path()).unwrap();

        insert_run(&conn, &make_run("r1", "opencode", "claude-sonnet-4", "2026-03-25T10:00:00Z", "myapp")).unwrap();
        insert_run(&conn, &make_run("r2", "praxis", "qwen3.5:9B", "2026-03-25T11:00:00Z", "praxis")).unwrap();

        let opts = StatsOptions {
            by: None,
            since: Some("2026-03-25".to_string()),
            until: Some("2026-03-26".to_string()),
            json: false,
        };
        let stats = query_summary(&conn, &opts).unwrap();
        assert_eq!(stats.total_runs, 2);
        assert!(stats.total_input > 0);
    }

    #[test]
    fn test_by_source_grouping() {
        let dir = TempDir::new().unwrap();
        let conn = init_db(dir.path()).unwrap();

        insert_run(&conn, &make_run("r1", "opencode", "claude-sonnet-4", "2026-03-25T10:00:00Z", "")).unwrap();
        insert_run(&conn, &make_run("r2", "opencode", "claude-sonnet-4", "2026-03-25T11:00:00Z", "")).unwrap();
        insert_run(&conn, &make_run("r3", "praxis", "qwen3.5:9B", "2026-03-25T12:00:00Z", "")).unwrap();

        let where_clause = "WHERE timestamp >= '2026-03-25T00:00:00Z'";
        let rows = query_by(&conn, where_clause, "source", 10).unwrap();
        assert_eq!(rows.len(), 2);
        // opencode should be first (2 runs)
        assert_eq!(rows[0].0, "opencode");
        assert_eq!(rows[0].1, 2);
    }

    #[test]
    fn test_fmt_count() {
        assert_eq!(fmt_count(0), "0");
        assert_eq!(fmt_count(1247), "1,247");
        assert_eq!(fmt_count(1_000_000), "1,000,000");
    }

    #[test]
    fn test_parse_date() {
        assert!(parse_date("7d").is_some());
        assert_eq!(parse_date("2026-03-20").unwrap(), "2026-03-20T00:00:00Z");
        assert!(parse_date("invalid").is_none());
    }

    #[test]
    fn test_parse_date_end() {
        let result = parse_date_end("2026-03-25").unwrap();
        assert!(result.starts_with("2026-03-26"));
    }

    #[test]
    fn test_build_where_clause_default() {
        let opts = StatsOptions { by: None, since: None, until: None, json: false };
        let clause = build_where_clause(&opts);
        assert!(clause.contains("WHERE"));
        assert!(clause.contains("timestamp >="));
    }
}
