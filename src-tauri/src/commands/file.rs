// 文件操作相关命令
//
// 包含文件读取、写入、删除、列表等命令函数

use crate::file_tree::FileTreeResult;
use crate::models::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

/// 读取目录内容
#[tauri::command]
pub fn read_directory(dir_path: String) -> serde_json::Value {
    use std::fs;
    use std::path::Path;

    println!("读取目录: {}", dir_path);

    let dir = Path::new(&dir_path);

    // 验证路径
    if !dir.exists() || !dir.is_dir() {
        return serde_json::json!({
            "success": false,
            "message": "目录不存在",
            "files": []
        });
    }

    let mut files = Vec::new();

    // 扫描目录
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let file_name = entry.file_name().to_string_lossy().to_string();

                // 跳过隐藏文件和特定文件
                if file_name.starts_with('.') || file_name == "node_modules" {
                    continue;
                }

                files.push(serde_json::json!({
                    "name": file_name,
                    "path": entry.path().to_string_lossy().to_string(),
                    "is_directory": metadata.is_dir(),
                }));
            }
        }
    }

    // 按名称排序，目录在前
    files.sort_by(|a, b| {
        let a_is_dir = a
            .get("is_directory")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let b_is_dir = b
            .get("is_directory")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let a_name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let b_name = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
                a_name.cmp(b_name)
            }
        }
    });

    serde_json::json!({
        "success": true,
        "message": "读取成功",
        "files": files
    })
}

/// Tauri 命令：读取文件内容（支持多种编码）
/// 参数:
/// - file_path: 文件路径
/// 返回: JSON 对象，包含 success, message, content 字段
#[tauri::command]
pub fn read_file_content(file_path: String) -> serde_json::Value {
    use std::fs;
    use std::path::Path;

    println!("读取文件: {}", file_path);

    let path = Path::new(&file_path);

    if !path.exists() {
        return serde_json::json!({
            "success": false,
            "message": "文件不存在"
        });
    }

    // 检查是否为图片文件
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if matches!(
            ext_str.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "dds" | "tga"
        ) {
            return serde_json::json!({
                "success": false,
                "message": "图片文件无法预览",
                "is_image": true
            });
        }
    }

    // 读取文件字节
    let bytes = match fs::read(&file_path) {
        Ok(b) => b,
        Err(e) => {
            return serde_json::json!({
                "success": false,
                "message": format!("读取文件失败: {}", e)
            })
        }
    };

    // 1. 尝试UTF-8
    if let Ok(content) = String::from_utf8(bytes.clone()) {
        return serde_json::json!({
            "success": true,
            "message": "读取成功 (UTF-8)",
            "content": content,
            "encoding": "UTF-8"
        });
    }

    // 2. 使用chardetng检测编码
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(&bytes, true);
    let detected_encoding = detector.guess(None, true);

    println!("检测到编码: {}", detected_encoding.name());

    // 3. 尝试使用检测到的编码解码
    let (decoded, encoding_used, had_errors) = detected_encoding.decode(&bytes);

    if !had_errors {
        return serde_json::json!({
            "success": true,
            "message": format!("读取成功 ({})", encoding_used.name()),
            "content": decoded.to_string(),
            "encoding": encoding_used.name()
        });
    }

    // 4. 如果仍然有错误，尝试常见编码
    let encodings_to_try = [
        encoding_rs::GBK,          // 简体中文
        encoding_rs::BIG5,         // 繁体中文
        encoding_rs::SHIFT_JIS,    // 日文
        encoding_rs::EUC_KR,       // 韩文
        encoding_rs::WINDOWS_1252, // 西欧
    ];

    for encoding in encodings_to_try {
        let (decoded, _, had_errors) = encoding.decode(&bytes);
        if !had_errors {
            return serde_json::json!({
                "success": true,
                "message": format!("读取成功 ({})", encoding.name()),
                "content": decoded.to_string(),
                "encoding": encoding.name()
            });
        }
    }

    // 5. 最后使用lossy转换
    let content = String::from_utf8_lossy(&bytes).to_string();
    serde_json::json!({
        "success": true,
        "message": "读取成功（使用UTF-8 Lossy转换，部分字符可能显示为�）",
        "content": content,
        "encoding": "UTF-8 (Lossy)",
        "is_binary": true
    })
}

