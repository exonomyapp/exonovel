use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use comrak::{markdown_to_html, Options};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocHeading {
    pub id: String,
    pub text: String,
    pub level: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocNode {
    pub id: String,
    pub name: String,
    pub title: String,
    pub path: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub children: Vec<DocNode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocPayload {
    pub path: String,
    pub name: String,
    pub title: String,
    pub html: String,
    pub word_count: usize,
    pub reading_minutes: usize,
    pub headings: Vec<DocHeading>,
}

fn slugify_heading(text: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in text.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
        } else if (ch.is_whitespace() || ch == '-') && !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

pub fn extract_title_and_headings(markdown: &str, fallback_name: &str) -> (String, Vec<DocHeading>, usize) {
    let mut title = fallback_name.trim_end_matches(".md").to_string();
    let mut found_h1 = false;
    let mut in_fence = false;
    let mut headings = Vec::new();
    let mut counts: HashMap<String, usize> = HashMap::new();

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        let hashes = trimmed.bytes().take_while(|b| *b == b'#').count();
        if (1..=4).contains(&hashes) && trimmed.as_bytes().get(hashes) == Some(&b' ') {
            let raw = trimmed[hashes..].trim().trim_end_matches('#').trim();
            let clean = raw.replace(['*', '_', '`'], "");
            if clean.is_empty() {
                continue;
            }
            if hashes == 1 && !found_h1 {
                title = clean.clone();
                found_h1 = true;
            }
            if hashes == 2 || hashes == 3 {
                let base = slugify_heading(&clean);
                let count = counts.entry(base.clone()).or_insert(0);
                let id = if *count == 0 {
                    base.clone()
                } else {
                    format!("{base}-{}", *count)
                };
                *count += 1;
                headings.push(DocHeading {
                    id,
                    text: clean,
                    level: hashes as u8,
                });
            }
        }
    }

    let word_count = markdown.split_whitespace().count();
    (title, headings, word_count)
}

pub fn render_safe_gfm_html(markdown: &str) -> String {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.footnotes = true;
    options.extension.header_ids = Some(String::new());
    options.render.unsafe_ = true;

    let raw_html = markdown_to_html(markdown, &options);

    let mut cleaner = ammonia::Builder::default();
    let extra_tags: HashSet<&str> = ["input"].into_iter().collect();
    cleaner.add_tags(extra_tags);
    let input_attrs: HashSet<&str> = ["type", "checked", "disabled"].into_iter().collect();
    cleaner.add_tag_attributes("input", input_attrs);
    for tag in ["h1", "h2", "h3", "h4", "a"] {
        let attrs: HashSet<&str> = ["id"].into_iter().collect();
        cleaner.add_tag_attributes(tag, attrs);
    }

    cleaner.clean(&raw_html).to_string()
}

fn format_folder_label(name: &str) -> String {
    if name.eq_ignore_ascii_case("adr") {
        return "ADRs".to_string();
    }
    let parts: Vec<&str> = name.split('-').collect();
    if parts.len() > 1 && parts[0].chars().all(|c| c.is_ascii_digit()) {
        let rest = parts[1..]
            .iter()
            .map(|p| title_case(p))
            .collect::<Vec<_>>()
            .join(" ");
        return format!("{} {rest}", parts[0]);
    }
    name.split(['-', '_'])
        .map(title_case)
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_case(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn sort_nodes(nodes: &mut [DocNode]) {
    nodes.sort_by(|a, b| {
        let stem_a = a.name.trim_end_matches(".md");
        let stem_b = b.name.trim_end_matches(".md");
        match stem_a.cmp(stem_b) {
            Ordering::Equal => {
                if a.node_type == b.node_type {
                    Ordering::Equal
                } else if a.node_type == "file" {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            other => other,
        }
    });
}

pub fn normalize_doc_locale(raw: &str) -> &'static str {
    let trimmed = raw.trim().to_ascii_lowercase();
    if trimmed == "de" || trimmed.starts_with("de-") {
        "de"
    } else if trimmed == "pl" || trimmed.starts_with("pl-") {
        "pl"
    } else {
        "en"
    }
}

fn localized_title_override(root_dir: &Path, rel_path: &str, locale: &str, fallback_name: &str) -> Option<String> {
    if locale == "en" {
        return None;
    }
    let sub = if rel_path == "AGENTS.md" {
        "AGENTS.md"
    } else if let Some(rest) = rel_path.strip_prefix("docs/") {
        rest
    } else {
        return None;
    };
    let loc_path = root_dir.join("docs").join("i18n").join(locale).join(sub);
    if !loc_path.is_file() {
        return None;
    }
    let content = fs::read_to_string(&loc_path).ok()?;
    let (title, _, _) = extract_title_and_headings(&content, fallback_name);
    Some(title)
}

fn walk_dir(root_dir: &Path, abs_dir: &Path, rel_dir: &str, locale: &str) -> Vec<DocNode> {
    let Ok(entries) = fs::read_dir(abs_dir) else {
        return Vec::new();
    };
    let mut collected = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "i18n" {
            continue;
        }
        let entry_path = entry.path();
        let rel_path = if rel_dir.is_empty() {
            name.clone()
        } else {
            format!("{rel_dir}/{name}")
        };
        if entry_path.is_dir() {
            let children = walk_dir(root_dir, &entry_path, &rel_path, locale);
            if !children.is_empty() {
                collected.push(DocNode {
                    id: rel_path.clone(),
                    name: name.clone(),
                    title: format_folder_label(&name),
                    path: rel_path,
                    node_type: "folder".to_string(),
                    children,
                });
            }
        } else if entry_path.is_file() && name.to_lowercase().ends_with(".md") {
            let title = localized_title_override(root_dir, &rel_path, locale, &name)
                .unwrap_or_else(|| {
                    let content = fs::read_to_string(&entry_path).unwrap_or_default();
                    let (t, _, _) = extract_title_and_headings(&content, &name);
                    t
                });
            collected.push(DocNode {
                id: rel_path.clone(),
                name,
                title,
                path: rel_path,
                node_type: "file".to_string(),
                children: Vec::new(),
            });
        }
    }
    sort_nodes(&mut collected);
    collected
}

pub fn get_docs_tree(root_dir: &Path) -> Vec<DocNode> {
    get_docs_tree_for_locale(root_dir, "en")
}

pub fn get_docs_tree_for_locale(root_dir: &Path, locale: &str) -> Vec<DocNode> {
    let norm_locale = normalize_doc_locale(locale);
    let mut nodes = Vec::new();
    let docs_dir = root_dir.join("docs");
    if docs_dir.is_dir() {
        nodes.extend(walk_dir(root_dir, &docs_dir, "docs", norm_locale));
    }
    let agents_file = root_dir.join("AGENTS.md");
    if agents_file.is_file() {
        let title = localized_title_override(root_dir, "AGENTS.md", norm_locale, "AGENTS.md")
            .unwrap_or_else(|| {
                let content = fs::read_to_string(&agents_file).unwrap_or_default();
                let (t, _, _) = extract_title_and_headings(&content, "AGENTS.md");
                t
            });
        nodes.push(DocNode {
            id: "AGENTS.md".to_string(),
            name: "AGENTS.md".to_string(),
            title,
            path: "AGENTS.md".to_string(),
            node_type: "file".to_string(),
            children: Vec::new(),
        });
    }
    nodes
}

pub fn resolve_safe_doc_path(root_dir: &Path, requested: &str) -> Option<PathBuf> {
    let normalized = requested.trim_start_matches('/').replace('\\', "/");
    if normalized.is_empty() || normalized.contains("..") {
        return None;
    }

    let root_canon = fs::canonicalize(root_dir).ok()?;
    let target = fs::canonicalize(root_dir.join(&normalized)).ok()?;
    let agents = fs::canonicalize(root_dir.join("AGENTS.md")).ok();

    if Some(&target) == agents.as_ref() && target.is_file() {
        return Some(target);
    }

    let docs_canon = fs::canonicalize(root_dir.join("docs")).ok()?;
    if target.starts_with(&docs_canon)
        && target.starts_with(&root_canon)
        && target
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false)
        && target.is_file()
    {
        return Some(target);
    }

    None
}

pub fn resolve_safe_doc_path_for_locale(
    root_dir: &Path,
    requested: &str,
    locale: &str,
) -> Option<(PathBuf, String)> {
    let canonical_abs = resolve_safe_doc_path(root_dir, requested)?;
    let root_canon = fs::canonicalize(root_dir).ok()?;
    let canonical_rel = canonical_abs
        .strip_prefix(&root_canon)
        .unwrap_or(&canonical_abs)
        .to_string_lossy()
        .replace('\\', "/");

    let norm_locale = normalize_doc_locale(locale);
    if norm_locale != "en" {
        let sub = if canonical_rel == "AGENTS.md" {
            Some("AGENTS.md")
        } else {
            canonical_rel.strip_prefix("docs/")
        };
        if let Some(rel_sub) = sub {
            let localized_rel = format!("docs/i18n/{norm_locale}/{rel_sub}");
            if let Some(localized_abs) = resolve_safe_doc_path(root_dir, &localized_rel) {
                return Some((localized_abs, canonical_rel));
            }
        }
    }

    Some((canonical_abs, canonical_rel))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn blocks_path_traversal() {
        let cwd = env::current_dir().unwrap();
        assert!(resolve_safe_doc_path(&cwd, "../Cargo.toml").is_none());
        assert!(resolve_safe_doc_path(&cwd, "Cargo.toml").is_none());
    }

    #[test]
    fn excludes_i18n_folder_from_tree() {
        let cwd = env::current_dir().unwrap();
        let tree = get_docs_tree(&cwd);
        fn contains_i18n(nodes: &[DocNode]) -> bool {
            nodes.iter().any(|n| {
                n.name == "i18n" || n.path.starts_with("docs/i18n") || contains_i18n(&n.children)
            })
        }
        assert!(!contains_i18n(&tree));
    }

    #[test]
    fn renders_gfm_tables_and_sanitizes_scripts() {
        let html = render_safe_gfm_html("| A | B |\n|---|---|\n| 1 | 2 |\n<script>alert(1)</script>");
        assert!(html.contains("<table>"));
        assert!(!html.contains("<script>"));
    }
}
