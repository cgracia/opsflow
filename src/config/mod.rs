use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PraxisConfig {
    pub llm_api_base: String,
    pub llm_model: String,
    pub llm_api_key: Option<String>,
    pub llm_timeout_secs: u64,
    pub llm_max_output_tokens: Option<u32>,
    pub praxis_dir: PathBuf,
    pub opencode_dir: Option<PathBuf>,
    pub claude_code_dir: Option<PathBuf>,

    // Phase 3: Todoist integration
    pub todoist_api_key: Option<String>,
    /// Default Todoist project name for pushed tasks.
    pub todoist_project: Option<String>,
    /// Todoist filter expression for pulling tasks into the triage view.
    pub todoist_pull_filter: Option<String>,

    // Phase 3: Triage settings
    /// Role context injected into the triage system prompt.
    pub triage_role_context: Option<String>,
    /// Maximum number of signals to include in a single triage call.
    pub triage_max_signals: Option<usize>,
    /// Max output tokens for triage LLM calls (overrides llm_max_output_tokens).
    pub triage_max_output_tokens: Option<u32>,
}

impl Default for PraxisConfig {
    fn default() -> Self {
        PraxisConfig {
            llm_api_base: "http://localhost:11434/v1".to_string(),
            llm_model: "qwen3.5:9B".to_string(),
            llm_api_key: None,
            llm_timeout_secs: 600,
            llm_max_output_tokens: Some(700),
            praxis_dir: default_praxis_dir(),
            opencode_dir: None,
            claude_code_dir: None,
            todoist_api_key: None,
            todoist_project: None,
            todoist_pull_filter: None,
            triage_role_context: None,
            triage_max_signals: None,
            triage_max_output_tokens: None,
        }
    }
}

impl PraxisConfig {
    /// Resolved OpenCode data directory: config → env → default.
    /// Returns None if neither configured nor the default path exists.
    pub fn resolved_opencode_dir(&self) -> Option<PathBuf> {
        // Env var overrides config
        if let Ok(v) = std::env::var("PRAXIS_OPENCODE_DIR") {
            return Some(PathBuf::from(v));
        }
        if let Some(ref d) = self.opencode_dir {
            return Some(d.clone());
        }
        // Default: ~/.local/share/opencode
        let default = dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local")
            .join("share")
            .join("opencode");
        if default.exists() {
            Some(default)
        } else {
            None
        }
    }

    /// Resolved Claude Code projects directory: config → env → default.
    /// Returns None if neither configured nor the default path exists.
    pub fn resolved_claude_code_dir(&self) -> Option<PathBuf> {
        if let Ok(v) = std::env::var("PRAXIS_CLAUDE_CODE_DIR") {
            return Some(PathBuf::from(v));
        }
        if let Some(ref d) = self.claude_code_dir {
            return Some(d.clone());
        }
        // Default: ~/.claude/projects
        let default = dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("projects");
        if default.exists() {
            Some(default)
        } else {
            None
        }
    }
}

fn default_praxis_dir() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".praxis")
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    llm_api_base: Option<String>,
    llm_model: Option<String>,
    llm_api_key: Option<String>,
    llm_timeout_secs: Option<u64>,
    llm_max_output_tokens: Option<u32>,
    opencode_dir: Option<String>,
    claude_code_dir: Option<String>,
    // Phase 3
    todoist_api_key: Option<String>,
    todoist_project: Option<String>,
    todoist_pull_filter: Option<String>,
    triage_role_context: Option<String>,
    triage_max_signals: Option<usize>,
    triage_max_output_tokens: Option<u32>,
}

pub fn load_config() -> PraxisConfig {
    let mut config = PraxisConfig::default();

    // Override praxis_dir from env first (needed to find config file)
    if let Ok(dir) = std::env::var("PRAXIS_DIR") {
        config.praxis_dir = PathBuf::from(dir);
    }

    // Load config file
    let config_path = config.praxis_dir.join("config.toml");
    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(contents) => {
                match toml::from_str::<ConfigFile>(&contents) {
                    Ok(file_config) => {
                        if let Some(v) = file_config.llm_api_base {
                            config.llm_api_base = v;
                        }
                        if let Some(v) = file_config.llm_model {
                            config.llm_model = v;
                        }
                        if let Some(v) = file_config.llm_api_key {
                            config.llm_api_key = Some(v);
                        }
                        if let Some(v) = file_config.llm_timeout_secs {
                            config.llm_timeout_secs = v;
                        }
                        if let Some(v) = file_config.llm_max_output_tokens {
                            config.llm_max_output_tokens = Some(v);
                        }
                        if let Some(v) = file_config.opencode_dir {
                            config.opencode_dir = Some(PathBuf::from(v));
                        }
                        if let Some(v) = file_config.claude_code_dir {
                            config.claude_code_dir = Some(PathBuf::from(v));
                        }
                        if let Some(v) = file_config.todoist_api_key {
                            config.todoist_api_key = Some(v);
                        }
                        if let Some(v) = file_config.todoist_project {
                            config.todoist_project = Some(v);
                        }
                        if let Some(v) = file_config.todoist_pull_filter {
                            config.todoist_pull_filter = Some(v);
                        }
                        if let Some(v) = file_config.triage_role_context {
                            config.triage_role_context = Some(v);
                        }
                        if let Some(v) = file_config.triage_max_signals {
                            config.triage_max_signals = Some(v);
                        }
                        if let Some(v) = file_config.triage_max_output_tokens {
                            config.triage_max_output_tokens = Some(v);
                        }
                    }
                    Err(e) => {
                        eprintln!("warning: failed to parse config file {}: {}", config_path.display(), e);
                    }
                }
            }
            Err(e) => {
                eprintln!("warning: failed to read config file {}: {}", config_path.display(), e);
            }
        }
    }

    // Env var overrides (highest priority)
    if let Ok(v) = std::env::var("PRAXIS_LLM_API_BASE") {
        config.llm_api_base = v;
    }
    if let Ok(v) = std::env::var("PRAXIS_LLM_MODEL") {
        config.llm_model = v;
    }
    if let Ok(v) = std::env::var("PRAXIS_LLM_API_KEY") {
        config.llm_api_key = Some(v);
    }
    if let Ok(v) = std::env::var("PRAXIS_LLM_TIMEOUT_SECS") {
        match v.parse::<u64>() {
            Ok(parsed) => config.llm_timeout_secs = parsed,
            Err(_) => eprintln!("warning: invalid PRAXIS_LLM_TIMEOUT_SECS value {:?}; expected integer seconds", v),
        }
    }
    if let Ok(v) = std::env::var("PRAXIS_LLM_MAX_OUTPUT_TOKENS") {
        match v.parse::<u32>() {
            Ok(parsed) => config.llm_max_output_tokens = Some(parsed),
            Err(_) => eprintln!(
                "warning: invalid PRAXIS_LLM_MAX_OUTPUT_TOKENS value {:?}; expected integer token count",
                v
            ),
        }
    }
    // Note: PRAXIS_OPENCODE_DIR and PRAXIS_CLAUDE_CODE_DIR are resolved lazily
    // via resolved_opencode_dir() / resolved_claude_code_dir() to keep load_config simple.

    // Phase 3 env overrides
    if let Ok(v) = std::env::var("PRAXIS_TODOIST_API_KEY") {
        config.todoist_api_key = Some(v);
    }

    config
}
