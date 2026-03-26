use std::time::Instant;

use chrono::Utc;
use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::PraxisConfig;
use crate::llm::{self, LlmRequestOptions};
use crate::registry::WorkflowEntry;
use crate::signals::{get_signals, Signal, SignalsFilter};
use crate::todoist::{TodoistClient, TodoistTask};

/// One triage decision returned by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageItem {
    pub id: String,
    pub kind: String, // "signal" or "task"
    pub priority: String,
    pub reason: String,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct TriageResponse {
    items: Vec<TriageItem>,
}

pub struct TriageResult {
    pub items_triaged: usize,
    pub model: String,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_read_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
    pub duration_ms: u64,
}

/// Run triage on all untriaged signals, optionally including Todoist tasks for context.
pub fn run_triage(
    conn: &Connection,
    config: &PraxisConfig,
    workflows: &[WorkflowEntry],
) -> Result<TriageResult, Box<dyn std::error::Error>> {
    let untriaged = get_signals(
        conn,
        &SignalsFilter { untriaged_only: true, workflow: None },
    )?;

    // Fetch Todoist tasks if configured
    let todoist_tasks: Vec<TodoistTask> = if config.todoist_api_key.is_some() {
        match TodoistClient::new(config) {
            Ok(client) => client
                .fetch_tasks(Some(config.todoist_pull_filter.as_deref().unwrap_or("today | overdue")))
                .unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    let cap = config.triage_max_signals.unwrap_or(50);
    let signals_to_triage: Vec<&Signal> = untriaged.iter().take(cap).collect();

    if signals_to_triage.is_empty() && todoist_tasks.is_empty() {
        return Ok(TriageResult {
            items_triaged: 0,
            model: config.llm_model.clone(),
            input_tokens: None,
            output_tokens: None,
            cache_read_tokens: None,
            cache_write_tokens: None,
            duration_ms: 0,
        });
    }

    let system_prompt = build_system_prompt(config);
    let user_prompt = build_user_prompt(&signals_to_triage, &todoist_tasks, workflows);

    // Use a higher token limit for triage responses
    let mut triage_config = config.clone();
    triage_config.llm_max_output_tokens = Some(config.triage_max_output_tokens.unwrap_or(4000));

    let start = Instant::now();
    let response = llm::generate_response(
        &system_prompt,
        &user_prompt,
        &triage_config,
        LlmRequestOptions { stream: false },
        |_| Ok(()),
    )
    .map_err(|e| format!("triage LLM call failed: {}", e))?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let triage_response = parse_triage_response(&response.text)?;

    let now = Utc::now().to_rfc3339();
    let mut items_triaged = 0;

    for item in &triage_response.items {
        if item.kind == "signal" {
            conn.execute(
                "UPDATE signals SET
                     priority = ?,
                     triage_reason = ?,
                     suggested_action = ?,
                     triaged_at = ?
                 WHERE id = ?",
                params![
                    item.priority,
                    item.reason,
                    item.suggested_action,
                    now,
                    item.id
                ],
            )?;
            items_triaged += 1;
        }
    }

    // Log triage run for observability
    let cost_usd = estimate_triage_cost(
        response.input_tokens,
        response.output_tokens,
        response.cache_read_tokens,
        &response.model,
    );
    conn.execute(
        "INSERT INTO triage_runs
             (id, timestamp, model, signals_triaged, tokens_input, tokens_output,
              cache_read_tokens, cache_write_tokens, cost_usd, duration_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            Uuid::new_v4().to_string(),
            now,
            response.model,
            items_triaged as i64,
            response.input_tokens.map(|t| t as i64),
            response.output_tokens.map(|t| t as i64),
            response.cache_read_tokens.map(|t| t as i64),
            response.cache_write_tokens.map(|t| t as i64),
            cost_usd,
            duration_ms as i64
        ],
    )?;

    Ok(TriageResult {
        items_triaged,
        model: response.model,
        input_tokens: response.input_tokens,
        output_tokens: response.output_tokens,
        cache_read_tokens: response.cache_read_tokens,
        cache_write_tokens: response.cache_write_tokens,
        duration_ms,
    })
}

fn build_system_prompt(config: &PraxisConfig) -> String {
    let role = config
        .triage_role_context
        .as_deref()
        .unwrap_or("A technical operations leader managing a team and key business workflows.");
    format!(
        "You are a triage assistant for a technical operations leader.\n\
         Role context: {}\n\n\
         Given the following signals (workflow outputs) and tasks, classify each item into \
         exactly one priority bucket:\n\n\
         - ACT_NOW: Requires immediate action. Customer escalations, SLA breaches, blocking issues.\n\
         - REVIEW_TODAY: Should be reviewed before end of day. New reports, pending decisions, due tasks.\n\
         - REVIEW_WEEK: Can wait until weekly review. Trends, non-urgent updates, informational.\n\
         - BACKGROUND: Awareness only. Normal operations, routine outputs.\n\
         - IGNORE: Noise. No action needed.\n\n\
         Respond with ONLY valid JSON in this exact structure — no markdown, no explanation:\n\
         {{\"items\": [\
         {{\"id\": \"...\", \"kind\": \"signal\", \"priority\": \"ACT_NOW\", \
         \"reason\": \"one sentence\", \"suggested_action\": \"one sentence\"}}\
         ]}}",
        role
    )
}

fn build_user_prompt(
    signals: &[&Signal],
    tasks: &[TodoistTask],
    workflows: &[WorkflowEntry],
) -> String {
    let mut out = String::new();

    if !signals.is_empty() {
        out.push_str("## Workflow Signals\n\n");
        for signal in signals {
            let wf_desc = workflows
                .iter()
                .find(|w| w.name == signal.workflow_name)
                .map(|w| w.workflow.description.as_str())
                .unwrap_or("unknown workflow");
            out.push_str(&format!("### Signal: {}\n", signal.id));
            out.push_str(&format!(
                "- Workflow: {} — {}\n",
                signal.workflow_name, wf_desc
            ));
            out.push_str(&format!("- File modified: {}\n", signal.file_modified));
            if let Some(preview) = &signal.content_preview {
                out.push_str(&format!("- Content:\n```\n{}\n```\n\n", preview));
            } else {
                out.push('\n');
            }
        }
    }

    if !tasks.is_empty() {
        out.push_str("## Todoist Tasks\n\n");
        for task in tasks {
            out.push_str(&format!("### Task: {}\n", task.id));
            out.push_str(&format!("- Content: {}\n", task.content));
            if let Some(due) = &task.due {
                out.push_str(&format!("- Due: {}\n", due.date));
            }
            out.push_str(&format!("- Priority: {}\n\n", task.priority));
        }
    }

    out.push_str("\nClassify each item above. Return JSON only.");
    out
}

fn parse_triage_response(text: &str) -> Result<TriageResponse, Box<dyn std::error::Error>> {
    let json_str = extract_json(text);
    serde_json::from_str::<TriageResponse>(json_str)
        .map_err(|e| format!("failed to parse triage JSON: {}\nRaw response:\n{}", e, text).into())
}

fn extract_json(text: &str) -> &str {
    let text = text.trim();
    // Strip ```json ... ``` or ``` ... ```
    if let Some(stripped) = text.strip_prefix("```json") {
        if let Some(end) = stripped.rfind("```") {
            return stripped[..end].trim();
        }
    }
    if let Some(stripped) = text.strip_prefix("```") {
        if let Some(end) = stripped.rfind("```") {
            return stripped[..end].trim();
        }
    }
    // Fall back to first { ... } span
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return &text[start..=end];
            }
        }
    }
    text
}