/// 写入文件内容
/// 参数:
/// - file_path: 文件路径
/// - content: 文件内容
/// 返回: JSON 对象，包含 success, message 字段
#[tauri::command]
pub fn write_file_content(file_path: String, content: String) -> serde_json::Value {
    use std::fs;
    use std::path::Path;

    println!("写入文件: {}", file_path);

    let path = Path::new(&file_path);

    // 写入文件
    match fs::write(path, content) {
        Ok(_) => serde_json::json!({
            "success": true,
            "message": "保存成功"
        }),
        Err(e) => serde_json::json!({
            "success": false,
            "message": format!("保存文件失败: {}", e)
        }),
    }
}

/// 创建新文件
#[tauri::command]
pub fn create_file(file_path: String, content: String, use_bom: bool) -> serde_json::Value {
    use std::fs;
    use std::path::Path;

    println!("创建文件: {}, 使用BOM: {}", file_path, use_bom);

    let path = Path::new(&file_path);

    // 检查文件是否已存在
    if path.exists() {
        return serde_json::json!({
            "success": false,
            "message": "文件已存在"
        });
    }

    // 确保父目录存在
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                return serde_json::json!({
                    "success": false,
                    "message": format!("创建目录失败: {}", e)
                });
            }
        }
    }

    // 准备文件内容
    let file_content = if use_bom {
        // UTF-8 BOM: EF BB BF
        let mut bom_content = vec![0xEF, 0xBB, 0xBF];
        bom_content.extend_from_slice(content.as_bytes());
        bom_content
    } else {
        content.as_bytes().to_vec()
    };

    // 写入文件
    match fs::write(&file_path, file_content) {
        Ok(_) => serde_json::json!({
            "success": true,
            "message": "文件创建成功",
            "path": file_path
        }),
        Err(e) => serde_json::json!({
            "success": false,
            "message": format!("创建文件失败: {}", e)
        }),
    }
}

/// 创建新文件夹
///
/// # 参数
/// * `folder_path` - 要创建的文件夹路径
///
/// # 返回值
/// 返回包含操作结果的 JSON 对象
#[tauri::command]
pub fn create_folder(folder_path: String) -> serde_json::Value {
    // 引入文件系统操作模块
    use std::fs;
    // 引入路径处理模块
    use std::path::Path;

    // 打印日志：显示正在创建的文件夹路径
    println!("创建文件夹: {}", folder_path);

    // 将字符串路径转换为 Path 对象
    let path = Path::new(&folder_path);

    // 检查文件夹是否已存在
    if path.exists() {
        // 如果文件夹已存在，返回失败结果
        return serde_json::json!({
            "success": false,
            "message": "文件夹已存在"
        });
    }

    // 创建文件夹（包括所有必要的父目录）
    match fs::create_dir_all(&folder_path) {
        // 创建成功
        Ok(_) => serde_json::json!({
            "success": true,
            "message": "文件夹创建成功",
            "path": folder_path
        }),
        // 创建失败，返回错误信息
        Err(e) => serde_json::json!({
            "success": false,
            "message": format!("创建文件夹失败: {}", e)
        }),
    }
}

