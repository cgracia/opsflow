use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::config::PraxisConfig;

pub struct LlmResponse {
    pub text: String,
    pub model: String,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
    model: Option<String>,
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: String,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
}

pub fn generate_response(
    system_prompt: &str,
    user_prompt: &str,
    config: &PraxisConfig,
) -> Result<LlmResponse, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(config.llm_timeout_secs))
        .build()
        .map_err(|e| format!("error: Failed to build HTTP client: {}", e))?;

    let request_body = ChatRequest {
        model: config.llm_model.clone(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ],
        temperature: 0.3,
        max_tokens: config.llm_max_output_tokens,
    };

    let url = format!("{}/chat/completions", config.llm_api_base.trim_end_matches('/'));

    let mut req = client.post(&url).json(&request_body);

    if let Some(api_key) = &config.llm_api_key {
        req = req.bearer_auth(api_key);
    }

    let response = req.send().map_err(|e| {
        if e.is_timeout() {
            format!(
                "error: LLM request timed out after {}s while calling {} with model {}\nHint: local models can take longer on first-token and full completion latency. Try increasing PRAXIS_LLM_TIMEOUT_SECS or lowering PRAXIS_LLM_MAX_OUTPUT_TOKENS.\nHint: to benchmark the raw model outside praxis, run: ollama run {} {:?}",
                config.llm_timeout_secs,
                config.llm_api_base,
                config.llm_model,
                config.llm_model,
                user_prompt
            )
        } else if e.is_connect() {
            format!(
                "error: Failed to connect to LLM at {}: {}\nHint: Is your local model running? (e.g., `ollama serve`)",
                config.llm_api_base, e
            )
        } else {
            format!("error: LLM request failed: {}", e)
        }
    })?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        return Err(format!("error: LLM API returned status {}: {}", status, body));
    }

    let chat_response: ChatResponse = response
        .json()
        .map_err(|e| format!("error: Failed to parse LLM response: {}", e))?;

    let text = chat_response
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| "error: LLM returned no choices".to_string())?;

    let (input_tokens, output_tokens) = if let Some(usage) = chat_response.usage {
        (usage.prompt_tokens, usage.completion_tokens)
    } else {
        (None, None)
    };

    let model = chat_response.model.unwrap_or_else(|| config.llm_model.clone());

    Ok(LlmResponse {
        text,
        model,
        input_tokens,
        output_tokens,
    })
}
