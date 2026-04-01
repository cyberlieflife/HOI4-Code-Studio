#![deny(clippy::unwrap_used)]

use crate::country_tags::{load_country_tags, TagEntry, TagLoadResponse};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::RwLock;

/// ：单个标签引用的错误信息。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagValidationError {
    pub line: usize,
    pub message: String,
}

/// ：校验结果，包含是否成功、消息和错误列表。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagValidationResponse {
    pub success: bool,
    pub message: String,
    pub errors: Vec<TagValidationError>,
}

/// ：缓存数据结构，存储标签集合及版本。
#[derive(Debug, Clone)]
struct TagCache {
    tags: HashSet<String>,
    version: u64,
}

/// ：共享标签缓存，版本号用于判断是否需要刷新。
static TAG_CACHE: Lazy<RwLock<Option<TagCache>>> = Lazy::new(|| RwLock::new(None));

/// ：匹配各种需要检查的模式。
static TARGET_BLOCK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?is)\b[a-zA-Z0-9_\.]+\s*=\s*\{[^{}]*?target\s*=\s*([A-Za-z0-9]{2,4})")
        .expect("无效的 TARGET_BLOCK_REGEX 正则表达式模式: 静态字面量编译失败")
});

/// ：匹配原始等号形式。
static DIRECT_ASSIGN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(original_tag|tag|add_core_of|owner|ROOT/[A-Za-z0-9_]+|FROM/[A-Za-z0-9_]+)\s*=\s*([A-Za-z0-9]{2,4})",
    )
    .expect("无效的 DIRECT_ASSIGN_REGEX 正则表达式模式: 静态字面量编译失败")
});

/// ：匹配作用域块 `ROOT/X = {` 或 `FROM/X = {`。
static SCOPE_BLOCK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(ROOT|FROM)/([A-Za-z0-9]{2,4})\s*=\s*\{")
        .expect("无效的 SCOPE_BLOCK_REGEX 正则表达式模式: 静态字面量编译失败")
});

fn normalize_tag(tag: &str) -> String {
    tag.trim().to_uppercase()
}

fn build_tag_set(tags: &[TagEntry]) -> HashSet<String> {
    tags.iter().map(|entry| entry.code.clone()).collect()
}

fn ensure_tag_cache(
    project_root: Option<String>,
    game_root: Option<String>,
    dependency_roots: Option<Vec<String>>,
) -> HashSet<String> {
    let current_version = {
        let cache = TAG_CACHE.read().unwrap_or_else(|e| e.into_inner());
        cache.as_ref().map(|c| c.version).unwrap_or(0)
    };

    // 每次请求都重新加载 tags，保持与 country_tags 的缓存一致
    let TagLoadResponse { success, tags, .. } =
        load_country_tags(project_root.clone(), game_root.clone(), dependency_roots);
    if success {
        if let Some(tags) = tags {
            let tag_set = build_tag_set(&tags);
            let cache_entry = TagCache {
                tags: tag_set.clone(),
                version: current_version.wrapping_add(1),
            };
            if let Ok(mut cache) = TAG_CACHE.write() {
                *cache = Some(cache_entry);
            }
            return tag_set;
        }
    }

    // 如果加载失败，仍然返回现有缓存
    let cache = TAG_CACHE.read().unwrap_or_else(|e| e.into_inner());
    cache.as_ref().map(|c| c.tags.clone()).unwrap_or_default()
}

fn collect_direct_assignments(content: &str) -> Vec<(usize, String)> {
    DIRECT_ASSIGN_REGEX
        .captures_iter(content)
        .filter_map(|caps| {
            let full = caps.get(0)?;
            let tag = normalize_tag(caps.get(2)?.as_str());
            let line = content[..full.start()]
                .chars()
                .filter(|&c| c == '\n')
                .count()
                + 1;
            Some((line, tag))
        })
        .collect()
}

fn collect_scope_blocks(content: &str) -> Vec<(usize, String)> {
    SCOPE_BLOCK_REGEX
        .captures_iter(content)
        .filter_map(|caps| {
            let full = caps.get(0)?;
            let tag = normalize_tag(caps.get(2)?.as_str());
            let line = content[..full.start()]
                .chars()
                .filter(|&c| c == '\n')
                .count()
                + 1;
            Some((line, tag))
        })
        .collect()
}

fn collect_target_blocks(content: &str) -> Vec<(usize, String)> {
    TARGET_BLOCK_REGEX
        .captures_iter(content)
        .filter_map(|caps| {
            let full = caps.get(0)?;
            let tag = normalize_tag(caps.get(1)?.as_str());
            let line = content[..full.start()]
                .chars()
                .filter(|&c| c == '\n')
                .count()
                + 1;
            Some((line, tag))
        })
        .collect()
}

fn validate_tags_internal(content: &str, tags: &HashSet<String>) -> Vec<TagValidationError> {
    let mut errors: Vec<TagValidationError> = Vec::new();

    for (line, tag) in collect_direct_assignments(content) {
        if !tags.contains(&tag) {
            errors.push(TagValidationError {
                line,
                message: format!("未定义的国家标签: {}", tag),
            });
        }
    }

    for (line, tag) in collect_scope_blocks(content) {
        if !tags.contains(&tag) {
            errors.push(TagValidationError {
                line,
                message: format!("作用域引用未定义的国家标签: {}", tag),
            });
        }
    }

    for (line, tag) in collect_target_blocks(content) {
        if !tags.contains(&tag) {
            errors.push(TagValidationError {
                line,
                message: format!("target = {} 未定义", tag),
            });
        }
    }

    errors
}

/// ：对给定文本执行标签校验。
pub fn validate_tags_content(
    content: &str,
    project_root: Option<String>,
    game_root: Option<String>,
    dependency_roots: Option<Vec<String>>,
) -> TagValidationResponse {
    let tag_set = ensure_tag_cache(project_root, game_root, dependency_roots);
    if tag_set.is_empty() {
        return TagValidationResponse {
            success: false,
            message: "未能加载国家标签，请检查目录设置".to_string(),
            errors: Vec::new(),
        };
    }

    let errors = validate_tags_internal(content, &tag_set);
    if errors.is_empty() {
        TagValidationResponse {
            success: true,
            message: format!("校验通过，共 {} 个标签可用", tag_set.len()),
            errors,
        }
    } else {
        TagValidationResponse {
            success: false,
            message: format!("发现 {} 处未定义标签", errors.len()),
            errors,
        }
    }
}

/// ：Tauri 命令入口，供前端调用。
#[tauri::command]
pub fn validate_tags(
    content: String,
    project_root: Option<String>,
    game_root: Option<String>,
    dependency_roots: Option<Vec<String>>,
) -> TagValidationResponse {
    validate_tags_content(&content, project_root, game_root, dependency_roots)
}
