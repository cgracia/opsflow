use crate::config::PraxisConfig;
use crate::db::init_db;
use colored::Colorize;
use duckdb::Connection;

pub struct RunsOptions {
    pub source: Option<String>,
    pub model: Option<String>,
    pub project: Option<String>,
    pub since: Option<String>,
    pub limit: usize,
    pub json: bool,
}

pub fn run(config: &PraxisConfig, opts: RunsOptions) -> Result<(), Box<dyn std::error::Error>> {
    let conn = init_db(&config.praxis_dir)?;
    if opts.json {
        print_json(&conn, &opts)
    } else {
        print_table(&conn, &opts)
    }
}

/// Rows returned from the runs query.
struct RunRow {
    timestamp: String,
    source: String,
    model: String,
    input_tokens: i64,
    output_tokens: i64,
    cost_usd: f64,
    cost_estimated: bool,
    project: String,
    session_title: String,
    data_quality: String,
}

fn query_runs(conn: &Connection, opts: &RunsOptions) -> Result<Vec<RunRow>, Box<dyn std::error::Error>> {
    let since_ts = opts.since.as_deref().map(parse_since).unwrap_or_default();

    let mut conditions: Vec<String> = Vec::new();
    if let Some(ref s) = opts.source {
        conditions.push(format!("r.source LIKE '%{}%'", sanitize(s)));
    }
    if let Some(ref m) = opts.model {
        conditions.push(format!("r.model LIKE '%{}%'", sanitize(m)));
    }
    if let Some(ref p) = opts.project {
        conditions.push(format!(
            "(r.project LIKE '%{}%' OR s.title LIKE '%{}%')",
            sanitize(p),
            sanitize(p)
        ));
    }
    if let Some(ref ts) = since_ts {
        conditions.push(format!("r.timestamp >= '{}'", ts));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT
             r.timestamp,
             r.source,
             COALESCE(r.model, 'unknown')          AS model,
             r.input_tokens,
             r.output_tokens,
             r.cost_usd,
             r.cost_estimated,
             COALESCE(r.project, '')               AS project,
             COALESCE(s.title, '')                 AS session_title,
             r.data_quality
         FROM runs r
         LEFT JOIN sessions s ON r.session_id = s.id
         {}
         ORDER BY r.timestamp DESC
         LIMIT {}",
        where_clause, opts.limit
    );

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([])?;

    let mut result: Vec<RunRow> = Vec::new();
    while let Some(row) = rows.next()? {
        result.push(RunRow {
            timestamp: row.get(0)?,
            source: row.get(1)?,
            model: row.get(2)?,
            input_tokens: row.get(3)?,
            output_tokens: row.get(4)?,
            cost_usd: row.get(5)?,
            cost_estimated: row.get(6)?,
            project: row.get(7)?,
            session_title: row.get(8)?,
            data_quality: row.get(9)?,
        });
    }
    Ok(result)
}

fn print_table(conn: &Connection, opts: &RunsOptions) -> Result<(), Box<dyn std::error::Error>> {
    let rows = query_runs(conn, opts)?;

    if rows.is_empty() {
        println!("{}", "No runs found.".dimmed());
        return Ok(());
    }

    // Header
    println!(
        "{:<24} {:<12} {:<30} {:>10} {:>10} {:>10} {:<20} {}",
        "Timestamp".bold(),
        "Source".bold(),
        "Model".bold(),
        "Tokens In".bold(),
        "Tokens Out".bold(),
        "Cost".bold(),
        "Project".bold(),
        "Session".bold(),
    );
    println!("{}", "─".repeat(130).dimmed());

    for r in &rows {
        let ts = format_ts(&r.timestamp);
        let cost = if r.cost_usd == 0.0 {
            "—".dimmed().to_string()
        } else if r.cost_estimated {
            format!("~${:.4}", r.cost_usd).yellow().to_string()
        } else {
            format!("${:.4}", r.cost_usd).green().to_string()
        };

        let model_display = truncate(&r.model, 30);
        let project_display = truncate(&r.project, 20);
        let title_display = truncate(&r.session_title, 30);

        let tokens_in = if r.data_quality.contains("unreliable") {
            format!("~{}", fmt_tokens(r.input_tokens)).yellow().to_string()
        } else {
            fmt_tokens(r.input_tokens)
        };
        let tokens_out = if r.data_quality.contains("unreliable") {
            format!("~{}", fmt_tokens(r.output_tokens)).yellow().to_string()
        } else {
            fmt_tokens(r.output_tokens)
        };

        let source_colored = match r.source.as_str() {
            "opencode" => r.source.cyan().to_string(),
            "claude-code" | "claude_code" => r.source.blue().to_string(),
            "praxis" => r.source.magenta().to_string(),
            _ => r.source.normal().to_string(),
        };

        println!(
            "{:<24} {:<12} {:<30} {:>10} {:>10} {:>10} {:<20} {}",
            ts, source_colored, model_display, tokens_in, tokens_out, cost, project_display, title_display
        );
    }

    println!("{}", "─".repeat(130).dimmed());
    println!(
        "{} {} runs",
        "Showing".dimmed(),
        rows.len().to_string().bold()
    );

    if rows.iter().any(|r| r.data_quality.contains("unreliable")) {
        println!(
            "{}",
            "~ = Claude Code JSONL token counts are unreliable (known Anthropic bug #22686)".yellow().dimmed()
        );
    }

    Ok(())
}

