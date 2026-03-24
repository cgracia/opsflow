use std::fs;
use std::path::{Path, PathBuf};

const MAX_FILES: usize = 12;
const MAX_FILE_BYTES: usize = 16_000;
const MAX_TOTAL_BYTES: usize = 64_000;

#[derive(Debug, Clone)]
pub struct ContextBundle {
    pub prompt_text: String,
    pub artifact_markdown: String,
}

#[derive(Debug, Clone)]
struct ContextFile {
    relative_path: PathBuf,
    contents: String,
}

pub fn load_repo_context(root: &Path) -> Result<ContextBundle, String> {
    let canonical_root = root
        .canonicalize()
        .map_err(|e| format!("error: Failed to resolve repo context root {}: {}", root.display(), e))?;

    let mut files = Vec::new();
    collect_files(&canonical_root, &canonical_root, &mut files)?;
    files.sort_by_key(|file| sort_key(&file.relative_path));
    files.truncate(MAX_FILES);

    if files.is_empty() {
        return Err(format!(
            "error: No readable project files found under {} after applying context safety filters",
            canonical_root.display()
        ));
    }

    let prompt_text = build_prompt_text(&canonical_root, &files);
    let artifact_markdown = build_artifact_markdown(&canonical_root, &files);

    Ok(ContextBundle {
        prompt_text,
        artifact_markdown,
    })
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<ContextFile>) -> Result<(), String> {
    let mut total_bytes = out.iter().map(|f| f.contents.len()).sum::<usize>();

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
            collect_files(root, &path, out)?;
            total_bytes = out.iter().map(|f| f.contents.len()).sum::<usize>();
            if out.len() >= MAX_FILES || total_bytes >= MAX_TOTAL_BYTES {
                break;
            }
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

        out.push(ContextFile {
            relative_path: relative.to_path_buf(),
            contents,
        });

        total_bytes += out.last().map(|f| f.contents.len()).unwrap_or(0);
        if out.len() >= MAX_FILES || total_bytes >= MAX_TOTAL_BYTES {
            break;
        }
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

fn build_prompt_text(root: &Path, files: &[ContextFile]) -> String {
    let mut text = format!(
        "Explicit local project context from {}.\nOnly use this material as project grounding. If the answer requires facts not present here, state the assumption clearly.\n\n",
        root.display()
    );

    for file in files {
        text.push_str(&format!("[File: {}]\n{}\n\n", file.relative_path.display(), file.contents));
    }

    text
}

fn build_artifact_markdown(root: &Path, files: &[ContextFile]) -> String {
    let mut markdown = format!("Repository root: `{}`\n\nFiles included:\n", root.display());

    for file in files {
        markdown.push_str(&format!("- `{}`\n", file.relative_path.display()));
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
}
