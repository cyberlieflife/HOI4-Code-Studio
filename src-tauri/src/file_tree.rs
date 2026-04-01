// 文件树构建模块
// 使用Rust的高级特性：异步、多线程、宏等
#![deny(clippy::unwrap_used)]
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// 文件树节点结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    /// 文件/文件夹名称
    pub name: String,
    /// 完整路径
    pub path: String,
    /// 是否为目录
    pub is_directory: bool,
    /// 子节点（仅目录有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileNode>>,
    /// 文件大小（字节）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// 是否展开（前端状态）
    #[serde(default)]
    pub expanded: bool,
}

/// 文件树构建结果
#[derive(Debug, Serialize, Deserialize)]
pub struct FileTreeResult {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree: Option<Vec<FileNode>>,
}

/// 宏：创建错误结果
macro_rules! error_result {
    ($msg:expr) => {
        FileTreeResult {
            success: false,
            message: $msg.to_string(),
            tree: None,
        }
    };
}

/// 宏：创建成功结果
macro_rules! success_result {
    ($tree:expr) => {
        FileTreeResult {
            success: true,
            message: "文件树构建成功".to_string(),
            tree: Some($tree),
        }
    };
}

/// 构建文件树（单线程版本，用于小型目录）
///
/// # 参数
/// * `path` - 目录路径
/// * `max_depth` - 最大递归深度（0表示无限制）
///
/// # 返回
/// 文件树构建结果
pub fn build_file_tree(path: &str, max_depth: usize) -> FileTreeResult {
    let path_buf = PathBuf::from(path);

    // 检查路径是否存在
    if !path_buf.exists() {
        return error_result!("路径不存在");
    }

    // 检查是否为目录
    if !path_buf.is_dir() {
        return error_result!("路径不是目录");
    }

    // 构建文件树
    match build_tree_recursive(&path_buf, 0, max_depth) {
        Ok(nodes) => success_result!(nodes),
        Err(e) => error_result!(format!("构建文件树失败: {}", e)),
    }
}

/// 递归构建文件树
///
/// # 参数
/// * `path` - 当前路径
/// * `current_depth` - 当前深度
/// * `max_depth` - 最大深度
fn build_tree_recursive(
    path: &Path,
    current_depth: usize,
    max_depth: usize,
) -> Result<Vec<FileNode>, std::io::Error> {
    // 检查深度限制
    if max_depth > 0 && current_depth >= max_depth {
        return Ok(Vec::new());
    }

    // 读取目录内容
    let entries = fs::read_dir(path)?;
    let mut nodes = Vec::new();

    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        let metadata = entry.metadata()?;

        // 获取文件名
        let name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // 获取完整路径
        let path_str = entry_path.to_str().unwrap_or("").to_string();

        if metadata.is_dir() {
            // 递归处理子目录
            let children = build_tree_recursive(&entry_path, current_depth + 1, max_depth)?;

            nodes.push(FileNode {
                name,
                path: path_str,
                is_directory: true,
                children: if children.is_empty() {
                    None
                } else {
                    Some(children)
                },
                size: None,
                expanded: false,
            });
        } else {
            // 文件节点
            nodes.push(FileNode {
                name,
                path: path_str,
                is_directory: false,
                children: None,
                size: Some(metadata.len()),
                expanded: false,
            });
        }
    }

    // 排序：目录在前，文件在后，同类按名称排序
    nodes.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(nodes)
}

/// 构建文件树（多线程版本，用于大型目录）
///
/// # 参数
/// * `path` - 目录路径
/// * `max_depth` - 最大递归深度
///
/// # 返回
/// 文件树构建结果
pub fn build_file_tree_parallel(path: &str, max_depth: usize) -> FileTreeResult {
    let path_buf = PathBuf::from(path);

    if !path_buf.exists() {
        return error_result!("路径不存在");
    }

    if !path_buf.is_dir() {
        return error_result!("路径不是目录");
    }

    // 使用Arc和Mutex实现线程安全的结果收集
    let result = Arc::new(Mutex::new(Vec::new()));

    match build_tree_parallel_recursive(&path_buf, 0, max_depth, result.clone()) {
        Ok(_) => {
            let nodes = result.lock().unwrap_or_else(|e| e.into_inner()).clone();
            success_result!(nodes)
        }
        Err(e) => error_result!(format!("构建文件树失败: {}", e)),
    }
}

/// 并行递归构建文件树
fn build_tree_parallel_recursive(
    path: &Path,
    current_depth: usize,
    max_depth: usize,
    result: Arc<Mutex<Vec<FileNode>>>,
) -> Result<(), std::io::Error> {
    if max_depth > 0 && current_depth >= max_depth {
        return Ok(());
    }

    let entries: Vec<_> = fs::read_dir(path)?.collect();

    // 使用rayon并行处理目录项
    let nodes: Vec<FileNode> = entries
        .par_iter()
        .filter_map(|entry| {
            let entry = entry.as_ref().ok()?;
            let entry_path = entry.path();
            let metadata = entry.metadata().ok()?;

            let name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            let path_str = entry_path.to_str().unwrap_or("").to_string();

            if metadata.is_dir() {
                // 对于子目录，递归构建
                let child_result = Arc::new(Mutex::new(Vec::new()));
                if build_tree_parallel_recursive(
                    &entry_path,
                    current_depth + 1,
                    max_depth,
                    child_result.clone(),
                )
                .is_ok()
                {
                    let children = child_result.lock().unwrap_or_else(|e| e.into_inner()).clone();
                    Some(FileNode {
                        name,
                        path: path_str,
                        is_directory: true,
                        children: if children.is_empty() {
                            None
                        } else {
                            Some(children)
                        },
                        size: None,
                        expanded: false,
                    })
                } else {
                    None
                }
            } else {
                Some(FileNode {
                    name,
                    path: path_str,
                    is_directory: false,
                    children: None,
                    size: Some(metadata.len()),
                    expanded: false,
                })
            }
        })
        .collect();

    // 排序并存储结果
    let mut sorted_nodes = nodes;
    sorted_nodes.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    *result.lock().unwrap_or_else(|e| e.into_inner()) = sorted_nodes;
    Ok(())
}

/// 过滤文件树（根据文件扩展名）
///
/// # 参数
/// * `nodes` - 文件树节点
/// * `extensions` - 允许的文件扩展名列表
#[allow(dead_code)]
pub fn filter_by_extensions(nodes: &mut Vec<FileNode>, extensions: &[String]) {
    nodes.retain(|node| {
        if node.is_directory {
            // 保留所有目录
            true
        } else {
            // 检查文件扩展名
            let path = Path::new(&node.path);
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    return extensions.iter().any(|e| e.eq_ignore_ascii_case(ext_str));
                }
            }
            false
        }
    });

    // 递归过滤子节点
    for node in nodes.iter_mut() {
        if let Some(ref mut children) = node.children {
            filter_by_extensions(children, extensions);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_file_tree() {
        let result = build_file_tree(".", 2);
        assert!(result.success);
        assert!(result.tree.is_some());
    }

    #[test]
    fn test_build_file_tree_parallel() {
        let result = build_file_tree_parallel(".", 2);
        assert!(result.success);
        assert!(result.tree.is_some());
    }
}