fn print_json(conn: &Connection, opts: &RunsOptions) -> Result<(), Box<dyn std::error::Error>> {
    let rows = query_runs(conn, opts)?;
    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "timestamp": r.timestamp,
                "source": r.source,
                "model": r.model,
                "input_tokens": r.input_tokens,
                "output_tokens": r.output_tokens,
                "cost_usd": r.cost_usd,
                "cost_estimated": r.cost_estimated,
                "project": r.project,
                "session_title": r.session_title,
                "data_quality": r.data_quality,
            })
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&items)?);
    Ok(())
}

/// Parse a "since" string: "7d", "30d", or "YYYY-MM-DD".
/// Returns an ISO 8601 timestamp string.
fn parse_since(s: &str) -> Option<String> {
    use chrono::{Duration, Utc};

    if let Some(days_str) = s.strip_suffix('d') {
        if let Ok(days) = days_str.parse::<i64>() {
            let dt = Utc::now() - Duration::days(days);
            return Some(dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());
        }
    }

    // Try YYYY-MM-DD
    if s.len() == 10 && s.chars().nth(4) == Some('-') {
        return Some(format!("{}T00:00:00Z", s));
    }

    None
}

fn fmt_tokens(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_ts(ts: &str) -> String {
    // Take the first 19 chars: "YYYY-MM-DDTHH:MM:SS"
    if ts.len() >= 19 {
        ts[..19].replace('T', " ")
    } else {
        ts.to_string()
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

/// Sanitize a string for use in a SQL LIKE clause by escaping SQL metacharacters.
fn sanitize(s: &str) -> String {
    s.replace('\'', "''").replace('%', "\\%").replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_since_days() {
        let result = parse_since("7d").unwrap();
        assert!(result.ends_with('Z'));
        assert!(result.contains('T'));
    }

    #[test]
    fn test_parse_since_date() {
        let result = parse_since("2026-03-20").unwrap();
        assert_eq!(result, "2026-03-20T00:00:00Z");
    }

    #[test]
    fn test_parse_since_invalid() {
        assert!(parse_since("invalid").is_none());
        assert!(parse_since("").is_none());
    }

    #[test]
    fn test_fmt_tokens() {
        assert_eq!(fmt_tokens(0), "0");
        assert_eq!(fmt_tokens(500), "500");
        assert_eq!(fmt_tokens(1500), "1.5K");
        assert_eq!(fmt_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world!", 8), "hello w…");
    }

    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize("normal"), "normal");
        assert_eq!(sanitize("it's"), "it''s");
    }

    #[test]
    fn test_runs_query_integration() {
        use crate::db::{init_db, insert_run};
        use crate::ingest::RunRecord;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let conn = init_db(dir.path()).unwrap();

        insert_run(&conn, &RunRecord {
            id: "r1".into(),
            session_id: None,
            source: "opencode".into(),
            model: Some("claude-sonnet-4".into()),
            timestamp: "2026-03-25T10:00:00Z".into(),
            role: Some("assistant".into()),
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            tokens_estimated: false,
            data_quality: "reliable".into(),
            cost_usd: 0.05,
            cost_estimated: false,
            duration_ms: None,
            project: Some("myapp".into()),
            git_branch: None,
            working_dir: None,
            command: None,
            source_path: None,
        }).unwrap();

        let opts = RunsOptions {
            source: None,
            model: None,
            project: None,
            since: None,
            limit: 20,
            json: false,
        };
        let rows = query_runs(&conn, &opts).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].source, "opencode");
        assert_eq!(rows[0].input_tokens, 1000);
    }
}