/// 重命名文件或文件夹
#[tauri::command]
pub fn rename_path(old_path: String, new_path: String) -> serde_json::Value {
    use std::fs;
    use std::path::Path;

    println!("重命名: {} -> {}", old_path, new_path);

    let old = Path::new(&old_path);
    let new = Path::new(&new_path);

    if !old.exists() {
        return serde_json::json!({
            "success": false,
            "message": "源文件不存在"
        });
    }

    if new.exists() {
        return serde_json::json!({
            "success": false,
            "message": "目标路径已存在"
        });
    }

    match fs::rename(old, new) {
        Ok(_) => serde_json::json!({
            "success": true,
            "message": "重命名成功"
        }),
        Err(e) => serde_json::json!({
            "success": false,
            "message": format!("重命名失败: {}", e)
        }),
    }
}

/// 删除文件或文件夹
#[tauri::command]
pub fn delete_path(target_path: String) -> serde_json::Value {
    use std::fs;
    use std::path::Path;

    println!("删除路径: {}", target_path);

    let path = Path::new(&target_path);

    if !path.exists() {
        return serde_json::json!({
            "success": false,
            "message": "目标路径不存在"
        });
    }

    if path.is_file() {
        match fs::remove_file(path) {
            Ok(_) => serde_json::json!({
                "success": true,
                "message": "文件删除成功"
            }),
            Err(e) => serde_json::json!({
                "success": false,
                "message": format!("删除文件失败: {}", e)
            }),
        }
    } else {
        match fs::remove_dir_all(path) {
            Ok(_) => serde_json::json!({
                "success": true,
                "message": "文件夹删除成功"
            }),
            Err(e) => serde_json::json!({
                "success": false,
                "message": format!("删除文件夹失败: {}", e)
            }),
        }
    }
}

/// 打开文件夹
fn build_paste_operations(
    source_paths: &[String],
    target_dir: &std::path::Path,
) -> Result<Vec<(std::path::PathBuf, std::path::PathBuf)>, String> {
    if source_paths.is_empty() {
        return Err("没有可粘贴的文件或文件夹".to_string());
    }

    if !target_dir.exists() || !target_dir.is_dir() {
        return Err("目标目录不存在".to_string());
    }

    let mut operations = Vec::new();

    for source_path in source_paths {
        let source = std::path::PathBuf::from(source_path);
        if !source.exists() {
            return Err(format!("源路径不存在: {}", source_path));
        }

        let file_name = source
            .file_name()
            .ok_or_else(|| format!("无法解析路径名称: {}", source_path))?;
        let target = target_dir.join(file_name);

        if target.exists() {
            return Err(format!("目标路径已存在: {}", target.display()));
        }

        if source == target {
            return Err("不能粘贴到相同路径".to_string());
        }

        if source.is_dir() && target.starts_with(&source) {
            return Err(format!("不能将目录粘贴到自身内部: {}", source.display()));
        }

        operations.push((source, target));
    }

    Ok(operations)
}

fn copy_path_recursive(source: &std::path::Path, target: &std::path::Path) -> std::io::Result<()> {
    use std::fs;

    if source.is_file() {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, target)?;
        return Ok(());
    }

    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let child_source = entry.path();
        let child_target = target.join(entry.file_name());
        copy_path_recursive(&child_source, &child_target)?;
    }

    Ok(())
}

#[tauri::command]
pub fn copy_paths(source_paths: Vec<String>, target_dir: String) -> serde_json::Value {
    let target_dir_path = std::path::Path::new(&target_dir);
    let operations = match build_paste_operations(&source_paths, target_dir_path) {
        Ok(value) => value,
        Err(message) => {
            return serde_json::json!({
                "success": false,
                "message": message
            })
        }
    };

    for (source, target) in operations {
        if let Err(error) = copy_path_recursive(&source, &target) {
            return serde_json::json!({
                "success": false,
                "message": format!("复制失败: {}", error)
            });
        }
    }

    serde_json::json!({
        "success": true,
        "message": "复制成功"
    })
}

