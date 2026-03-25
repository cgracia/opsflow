use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_FILES: usize = 10;
const MAX_FILE_BYTES: usize = 24_000;
const MAX_TOTAL_BYTES: usize = 12_000;
const MAX_SNIPPET_BYTES: usize = 1_800;

#[derive(Debug, Clone)]
pub struct ContextBundle {
    pub prompt_text: String,
    pub artifact_markdown: String,
}

#[derive(Debug, Clone)]
struct ContextCandidate {
    relative_path: PathBuf,
    contents: String,
}

#[derive(Debug, Clone)]
struct SelectedContext {
    relative_path: PathBuf,
    snippet: String,
    score: usize,
}

pub fn load_repo_context(root: &Path, query: &str) -> Result<ContextBundle, String> {
    let canonical_root = root
        .canonicalize()
        .map_err(|e| format!("error: Failed to resolve repo context root {}: {}", root.display(), e))?;

    let mut candidates = Vec::new();
    collect_candidates(&canonical_root, &canonical_root, &mut candidates)?;

    if candidates.is_empty() {
        return Err(format!(
            "error: No readable project files found under {} after applying context safety filters",
            canonical_root.display()
        ));
    }

    let selected = select_context(candidates, query);
    let prompt_text = build_prompt_text(&canonical_root, &selected);
    let artifact_markdown = build_artifact_markdown(&canonical_root, &selected);

    Ok(ContextBundle {
        prompt_text,
        artifact_markdown,
    })
}

fn collect_candidates(root: &Path, dir: &Path, out: &mut Vec<ContextCandidate>) -> Result<(), String> {
    for entry in fs::read_dir(dir)
        .map_err(|e| format!("error: Failed to read directory {}: {}", dir.display(), e))?
    {
        let entry = entry.map_err(|e| format!("error: Failed to inspect directory entry: {}", e))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|e| format!("error: Failed to relativize context path {}: {}", path.display(), e))?;

        if path.is_dir() {
            if should_skip_dir(relative) {
                continue;
            }
            collect_candidates(root, &path, out)?;
            continue;
        }

        if should_skip_file(relative) {
            continue;
        }

        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(_) => continue,
        };

        if contents.is_empty() || contents.len() > MAX_FILE_BYTES {
            continue;
        }

        out.push(ContextCandidate {
            relative_path: relative.to_path_buf(),
            contents,
        });
    }

    Ok(())
}

fn should_skip_dir(relative: &Path) -> bool {
    relative.components().any(|component| {
        matches!(
            component.as_os_str().to_str(),
            Some(
                ".git"
                    | ".direnv"
                    | ".devenv"
                    | ".venv"
                    | "venv"
                    | "__pycache__"
                    | "node_modules"
                    | "target"
                    | "dist"
                    | "build"
                    | "result"
                    | ".next"
            )
        )
    })
}

fn should_skip_file(relative: &Path) -> bool {
    let Some(name) = relative.file_name().and_then(|name| name.to_str()) else {
        return true;
    };

    if name.starts_with(".env")
        || name.ends_with(".pem")
        || name.ends_with(".key")
        || name.ends_with(".crt")
        || name.ends_with(".p12")
        || name.ends_with(".pfx")
        || name.contains("secret")
        || name.contains("token")
        || name.contains("credential")
    {
        return true;
    }

    !matches!(
        relative.extension().and_then(|ext| ext.to_str()),
        Some(
            "rs" | "md" | "toml" | "json" | "yaml" | "yml" | "nix" | "py" | "ts" | "tsx" | "js" | "jsx"
                | "go" | "sh"
        )
    )
}

fn select_context(candidates: Vec<ContextCandidate>, query: &str) -> Vec<SelectedContext> {
    let query_terms = extract_query_terms(query);
    let mut scored = candidates
        .into_iter()
        .map(|candidate| {
            let score = score_candidate(&candidate, &query_terms);
            (score, candidate)
        })
        .collect::<Vec<_>>();

    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| sort_key(&a.1.relative_path).cmp(&sort_key(&b.1.relative_path)))
    });

    let mut selected = Vec::new();
    let mut total_bytes = 0;

    for (score, candidate) in scored {
        if selected.len() >= MAX_FILES || total_bytes >= MAX_TOTAL_BYTES {
            break;
        }

        let remaining = MAX_TOTAL_BYTES.saturating_sub(total_bytes);
        if remaining < 200 {
            break;
        }

        let snippet_budget = remaining.min(MAX_SNIPPET_BYTES);
        let snippet = build_snippet(&candidate.contents, &query_terms, snippet_budget);
        if snippet.trim().is_empty() {
            continue;
        }

        total_bytes += snippet.len();
        selected.push(SelectedContext {
            relative_path: candidate.relative_path,
            snippet,
            score,
        });
    }

    selected.sort_by_key(|item| sort_key(&item.relative_path));
    selected
}

