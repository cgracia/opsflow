use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

const PRICING_URL: &str =
    "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";

const CACHE_FILENAME: &str = "pricing-cache.json";
/// Refresh the pricing cache if older than this many seconds (7 days).
const CACHE_TTL_SECS: u64 = 7 * 24 * 3600;

/// Minimal hardcoded fallback pricing for the most common models.
/// Values are USD per token (not per 1K tokens).
fn hardcoded_fallback() -> Vec<ModelPricing> {
    vec![
        ModelPricing {
            input_cost_per_token: Some(0.000_015),
            output_cost_per_token: Some(0.000_075),
            cache_read_input_token_cost: Some(0.000_001_5),
            cache_creation_input_token_cost: Some(0.000_018_75),
            key: "claude-opus-4-20250514".to_string(),
        },
        ModelPricing {
            input_cost_per_token: Some(0.000_003),
            output_cost_per_token: Some(0.000_015),
            cache_read_input_token_cost: Some(0.000_000_3),
            cache_creation_input_token_cost: Some(0.000_003_75),
            key: "claude-sonnet-4-20250514".to_string(),
        },
        ModelPricing {
            input_cost_per_token: Some(0.000_000_8),
            output_cost_per_token: Some(0.000_004),
            cache_read_input_token_cost: Some(0.000_000_08),
            cache_creation_input_token_cost: Some(0.000_001),
            key: "claude-haiku-4-5-20251001".to_string(),
        },
        ModelPricing {
            input_cost_per_token: Some(0.000_005),
            output_cost_per_token: Some(0.000_015),
            cache_read_input_token_cost: None,
            cache_creation_input_token_cost: None,
            key: "gpt-4o".to_string(),
        },
        ModelPricing {
            input_cost_per_token: Some(0.000_000_15),
            output_cost_per_token: Some(0.000_000_6),
            cache_read_input_token_cost: None,
            cache_creation_input_token_cost: None,
            key: "gpt-4o-mini".to_string(),
        },
        ModelPricing {
            input_cost_per_token: Some(0.000_01),
            output_cost_per_token: Some(0.000_03),
            cache_read_input_token_cost: None,
            cache_creation_input_token_cost: None,
            key: "gpt-4-turbo".to_string(),
        },
    ]
}

#[derive(Debug, Clone, Deserialize)]
struct ModelPricing {
    #[serde(skip)]
    key: String,
    input_cost_per_token: Option<f64>,
    output_cost_per_token: Option<f64>,
    cache_read_input_token_cost: Option<f64>,
    cache_creation_input_token_cost: Option<f64>,
}

/// The pricing database — a map of model name → pricing.
pub struct PricingDb {
    /// All entries keyed by their canonical model name.
    models: HashMap<String, ModelPricing>,
}

impl PricingDb {
    /// Construct an empty pricing database with no models — all lookups return (0.0, true).
    /// Intended for use in unit tests that don't need cost calculation.
    #[cfg(test)]
    pub fn empty() -> Self {
        PricingDb {
            models: HashMap::new(),
        }
    }

    /// Load the pricing database from cache or network.
    /// Falls back to the hardcoded table if both fail.
    pub fn load(praxis_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let cache_path = praxis_dir.join(CACHE_FILENAME);
        let mut db = PricingDb {
            models: HashMap::new(),
        };

        // Attempt to use cached file if fresh
        if let Some(raw) = try_read_cache(&cache_path) {
            if let Ok(parsed) = parse_pricing_json(&raw) {
                db.models = parsed;
                return Ok(db);
            }
        }

        // Try fetching from network
        match fetch_pricing_json() {
            Ok(raw) => {
                // Save to cache
                let _ = std::fs::create_dir_all(praxis_dir);
                let _ = std::fs::write(&cache_path, &raw);
                if let Ok(parsed) = parse_pricing_json(&raw) {
                    db.models = parsed;
                    return Ok(db);
                }
            }
            Err(e) => {
                eprintln!("warning: could not fetch pricing database ({}); using fallback", e);
            }
        }

        // Load hardcoded fallback
        for entry in hardcoded_fallback() {
            db.models.insert(entry.key.clone(), entry);
        }
        Ok(db)
    }

    /// Estimate cost for a given model and token counts.
    ///
    /// Returns `(cost_usd, is_estimated)`.
    /// `is_estimated` is true when the model was not found in the pricing DB
    /// or when token counts themselves are marked as estimated upstream.
    pub fn estimate_cost(
        &self,
        model: &str,
        input_tokens: i64,
        output_tokens: i64,
        cache_read: i64,
        cache_write: i64,
    ) -> (f64, bool) {
        match self.find_pricing(model) {
            Some(p) => {
                let cost = p.input_cost_per_token.unwrap_or(0.0) * input_tokens as f64
                    + p.output_cost_per_token.unwrap_or(0.0) * output_tokens as f64
                    + p.cache_read_input_token_cost.unwrap_or(0.0) * cache_read as f64
                    + p.cache_creation_input_token_cost.unwrap_or(0.0) * cache_write as f64;
                (cost, false)
            }
            None => (0.0, true),
        }
    }