#[tauri::command]
pub fn move_paths(source_paths: Vec<String>, target_dir: String) -> serde_json::Value {
    use std::fs;

    let target_dir_path = std::path::Path::new(&target_dir);
    let operations = match build_paste_operations(&source_paths, target_dir_path) {
        Ok(value) => value,
        Err(message) => {
            return serde_json::json!({
                "success": false,
                "message": message
            })
        }
    };

    for (source, target) in operations {
        if let Err(error) = fs::rename(&source, &target) {
            return serde_json::json!({
                "success": false,
                "message": format!("移动失败: {}", error)
            });
        }
    }

    serde_json::json!({
        "success": true,
        "message": "移动成功"
    })
}

#[tauri::command]
pub fn open_folder(path: String) -> serde_json::Value {
    use std::process::Command;

    println!("打开文件夹: {}", path);

    // 根据操作系统使用不同的命令
    #[cfg(target_os = "windows")]
    let result = Command::new("explorer").arg(&path).spawn();

    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(&path).spawn();

    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(&path).spawn();

    match result {
        Ok(_) => serde_json::json!({
            "success": true,
            "message": "已打开文件夹"
        }),
        Err(e) => serde_json::json!({
            "success": false,
            "message": format!("打开文件夹失败: {}", e)
        }),
    }
}

/// 搜索文件内容（多线程）
///
/// # 参数
/// * `directory_path` - 要搜索的目录路径
/// * `query` - 搜索关键词
/// * `case_sensitive` - 是否区分大小写
/// * `use_regex` - 是否使用正则表达式
///
/// # 返回
/// 返回搜索结果的JSON对象
#[tauri::command]
pub fn search_files(
    directory_path: String,
    query: String,
    case_sensitive: bool,
    use_regex: bool,
    include_all_files: bool,
) -> serde_json::Value {
    use regex::Regex;
    use std::fs;
    use std::path::Path;

    println!("搜索目录: {}, 关键词: {}", directory_path, query);

    // 验证目录是否存在
    let dir_path = Path::new(&directory_path);
    if !dir_path.exists() || !dir_path.is_dir() {
        return serde_json::json!({
            "success": false,
            "message": "目录不存在",
            "results": []
        });
    }

    // 递归收集所有文件，支持文件类型过滤
    let mut all_files = Vec::new();

    // 使用 FileService 收集文件
    use crate::services::FileService;
    let file_service = FileService::new();
    file_service.collect_files(dir_path, &mut all_files, include_all_files);

    println!("找到 {} 个文件", all_files.len());

    // 使用Arc和Mutex来安全地共享结果
    let results = Arc::new(Mutex::new(Vec::new()));

    // 编译正则表达式（如果使用）
    let regex_pattern = if use_regex {
        if case_sensitive {
            Regex::new(&query).ok()
        } else {
            Regex::new(&format!("(?i){}", query)).ok()
        }
    } else {
        None
    };

    // 使用rayon进行并行搜索
    all_files.par_iter().for_each(|file_path| {
        // 读取文件内容
        if let Ok(content) = fs::read_to_string(file_path) {
            let lines: Vec<&str> = content.lines().collect();

            for (line_index, line) in lines.iter().enumerate() {
                let mut found = false;
                let mut match_start = 0;
                let mut match_end = 0;

                if let Some(ref regex) = regex_pattern {
                    // 使用正则表达式搜索
                    if let Some(mat) = regex.find(line) {
                        found = true;
                        match_start = mat.start();
                        match_end = mat.end();
                    }
                } else {
                    // 普通文本搜索
                    let search_line = if case_sensitive {
                        line.to_string()
                    } else {
                        line.to_lowercase()
                    };

                    let search_query = if case_sensitive {
                        query.clone()
                    } else {
                        query.to_lowercase()
                    };

                    if let Some(pos) = search_line.find(&search_query) {
                        found = true;
                        match_start = pos;
                        match_end = pos + query.len();
                    }
                }

                if found {
                    let result = SearchResult {
                        file_path: file_path.to_string_lossy().to_string(),
                        file_name: file_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        line: line_index + 1,
                        content: line.to_string(),
                        match_start,
                        match_end,
                    };

                    // 安全地添加结果
                    if let Ok(mut results_lock) = results.lock() {
                        results_lock.push(result);
                    }
                }
            }
        }
    });

    // 提取结果
    let final_results = match results.lock() {
        Ok(results_lock) => results_lock.clone(),
        Err(_) => Vec::new(),
    };

    println!("搜索完成，找到 {} 个匹配项", final_results.len());

    serde_json::json!({
        "success": true,
        "message": format!("找到 {} 个匹配项", final_results.len()),
        "results": final_results
    })
}