fn extract_query_terms(query: &str) -> HashSet<String> {
    query
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter_map(|term| {
            let lowered = term.to_ascii_lowercase();
            if lowered.len() >= 4 {
                Some(lowered)
            } else {
                None
            }
        })
        .collect()
}

fn score_candidate(candidate: &ContextCandidate, query_terms: &HashSet<String>) -> usize {
    let path = candidate.relative_path.display().to_string().to_ascii_lowercase();
    let body = candidate.contents.to_ascii_lowercase();
    let mut score = match candidate.relative_path.file_name().and_then(|name| name.to_str()) {
        Some("AGENTS.md") => 180,
        Some("README.md") => 160,
        Some("Cargo.toml" | "package.json" | "pyproject.toml" | "go.mod" | "flake.nix") => 150,
        _ if path.starts_with("src/") => 100,
        _ => 40,
    };

    for term in query_terms {
        if path.contains(term) {
            score += 60;
        }
        if body.contains(term) {
            score += 20;
        }
    }

    score
}

fn build_snippet(contents: &str, query_terms: &HashSet<String>, budget: usize) -> String {
    let trimmed = contents.trim();
    if trimmed.len() <= budget {
        return trimmed.to_string();
    }

    let lines = trimmed.lines().collect::<Vec<_>>();
    let mut sections = Vec::new();

    for line in &lines {
        if query_terms.iter().any(|term| line.to_ascii_lowercase().contains(term)) {
            sections.push((*line).to_string());
        }
        if sections.join("\n").len() >= budget / 2 {
            break;
        }
    }

    if sections.is_empty() {
        let mut snippet = String::new();
        for line in lines {
            if snippet.len() + line.len() + 1 > budget {
                break;
            }
            snippet.push_str(line);
            snippet.push('\n');
        }
        snippet.truncate(snippet.trim_end().len());
        return snippet;
    }

    let mut snippet = sections.join("\n");
    if snippet.len() > budget {
        snippet.truncate(budget);
    }
    snippet
}

fn sort_key(path: &Path) -> (u8, String) {
    let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
    let path_str = path.display().to_string();

    let priority = match name {
        "AGENTS.md" => 0,
        "README.md" => 1,
        "Cargo.toml" | "package.json" | "pyproject.toml" | "go.mod" | "flake.nix" => 2,
        _ if path_str.starts_with("src/") => 3,
        _ => 4,
    };

    (priority, path_str)
}

fn build_prompt_text(root: &Path, files: &[SelectedContext]) -> String {
    let mut text = format!(
        "Explicit local project context from {}.\nThis is a budgeted subset selected to fit local model limits. If the answer requires facts not present here, say so.\n\n",
        root.display()
    );

    for file in files {
        text.push_str(&format!(
            "[File: {} | relevance score: {}]\n{}\n\n",
            file.relative_path.display(),
            file.score,
            file.snippet
        ));
    }

    text
}

fn build_artifact_markdown(root: &Path, files: &[SelectedContext]) -> String {
    let mut markdown = format!(
        "Repository root: `{}`\n\nBudgeted context selection used for this run.\n\nFiles included:\n",
        root.display()
    );

    for file in files {
        markdown.push_str(&format!(
            "- `{}` (score {}, {} chars)\n",
            file.relative_path.display(),
            file.score,
            file.snippet.len()
        ));
    }

    markdown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_sensitive_env_files() {
        assert!(should_skip_file(Path::new(".env")));
        assert!(should_skip_file(Path::new("secrets/api_token.txt")));
    }

    #[test]
    fn allows_typical_project_files() {
        assert!(!should_skip_file(Path::new("Cargo.toml")));
        assert!(!should_skip_file(Path::new("src/main.rs")));
    }

    #[test]
    fn prioritizes_query_matches() {
        let selected = select_context(
            vec![
                ContextCandidate {
                    relative_path: PathBuf::from("README.md"),
                    contents: "general overview".to_string(),
                },
                ContextCandidate {
                    relative_path: PathBuf::from("src/database.rs"),
                    contents: "database pool postgres sqlite".to_string(),
                },
            ],
            "what database should i use",
        );
        assert_eq!(selected[0].relative_path, PathBuf::from("README.md"));
        assert!(selected.iter().any(|item| item.relative_path == PathBuf::from("src/database.rs")));
    }

    #[test]
    fn snippet_is_budgeted() {
        let snippet = build_snippet(&"a".repeat(5000), &HashSet::new(), 100);
        assert!(snippet.len() <= 100);
    }
}