    fn find_pricing(&self, model: &str) -> Option<&ModelPricing> {
        // 1. Exact match
        if let Some(p) = self.models.get(model) {
            return Some(p);
        }

        // 2. Strip provider prefix (e.g. "anthropic/claude-sonnet-4" → "claude-sonnet-4")
        let stripped = if let Some(pos) = model.find('/') {
            &model[pos + 1..]
        } else {
            model
        };

        if let Some(p) = self.models.get(stripped) {
            return Some(p);
        }

        // 3. Prefix match — find any key that starts with the given model name (handles version suffixes)
        let key_lower = stripped.to_lowercase();
        let mut best: Option<&ModelPricing> = None;
        for (k, v) in &self.models {
            let k_lower = k.to_lowercase();
            if k_lower.starts_with(&key_lower) || key_lower.starts_with(&k_lower) {
                best = Some(v);
                break;
            }
        }
        best
    }
}

fn try_read_cache(cache_path: &Path) -> Option<String> {
    let metadata = std::fs::metadata(cache_path).ok()?;
    let modified = metadata.modified().ok()?;
    let age = std::time::SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default();
    if age.as_secs() > CACHE_TTL_SECS {
        return None; // Stale
    }
    std::fs::read_to_string(cache_path).ok()
}

fn fetch_pricing_json() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let text = client.get(PRICING_URL).send()?.text()?;
    Ok(text)
}

fn parse_pricing_json(raw: &str) -> Result<HashMap<String, ModelPricing>, Box<dyn std::error::Error>> {
    let raw_map: HashMap<String, serde_json::Value> = serde_json::from_str(raw)?;
    let mut result = HashMap::new();

    for (key, val) in raw_map {
        // Skip non-object entries (e.g. the "sample_spec" key in the LiteLLM file)
        if !val.is_object() {
            continue;
        }
        if let Ok(mut pricing) = serde_json::from_value::<ModelPricing>(val) {
            pricing.key = key.clone();
            result.insert(key, pricing);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn db_from_fallback() -> PricingDb {
        let mut db = PricingDb {
            models: HashMap::new(),
        };
        for entry in hardcoded_fallback() {
            db.models.insert(entry.key.clone(), entry);
        }
        db
    }

    #[test]
    fn test_exact_model_lookup() {
        let db = db_from_fallback();
        let (cost, estimated) = db.estimate_cost("claude-opus-4-20250514", 1000, 500, 0, 0);
        assert!(!estimated);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_provider_prefix_stripped() {
        let db = db_from_fallback();
        let (cost, estimated) = db.estimate_cost("anthropic/claude-opus-4-20250514", 1000, 500, 0, 0);
        assert!(!estimated);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_prefix_fuzzy_match() {
        let db = db_from_fallback();
        // "claude-sonnet-4" should fuzzy-match "claude-sonnet-4-20250514"
        let (cost, estimated) = db.estimate_cost("claude-sonnet-4", 1000, 200, 0, 0);
        assert!(!estimated);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_unknown_model_returns_zero_estimated() {
        let db = db_from_fallback();
        let (cost, estimated) = db.estimate_cost("totally-unknown-model-xyz", 1000, 500, 0, 0);
        assert_eq!(cost, 0.0);
        assert!(estimated);
    }

    #[test]
    fn test_cache_tokens() {
        let db = db_from_fallback();
        // Cost with cache read should be less than without (cache is cheaper)
        let (cost_no_cache, _) = db.estimate_cost("claude-opus-4-20250514", 1000, 0, 0, 0);
        let (cost_with_cache_read, _) = db.estimate_cost("claude-opus-4-20250514", 0, 0, 1000, 0);
        assert!(cost_with_cache_read < cost_no_cache);
    }

    #[test]
    fn test_parse_pricing_json() {
        let json = r#"{"claude-test":{"input_cost_per_token":0.001,"output_cost_per_token":0.002},"sample_spec":"ignored"}"#;
        let result = parse_pricing_json(json).unwrap();
        assert!(result.contains_key("claude-test"));
        assert!(!result.contains_key("sample_spec")); // non-object skipped
    }

    #[test]
    fn test_cache_staleness_logic() {
        let dir = TempDir::new().unwrap();
        let cache_path = dir.path().join("pricing-cache.json");

        // No cache file → None
        assert!(try_read_cache(&cache_path).is_none());

        // Fresh cache → Some
        std::fs::write(&cache_path, r#"{"test":{}}"#).unwrap();
        assert!(try_read_cache(&cache_path).is_some());
    }
}