/// Tauri命令：构建文件树（单线程版本）
///
/// # 参数
/// * `path` - 目录路径
/// * `max_depth` - 最大递归深度（0表示无限制）
#[tauri::command]
pub fn build_directory_tree(path: String, max_depth: usize) -> FileTreeResult {
    use crate::file_tree::build_file_tree;
    println!("构建文件树: {}, 最大深度: {}", path, max_depth);
    build_file_tree(&path, max_depth)
}

/// Tauri命令：构建文件树（多线程版本）
///
/// # 参数
/// * `path` - 目录路径
/// * `max_depth` - 最大递归深度
#[tauri::command]
pub fn build_directory_tree_fast(path: String, max_depth: usize) -> FileTreeResult {
    use crate::file_tree::build_file_tree_parallel;
    // println!("快速构建文件树（多线程）: {}, 最大深度: {}", path, max_depth);
    build_file_tree_parallel(&path, max_depth)
}

/// 读取图片文件为 base64
///
/// # 参数
/// * `file_path` - 图片文件路径
#[tauri::command]
pub fn read_image_as_base64(file_path: String) -> ImageReadResult {
    use image::ImageFormat;
    use std::fs;
    use std::io::Cursor;

    println!("读取图片为 base64: {}", file_path);

    // 检查文件是否存在
    if !std::path::Path::new(&file_path).exists() {
        return ImageReadResult {
            success: false,
            message: Some("文件不存在".to_string()),
            base64: None,
            mime_type: None,
        };
    }

    // 获取文件扩展名
    let ext = std::path::Path::new(&file_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 对于 DDS 文件，使用 image_dds 库处理
    if ext.as_str() == "dds" {
        println!("转换 DDS 图片为 PNG: {}", file_path);

        match fs::read(&file_path) {
            Ok(dds_data) => {
                // 先解析 DDS 文件
                match image_dds::ddsfile::Dds::read(&mut Cursor::new(&dds_data)) {
                    Ok(dds) => {
                        // 使用 image_dds 解码 DDS 文件，尝试获取第一个 mipmap
                        match image_dds::image_from_dds(&dds, 0) {
                            Ok(img) => {
                                let mut buffer = Cursor::new(Vec::new());

                                // 转换为 PNG
                                match img.write_to(&mut buffer, ImageFormat::Png) {
                                    Ok(_) => {
                                        use base64::{engine::general_purpose, Engine as _};
                                        let base64_string =
                                            general_purpose::STANDARD.encode(buffer.get_ref());

                                        return ImageReadResult {
                                            success: true,
                                            message: None,
                                            base64: Some(base64_string),
                                            mime_type: Some("image/png".to_string()),
                                        };
                                    }
                                    Err(e) => {
                                        println!("转换 DDS 为 PNG 失败: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("无法从 DDS 创建图片: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("无法解析 DDS 文件: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("无法读取 DDS 文件: {}", e);
            }
        }
    }

    // 对于 TGA 文件，使用 image crate 转换为 PNG
    if ext.as_str() == "tga" {
        println!("转换 TGA 图片为 PNG: {}", file_path);

        // 打开图片
        match image::open(&file_path) {
            Ok(img) => {
                let mut buffer = Cursor::new(Vec::new());

                // 转换为 PNG
                match img.write_to(&mut buffer, ImageFormat::Png) {
                    Ok(_) => {
                        use base64::{engine::general_purpose, Engine as _};
                        let base64_string = general_purpose::STANDARD.encode(buffer.get_ref());

                        return ImageReadResult {
                            success: true,
                            message: None,
                            base64: Some(base64_string),
                            mime_type: Some("image/png".to_string()),
                        };
                    }
                    Err(e) => {
                        println!("转换图片格式失败: {}", e);
                        // 如果转换失败，尝试直接读取（可能前端有办法处理，或者只是为了显示错误）
                    }
                }
            }
            Err(e) => {
                println!("无法使用 image crate 打开图片: {}", e);
                // 失败后继续，尝试直接读取
            }
        }
    }

    // 读取文件（原始逻辑）
    match fs::read(&file_path) {
        Ok(bytes) => {
            // 转换为 base64
            use base64::{engine::general_purpose, Engine as _};
            let base64_string = general_purpose::STANDARD.encode(&bytes);

            let mime_type = match ext.as_str() {
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "bmp" => "image/bmp",
                "webp" => "image/webp",
                "svg" => "image/svg+xml",
                // 如果上面转换失败了，这里仍然返回原始 MIME
                "tga" => "image/x-tga",
                "dds" => "image/vnd-ms.dds",
                _ => "application/octet-stream",
            };

            ImageReadResult {
                success: true,
                message: None,
                base64: Some(base64_string),
                mime_type: Some(mime_type.to_string()),
            }
        }
        Err(e) => ImageReadResult {
            success: false,
            message: Some(format!("读取文件失败: {}", e)),
            base64: None,
            mime_type: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{copy_paths, move_paths};
    use std::fs;

    #[test]
    fn copy_paths_should_copy_file_and_directory() {
        let temp_dir = tempfile::tempdir().expect("应能创建临时目录");
        let source_root = temp_dir.path().join("source");
        let target_root = temp_dir.path().join("target");
        fs::create_dir_all(&source_root).expect("应能创建源目录");
        fs::create_dir_all(&target_root).expect("应能创建目标目录");

        let file_path = source_root.join("test.txt");
        fs::write(&file_path, "hello").expect("应能写入测试文件");

        let folder_path = source_root.join("folder");
        fs::create_dir_all(&folder_path).expect("应能创建测试文件夹");
        fs::write(folder_path.join("nested.txt"), "nested").expect("应能写入嵌套文件");

        let result = copy_paths(
            vec![
                file_path.to_string_lossy().to_string(),
                folder_path.to_string_lossy().to_string(),
            ],
            target_root.to_string_lossy().to_string(),
        );

        assert_eq!(result["success"].as_bool(), Some(true));
        assert_eq!(
            fs::read_to_string(target_root.join("test.txt")).expect("复制后应能读取文件"),
            "hello"
        );
        assert_eq!(
            fs::read_to_string(target_root.join("folder").join("nested.txt"))
                .expect("复制后应能读取嵌套文件"),
            "nested"
        );
    }

    #[test]
    fn move_paths_should_move_file_and_remove_source() {
        let temp_dir = tempfile::tempdir().expect("应能创建临时目录");
        let source_root = temp_dir.path().join("source");
        let target_root = temp_dir.path().join("target");
        fs::create_dir_all(&source_root).expect("应能创建源目录");
        fs::create_dir_all(&target_root).expect("应能创建目标目录");

        let file_path = source_root.join("move.txt");
        fs::write(&file_path, "move").expect("应能写入测试文件");

        let result = move_paths(
            vec![file_path.to_string_lossy().to_string()],
            target_root.to_string_lossy().to_string(),
        );

        assert_eq!(result["success"].as_bool(), Some(true));
        assert!(!file_path.exists());
        assert_eq!(
            fs::read_to_string(target_root.join("move.txt")).expect("移动后应能读取文件"),
            "move"
        );
    }
}
