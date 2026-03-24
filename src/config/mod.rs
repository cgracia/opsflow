use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PraxisConfig {
    pub llm_api_base: String,
    pub llm_model: String,
    pub llm_api_key: Option<String>,
    pub praxis_dir: PathBuf,
}

impl Default for PraxisConfig {
    fn default() -> Self {
        PraxisConfig {
            llm_api_base: "http://localhost:11434/v1".to_string(),
            llm_model: "llama3".to_string(),
            llm_api_key: None,
            praxis_dir: default_praxis_dir(),
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

    config
}