/// Very rough cost estimate based on model name prefix.
/// Cache read tokens are billed at ~10% of the input rate; cache write tokens at ~125%.
fn estimate_triage_cost(
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cache_read_tokens: Option<u64>,
    model: &str,
) -> f64 {
    let in_tok = input_tokens.unwrap_or(0) as f64;
    let out_tok = output_tokens.unwrap_or(0) as f64;
    let cache_read = cache_read_tokens.unwrap_or(0) as f64;
    let m = model.to_lowercase();
    // Approximate rates (USD per 1M tokens)
    let (in_rate, out_rate) = if m.contains("opus") {
        (15.0, 75.0)
    } else if m.contains("sonnet") {
        (3.0, 15.0)
    } else if m.contains("haiku") {
        (0.25, 1.25)
    } else {
        (1.0, 3.0) // local/unknown
    };
    let cache_read_rate = in_rate * 0.1;
    (in_tok * in_rate + out_tok * out_rate + cache_read * cache_read_rate) / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_plain() {
        let json = r#"{"items": []}"#;
        assert_eq!(extract_json(json), json);
    }

    #[test]
    fn test_extract_json_markdown_fence() {
        let text = "Here is the result:\n```json\n{\"items\": []}\n```\n";
        let extracted = extract_json(text);
        assert_eq!(extracted, "{\"items\": []}");
    }

    #[test]
    fn test_extract_json_plain_fence() {
        let text = "```\n{\"items\": []}\n```";
        let extracted = extract_json(text);
        assert_eq!(extracted, "{\"items\": []}");
    }

    #[test]
    fn test_parse_triage_response_valid() {
        let json = r#"
        {
          "items": [
            {
              "id": "sig-123",
              "kind": "signal",
              "priority": "ACT_NOW",
              "reason": "P1 tickets unresolved.",
              "suggested_action": "Escalate immediately."
            }
          ]
        }
        "#;
        let result = parse_triage_response(json).unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].priority, "ACT_NOW");
        assert_eq!(result.items[0].id, "sig-123");
    }

    #[test]
    fn test_parse_triage_response_fixture() {
        let fixture = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/triage/response.json"),
        )
        .unwrap();
        let result = parse_triage_response(&fixture).unwrap();
        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[0].priority, "ACT_NOW");
        assert_eq!(result.items[1].priority, "REVIEW_TODAY");
    }

    #[test]
    fn test_estimate_triage_cost_zero_tokens() {
        let cost = estimate_triage_cost(None, None, None, "claude-sonnet-4");
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_estimate_triage_cost_sonnet() {
        // 1000 input + 500 output with sonnet rates
        let cost = estimate_triage_cost(Some(1000), Some(500), None, "claude-sonnet-4");
        assert!(cost > 0.0);
        assert!(cost < 0.1); // sanity check
    }

    #[test]
    fn test_estimate_triage_cost_cache_read_cheaper() {
        // Cache read tokens should cost less than regular input tokens
        let cost_no_cache = estimate_triage_cost(Some(1000), Some(0), None, "claude-sonnet-4");
        let cost_with_cache = estimate_triage_cost(Some(0), Some(0), Some(1000), "claude-sonnet-4");
        assert!(cost_with_cache < cost_no_cache);
    }
}
