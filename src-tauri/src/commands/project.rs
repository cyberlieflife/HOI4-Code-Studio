// 项目管理相关命令
//
// 包含项目创建、打开、保存、关闭等命令函数

use crate::models::*;

/// 创建新项目
#[tauri::command]
pub fn create_new_project(
    project_name: String,
    version: String,
    project_path: String,
    replace_path: Vec<String>,
) -> CreateProjectResult {
    use std::fs;
    use std::path::Path;

    println!(
        "创建新项目: {} v{} 于 {}",
        project_name, version, project_path
    );
    println!("Replace Path 目录: {:?}", replace_path);

    if project_name.is_empty() {
        return CreateProjectResult {
            success: false,
            message: "项目名称不能为空".to_string(),
            project_path: None,
        };
    }

    let full_path = Path::new(&project_path).join(&project_name);

    if full_path.exists() {
        return CreateProjectResult {
            success: false,
            message: "项目目录已存在".to_string(),
            project_path: None,
        };
    }

    if let Err(e) = fs::create_dir_all(&full_path) {
        return CreateProjectResult {
            success: false,
            message: format!("创建目录失败: {}", e),
            project_path: None,
        };
    }

    let subdirs = vec!["interface", "gfx", "localisation"];

    for subdir in subdirs {
        let subdir_path = full_path.join(subdir);

        if let Err(e) = fs::create_dir_all(&subdir_path) {
            return CreateProjectResult {
                success: false,
                message: format!("创建子目录 {} 失败: {}", subdir, e),
                project_path: None,
            };
        }
    }

    for dir in &replace_path {
        let replace_path_dir = full_path.join(dir);

        if let Err(e) = fs::create_dir_all(&replace_path_dir) {
            return CreateProjectResult {
                success: false,
                message: format!("创建 replace_path 目录 {} 失败: {}", dir, e),
                project_path: None,
            };
        }
    }

    let config = serde_json::json!({
        "name": project_name,
        "version": version,
        "replace_path": replace_path,
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    let config_path = full_path.join("project.json");
    let config_str = match serde_json::to_string_pretty(&config) {
        Ok(s) => s,
        Err(e) => {
            return CreateProjectResult {
                success: false,
                message: format!("序列化配置失败: {}", e),
                project_path: None,
            };
        }
    };

    if let Err(e) = fs::write(&config_path, config_str) {
        return CreateProjectResult {
            success: false,
            message: format!("创建配置文件失败: {}", e),
            project_path: None,
        };
    }

    let replace_path_str = if !replace_path.is_empty() {
        replace_path
            .iter()
            .map(|s| format!("replace_path=\"{}\"", s))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    } else {
        String::new()
    };

    let descriptor_content = format!(
        r#"version="{}"
tags={{
    "Graphics"
}}
name="{}"
{}supported_version="1.14.*"
"#,
        version, project_name, replace_path_str
    );

    let descriptor_path = full_path.join("descriptor.mod");

    if let Err(e) = fs::write(&descriptor_path, descriptor_content) {
        return CreateProjectResult {
            success: false,
            message: format!("创建 descriptor.mod 失败: {}", e),
            project_path: None,
        };
    }

    CreateProjectResult {
        success: true,
        message: format!("项目 '{}' 创建成功", project_name),
        project_path: Some(full_path.to_string_lossy().to_string()),
    }
}

/// 初始化项目（为非HOICS项目创建配置文件）
#[tauri::command]
pub fn initialize_project(project_path: String) -> OpenProjectResult {
    use std::fs;
    use std::path::Path;

    println!("初始化项目: {}", project_path);

    // 验证项目路径是否存在
    let project_dir = Path::new(&project_path);
    if !project_dir.exists() || !project_dir.is_dir() {
        return OpenProjectResult {
            success: false,
            message: "项目目录不存在".to_string(),
            project_data: None,
        };
    }

    // 检查是否已经存在配置文件
    let config_path = project_dir.join("project.json");
    if config_path.exists() {
        return OpenProjectResult {
            success: false,
            message: "项目已存在配置文件".to_string(),
            project_data: None,
        };
    }

    // 读取descriptor.mod文件
    let descriptor_path = project_dir.join("descriptor.mod");
    let mod_name = if descriptor_path.exists() {
        match fs::read_to_string(&descriptor_path) {
            Ok(content) => {
                // 解析name属性
                if let Some(name_match) = content
                    .lines()
                    .find(|line| line.trim().starts_with("name="))
                    .and_then(|line| {
                        let line = line.trim();
                        if line.starts_with("name=\"") && line.ends_with('\"') && line.len() > 7 {
                            Some(line[6..line.len() - 1].to_string())
                        } else {
                            None
                        }
                    })
                {
                    name_match
                } else {
                    "Unknown Mod".to_string()
                }
            }
            Err(_) => "Unknown Mod".to_string(),
        }
    } else {
        "Unknown Mod".to_string()
    };

    // 创建项目配置
    let mut config = serde_json::json!({
        "name": mod_name,
        "version": "1.0.0",
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    // 添加项目路径到配置
    if let Some(obj) = config.as_object_mut() {
        obj.insert("path".to_string(), serde_json::json!(project_path.clone()));
    }

    // 扫描项目文件
    let files = scan_project_files(&project_path);
    if let Some(obj) = config.as_object_mut() {
        obj.insert("files".to_string(), serde_json::json!(files));
    }

    // 保存配置文件
    let config_str = match serde_json::to_string_pretty(&config) {
        Ok(s) => s,
        Err(e) => {
            return OpenProjectResult {
                success: false,
                message: format!("序列化配置失败: {}", e),
                project_data: None,
            };
        }
    };

    if let Err(e) = fs::write(&config_path, config_str) {
        return OpenProjectResult {
            success: false,
            message: format!("创建配置文件失败: {}", e),
            project_data: None,
        };
    }

    // 更新最近项目列表
    if let Err(e) = update_recent_projects(&project_path, &mod_name) {
        println!("更新最近项目失败: {}", e);
    }

    OpenProjectResult {
        success: true,
        message: format!("项目 '{}' 初始化成功", mod_name),
        project_data: Some(config),
    }
}

/// 打开现有项目
#[tauri::command]
pub fn open_project(project_path: String) -> OpenProjectResult {
    use std::fs;
    use std::path::Path;

    println!("打开项目: {}", project_path);

    // 验证项目路径是否存在
    let project_dir = Path::new(&project_path);
    if !project_dir.exists() || !project_dir.is_dir() {
        return OpenProjectResult {
            success: false,
            message: "项目目录不存在".to_string(),
            project_data: None,
        };
    }

    // 读取项目配置文件
    let config_path = project_dir.join("project.json");
    if !config_path.exists() {
        return OpenProjectResult {
            success: false,
            message: "检测到此文件夹不是HOI4 Code Studio项目，是否要将其初始化为项目？".to_string(),
            project_data: None,
        };
    }

    // 解析配置文件
    let config_content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => {
            return OpenProjectResult {
                success: false,
                message: format!("读取配置文件失败: {}", e),
                project_data: None,
            };
        }
    };

    let mut config: serde_json::Value = match serde_json::from_str(&config_content) {
        Ok(cfg) => cfg,
        Err(e) => {
            return OpenProjectResult {
                success: false,
                message: format!("解析配置文件失败: {}", e),
                project_data: None,
            };
        }
    };

    // 添加项目路径到配置
    if let Some(obj) = config.as_object_mut() {
        obj.insert("path".to_string(), serde_json::json!(project_path.clone()));
    }

    // 扫描项目文件
    let files = scan_project_files(&project_path);
    if let Some(obj) = config.as_object_mut() {
        obj.insert("files".to_string(), serde_json::json!(files));
    }

    // 更新最近项目列表
    if let Err(e) = update_recent_projects(
        &project_path,
        config
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("未命名项目"),
    ) {
        println!("更新最近项目失败: {}", e);
    }

    OpenProjectResult {
        success: true,
        message: "项目打开成功".to_string(),
        project_data: Some(config),
    }
}

/// 获取最近打开的项目列表
#[tauri::command]
pub fn get_recent_projects() -> RecentProjectsResult {
    use std::fs;

    println!("获取最近项目列表");

    // 获取最近项目文件路径
    let recent_path = get_recent_projects_path();

    // 如果文件不存在，返回空列表
    if !recent_path.exists() {
        return RecentProjectsResult {
            success: true,
            projects: vec![],
        };
    }

    // 读取并解析文件
    match fs::read_to_string(&recent_path) {
        Ok(content) => {
            match serde_json::from_str::<Vec<RecentProject>>(&content) {
                Ok(mut projects) => {
                    // 按最后打开时间排序（最新的在前）
                    projects.sort_by(|a, b| b.last_opened.cmp(&a.last_opened));

                    // 只保留前 10 个
                    projects.truncate(10);

                    RecentProjectsResult {
                        success: true,
                        projects,
                    }
                }
                Err(e) => {
                    println!("解析最近项目文件失败: {}", e);
                    RecentProjectsResult {
                        success: true,
                        projects: vec![],
                    }
                }
            }
        }
        Err(e) => {
            println!("读取最近项目文件失败: {}", e);
            RecentProjectsResult {
                success: true,
                projects: vec![],
            }
        }
    }
}

/// 获取最近项目的统计信息
#[tauri::command]
pub fn get_recent_project_stats(paths: Vec<String>) -> RecentProjectStatsResult {
    use rayon::prelude::*;
    use std::fs;
    use std::path::Path;
    use walkdir::WalkDir;

    // 使用 rayon 并行处理多个项目的路径扫描
    let stats: Vec<ProjectStats> = paths
        .into_par_iter()
        .map(|path| {
            let project_path = Path::new(&path);

            let mut file_count: u64 = 0;
            let mut total_size: u64 = 0;

            if project_path.exists() && project_path.is_dir() {
                // 虽然 walkdir 本身是单线程的，但 rayon 使得多个项目可以同时被扫描
                for entry in WalkDir::new(project_path).follow_links(false).into_iter() {
                    let entry = match entry {
                        Ok(e) => e,
                        Err(_) => continue,
                    };

                    if !entry.file_type().is_file() {
                        continue;
                    }

                    file_count = file_count.saturating_add(1);
                    if let Ok(meta) = entry.metadata() {
                        total_size = total_size.saturating_add(meta.len());
                    }
                }
            }

            // 获取项目版本号（如果有 project.json）
            let version = project_path
                .join("project.json")
                .to_str()
                .and_then(|p| fs::read_to_string(p).ok())
                .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                .and_then(|v| {
                    v.get("version")
                        .and_then(|vv| vv.as_str())
                        .map(|s| s.to_string())
                });

            ProjectStats {
                path,
                file_count,
                total_size,
                version,
            }
        })
        .collect();

    RecentProjectStatsResult {
        success: true,
        stats,
    }
}

const RECENT_PROJECT_STATS_CACHE_TTL_SECONDS: i64 = 300;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CachedProjectStatsEntry {
    updated_at: String,
    stats: ProjectStats,
}

#[tauri::command]
pub fn get_recent_project_stats_cached(paths: Vec<String>) -> RecentProjectStatsResult {
    let mut cache = read_recent_project_stats_cache();
    let mut stats = Vec::with_capacity(paths.len());
    let mut stale_paths = Vec::new();
    let mut cache_changed = false;

    for path in &paths {
        match cache.get(path) {
            Some(entry) => {
                stats.push(entry.stats.clone());
                if !is_recent_project_stats_cache_fresh(entry) {
                    stale_paths.push(path.clone());
                }
            }
            None => {
                let computed = compute_project_stats(path);
                cache.insert(path.clone(), build_cached_project_stats(&computed));
                stats.push(computed);
                cache_changed = true;
            }
        }
    }

    if cache_changed {
        let _ = write_recent_project_stats_cache(&cache);
    }

    if !stale_paths.is_empty() {
        std::thread::spawn(move || {
            refresh_recent_project_stats_cache(stale_paths);
        });
    }

    RecentProjectStatsResult {
        success: true,
        stats,
    }
}

fn build_cached_project_stats(stats: &ProjectStats) -> CachedProjectStatsEntry {
    CachedProjectStatsEntry {
        updated_at: chrono::Utc::now().to_rfc3339(),
        stats: stats.clone(),
    }
}

fn is_recent_project_stats_cache_fresh(entry: &CachedProjectStatsEntry) -> bool {
    chrono::DateTime::parse_from_rfc3339(&entry.updated_at)
        .map(|updated_at| {
            let age =
                chrono::Utc::now().signed_duration_since(updated_at.with_timezone(&chrono::Utc));
            age.num_seconds() < RECENT_PROJECT_STATS_CACHE_TTL_SECONDS
        })
        .unwrap_or(false)
}

fn read_recent_project_stats_cache() -> std::collections::HashMap<String, CachedProjectStatsEntry> {
    use std::fs;

    let cache_path = get_recent_project_stats_cache_path();
    if !cache_path.exists() {
        return std::collections::HashMap::new();
    }

    fs::read_to_string(&cache_path)
        .ok()
        .and_then(|content| {
            serde_json::from_str::<std::collections::HashMap<String, CachedProjectStatsEntry>>(
                &content,
            )
            .ok()
        })
        .unwrap_or_default()
}

fn write_recent_project_stats_cache(
    cache: &std::collections::HashMap<String, CachedProjectStatsEntry>,
) -> Result<(), String> {
    use std::fs;

    let cache_path = get_recent_project_stats_cache_path();
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建统计缓存目录失败: {}", e))?;
    }

    let content =
        serde_json::to_string_pretty(cache).map_err(|e| format!("序列化统计缓存失败: {}", e))?;

    fs::write(cache_path, content).map_err(|e| format!("写入统计缓存失败: {}", e))
}

fn refresh_recent_project_stats_cache(paths: Vec<String>) {
    let mut cache = read_recent_project_stats_cache();
    let mut changed = false;

    for path in paths {
        let stats = compute_project_stats(&path);
        cache.insert(path, build_cached_project_stats(&stats));
        changed = true;
    }

    if changed {
        let _ = write_recent_project_stats_cache(&cache);
    }
}

fn compute_project_stats(path: &str) -> ProjectStats {
    use std::fs;
    use std::path::Path;
    use walkdir::WalkDir;

    let project_path = Path::new(path);
    let mut file_count: u64 = 0;
    let mut total_size: u64 = 0;

    if project_path.exists() && project_path.is_dir() {
        for entry in WalkDir::new(project_path).follow_links(false).into_iter() {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if !entry.file_type().is_file() {
                continue;
            }

            file_count = file_count.saturating_add(1);
            if let Ok(meta) = entry.metadata() {
                total_size = total_size.saturating_add(meta.len());
            }
        }
    }

    let version = project_path
        .join("project.json")
        .to_str()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
        .and_then(|v| {
            v.get("version")
                .and_then(|vv| vv.as_str())
                .map(|s| s.to_string())
        });

    ProjectStats {
        path: path.to_string(),
        file_count,
        total_size,
        version,
    }
}

/// 打开文件选择对话框
#[tauri::command]
pub async fn open_file_dialog(mode: String) -> FileDialogResult {
    println!("打开文件对话框: {}", mode);

    match mode.as_str() {
        "directory" => {
            let folder = rfd::AsyncFileDialog::new()
                .set_title("选择文件夹")
                .pick_folder()
                .await;

            match folder {
                Some(handle) => {
                    let path = handle.path().to_string_lossy().to_string();
                    println!("选择的文件夹: {}", path);
                    FileDialogResult {
                        success: true,
                        path: Some(path),
                    }
                }
                None => {
                    println!("用户取消了选择");
                    FileDialogResult {
                        success: false,
                        path: None,
                    }
                }
            }
        }
        "file" => {
            let file = rfd::AsyncFileDialog::new()
                .set_title("选择文件")
                .pick_file()
                .await;

            match file {
                Some(handle) => {
                    let path = handle.path().to_string_lossy().to_string();
                    println!("选择的文件: {}", path);
                    FileDialogResult {
                        success: true,
                        path: Some(path),
                    }
                }
                None => {
                    println!("用户取消了选择");
                    FileDialogResult {
                        success: false,
                        path: None,
                    }
                }
            }
        }
        _ => {
            println!("无效的 mode: {}", mode);
            FileDialogResult {
                success: false,
                path: None,
            }
        }
    }
}

// ==================== 辅助函数 ====================

/// 获取最近项目文件路径
///
/// 使用 ProjectService 获取路径
fn get_recent_projects_path() -> std::path::PathBuf {
    use crate::services::ProjectService;
    let service = ProjectService::new();
    service.get_recent_projects_path()
}

fn get_recent_project_stats_cache_path() -> std::path::PathBuf {
    use crate::services::ProjectService;
    let service = ProjectService::new();
    service.get_recent_project_stats_cache_path()
}

/// 更新最近项目列表
///
/// 使用 ProjectService 更新列表
fn update_recent_projects(project_path: &str, project_name: &str) -> Result<(), String> {
    use crate::services::ProjectService;
    let service = ProjectService::new();
    service.update_recent_projects(project_path, project_name)
}

/// 扫描项目文件
///
/// 使用 ProjectService 扫描文件
fn scan_project_files(project_path: &str) -> Vec<serde_json::Value> {
    use crate::services::ProjectService;
    let service = ProjectService::new();
    service.scan_project_files(project_path)
}

// ==================== 项目打包命令 ====================

/// 打包项目到 ZIP 文件的内部实现
///
/// # 参数
/// * `opts` - 打包选项
fn package_project_impl(opts: PackageOptions) -> PackageResult {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use walkdir::WalkDir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    println!("开始打包项目: {}", opts.project_path);
    println!("输出文件名: {}", opts.output_name);

    let project_path_obj = Path::new(&opts.project_path);
    if !project_path_obj.exists() {
        return PackageResult {
            success: false,
            message: "项目路径不存在".to_string(),
            output_path: None,
            file_size: None,
        };
    }

    // 创建 package 目录
    let package_dir = project_path_obj.join("package");
    if let Err(e) = fs::create_dir_all(&package_dir) {
        return PackageResult {
            success: false,
            message: format!("创建 package 目录失败: {}", e),
            output_path: None,
            file_size: None,
        };
    }

    // 输出文件路径
    let output_path = package_dir.join(&opts.output_name);

    // 如果文件已存在，尝试删除
    if output_path.exists() {
        if let Err(e) = fs::remove_file(&output_path) {
            return PackageResult {
                success: false,
                message: format!("无法覆盖已存在的文件: {}", e),
                output_path: None,
                file_size: None,
            };
        }
    }

    // 创建 ZIP 文件
    let file = match File::create(&output_path) {
        Ok(f) => f,
        Err(e) => {
            return PackageResult {
                success: false,
                message: format!("创建 ZIP 文件失败: {}", e),
                output_path: None,
                file_size: None,
            };
        }
    };

    let mut zip = ZipWriter::new(file);
    let file_options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));

    // 定义要排除的目录和文件
    let exclude_dirs = [
        "node_modules",
        "target",
        ".git",
        ".idea",
        ".vscode",
        ".windsurf",
        "package",
    ];

    // 读取 dependencies.json 获取依赖项路径（如果需要排除）
    let dependency_paths: Vec<PathBuf> = if opts.exclude_dependencies {
        let deps_file = project_path_obj.join("dependencies.json");
        if deps_file.exists() {
            match fs::read_to_string(&deps_file) {
                Ok(content) => {
                    if let Ok(deps) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(arr) = deps.as_array() {
                            arr.iter()
                                .filter_map(|dep| {
                                    dep.get("path").and_then(|p| p.as_str()).map(PathBuf::from)
                                })
                                .collect()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                }
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // 遍历项目文件夹
    let mut file_count = 0;
    for entry in WalkDir::new(project_path_obj)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let path = e.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // 排除指定目录
            if e.file_type().is_dir() && exclude_dirs.contains(&file_name) {
                return false;
            }

            // 排除依赖项路径（如果依赖项在项目内部）
            for dep_path in &dependency_paths {
                if path.starts_with(dep_path) {
                    return false;
                }
            }

            true
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        // 跳过项目根目录本身
        if path == project_path_obj {
            continue;
        }

        // 跳过目录，只打包文件
        if !path.is_file() {
            continue;
        }

        // 计算相对路径
        let relative_path = match path.strip_prefix(project_path_obj) {
            Ok(p) => p,
            Err(_) => continue,
        };

        // 转换为字符串路径（使用 / 作为分隔符）
        let zip_path = relative_path.to_str().unwrap_or("").replace("\\", "/");

        // 读取文件内容
        let file_content = match fs::read(path) {
            Ok(content) => content,
            Err(e) => {
                println!("警告: 无法读取文件 {}: {}", path.display(), e);
                continue;
            }
        };

        // 添加到 ZIP
        if let Err(e) = zip.start_file(&zip_path, file_options) {
            println!("警告: 无法添加文件到 ZIP {}: {}", zip_path, e);
            continue;
        }

        if let Err(e) = zip.write_all(&file_content) {
            println!("警告: 无法写入文件内容 {}: {}", zip_path, e);
            continue;
        }

        file_count += 1;
    }

    // 完成 ZIP 文件
    if let Err(e) = zip.finish() {
        return PackageResult {
            success: false,
            message: format!("完成 ZIP 文件失败: {}", e),
            output_path: None,
            file_size: None,
        };
    }

    // 获取文件大小
    let file_size = fs::metadata(&output_path).ok().map(|m| m.len());

    println!("打包完成: {} 个文件", file_count);

    PackageResult {
        success: true,
        message: format!("打包成功！已打包 {} 个文件", file_count),
        output_path: Some(output_path.to_string_lossy().to_string()),
        file_size,
    }
}

/// 打包项目 Tauri 命令
#[tauri::command]
pub fn pack_project(opts: PackageOptions) -> PackageResult {
    package_project_impl(opts)
}
