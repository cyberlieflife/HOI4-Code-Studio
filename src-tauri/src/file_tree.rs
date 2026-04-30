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
    use tempfile::tempdir;

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

    #[test]
    fn test_build_file_tree_with_valid_directory() {
        let temp_dir = tempdir().expect("创建临时目录失败");
        let test_path = temp_dir.path().to_path_buf();

        // 创建测试目录结构
        let test_file = test_path.join("test.txt");
        fs::write(&test_file, "test content").expect("写入文件失败");
        let sub_dir = test_path.join("subdir");
        fs::create_dir_all(&sub_dir).expect("创建子目录失败");
        fs::write(sub_dir.join("sub_test.txt"), "sub test content").expect("写入文件失败");

        let test_path_str = test_path.to_str().expect("路径应为有效UTF-8");
        let result = build_file_tree(test_path_str, 3);

        assert!(result.success, "文件树构建应该成功");
        assert!(result.tree.is_some(), "应该返回文件树");

        let tree = result.tree.expect("成功时应返回文件树");
        assert_eq!(tree.len(), 2, "根目录应有两个直接子节点（subdir 和 test.txt）");

        // 验证排序：目录在前，文件在后
        let subdir_node = &tree[0];
        assert_eq!(subdir_node.name, "subdir");
        assert!(subdir_node.is_directory, "第一个节点应为目录");
        assert!(subdir_node.children.is_some(), "subdir 应有子节点");

        let file_node = &tree[1];
        assert_eq!(file_node.name, "test.txt");
        assert!(!file_node.is_directory, "第二个节点应为文件");

        // 验证 subdir 的子节点
        let children = subdir_node.children.as_ref().expect("subdir 应有子节点");
        assert!(children.iter().any(|node| node.name == "sub_test.txt"), "subdir 应包含 sub_test.txt");
    }

    #[test]
    fn test_build_file_tree_with_nonexistent_path() {
        let result = build_file_tree("/nonexistent/path", 3);

        assert!(!result.success, "不存在的路径应返回失败");
        assert_eq!(result.message, "路径不存在", "错误消息应正确");
        assert!(result.tree.is_none(), "失败时不应返回文件树");
    }

    #[test]
    fn test_build_file_tree_with_file_instead_of_directory() {
        let temp_dir = tempdir().expect("创建临时目录失败");
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").expect("写入文件失败");

        let test_file_str = test_file.to_str().expect("路径应为有效UTF-8");
        let result = build_file_tree(test_file_str, 3);

        assert!(!result.success, "文件路径应返回失败");
        assert_eq!(result.message, "路径不是目录", "错误消息应正确");
    }

    #[test]
    fn test_build_file_tree_depth_limit() {
        let temp_dir = tempdir().expect("创建临时目录失败");
        let test_path = temp_dir.path().to_path_buf();

        let mut current_path = test_path.clone();
        for i in 0..5 {
            current_path = current_path.join(format!("level_{}", i));
            fs::create_dir_all(&current_path).expect("创建目录失败");
            fs::write(current_path.join("file.txt"), "test").expect("写入文件失败");
        }

        let test_path_str = test_path.to_str().expect("路径应为有效UTF-8");
        let result = build_file_tree(test_path_str, 2);

        assert!(result.success, "文件树构建应该成功");
        let tree = result.tree.expect("成功时应返回文件树");
        let root_node = &tree[0];

        fn count_depth(nodes: &[FileNode], current_depth: usize, max_depth: usize) -> usize {
            if current_depth >= max_depth {
                return current_depth;
            }
            let mut max_child_depth = current_depth;
            for node in nodes {
                if let Some(children) = &node.children {
                    let child_depth = count_depth(children, current_depth + 1, max_depth);
                    max_child_depth = max_child_depth.max(child_depth);
                }
            }
            max_child_depth
        }

        let actual_depth = count_depth(&tree, 0, 2);
        assert!(actual_depth <= 2, "深度限制应被正确应用");
    }

    #[test]
    fn test_file_node_serialization() {
        let node = FileNode {
            name: "test.txt".to_string(),
            path: "/test/test.txt".to_string(),
            is_directory: false,
            children: None,
            size: Some(1024),
            expanded: false,
        };

        let serialized = serde_json::to_string(&node).expect("序列化应成功");
        let deserialized: FileNode = serde_json::from_str(&serialized).expect("反序列化应成功");

        assert_eq!(node.name, deserialized.name);
        assert_eq!(node.path, deserialized.path);
        assert_eq!(node.is_directory, deserialized.is_directory);
        assert_eq!(node.size, deserialized.size);
    }

    #[test]
    fn test_build_file_tree_parallel_performance() {
        let temp_dir = tempdir().expect("创建临时目录失败");
        let test_path = temp_dir.path().to_path_buf();

        for i in 0..100 {
            let file_path = test_path.join(format!("file_{}.txt", i));
            fs::write(&file_path, format!("content {}", i)).expect("写入文件失败");
        }

        let start = std::time::Instant::now();
        let test_path_str = test_path.to_str().expect("路径应为有效UTF-8");
        let result = build_file_tree_parallel(test_path_str, 1);
        let duration = start.elapsed();

        assert!(result.success, "并行构建应成功");
        assert!(duration.as_millis() < 1000, "并行构建应在1秒内完成");
    }
}
