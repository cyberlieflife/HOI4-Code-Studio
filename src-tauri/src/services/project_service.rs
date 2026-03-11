// 项目管理服务
// 提供项目管理的核心业务逻辑

use chrono;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::*;

/// 项目管理服务结构体
pub struct ProjectService;

impl ProjectService {
    /// 创建新的项目服务实例
    pub fn new() -> Self {
        ProjectService
    }

    /// 获取最近项目文件路径
    ///
    /// # 返回值
    /// 返回最近项目列表文件的路径
    pub fn get_recent_projects_path(&self) -> PathBuf {
        let config_path = self.get_config_path();
        config_path
            .parent()
            .map(|p| p.join("recent_projects.json"))
            .unwrap_or_else(|| {
                let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
                config_dir
                    .join("HOI4_GUI_Editor")
                    .join("recent_projects.json")
            })
    }

    /// 获取最近项目统计缓存文件路径
    pub fn get_recent_project_stats_cache_path(&self) -> PathBuf {
        let config_path = self.get_config_path();
        config_path
            .parent()
            .map(|p| p.join("recent_project_stats_cache.json"))
            .unwrap_or_else(|| {
                let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
                config_dir
                    .join("HOI4_GUI_Editor")
                    .join("recent_project_stats_cache.json")
            })
    }

    /// 获取配置文件路径
    ///
    /// # 返回值
    /// 返回应用配置文件的路径
    pub fn get_config_path(&self) -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("HOI4_GUI_Editor").join("settings.json")
    }

    /// 更新最近项目列表
    ///
    /// # 参数
    /// * `project_path` - 项目路径
    /// * `project_name` - 项目名称
    ///
    /// # 返回值
    /// 成功返回 Ok(())，失败返回错误信息
    pub fn update_recent_projects(
        &self,
        project_path: &str,
        project_name: &str,
    ) -> Result<(), String> {
        let recent_path = self.get_recent_projects_path();

        // 确保目录存在
        if let Some(parent) = recent_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(format!("创建配置目录失败: {}", e));
            }
        }

        // 读取现有列表
        let mut projects: Vec<RecentProject> = if recent_path.exists() {
            match fs::read_to_string(&recent_path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| vec![]),
                Err(_) => vec![],
            }
        } else {
            vec![]
        };

        // 移除已存在的相同路径
        projects.retain(|p| p.path != project_path);

        // 添加新项目到列表开头
        projects.insert(
            0,
            RecentProject {
                name: project_name.to_string(),
                path: project_path.to_string(),
                last_opened: chrono::Utc::now().to_rfc3339(),
            },
        );

        // 只保留最近 10 个
        projects.truncate(10);

        // 保存到文件
        let content = match serde_json::to_string_pretty(&projects) {
            Ok(s) => s,
            Err(e) => return Err(format!("序列化失败: {}", e)),
        };

        match fs::write(&recent_path, content) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("写入文件失败: {}", e)),
        }
    }

    /// 扫描项目文件
    ///
    /// # 参数
    /// * `project_path` - 项目路径
    ///
    /// # 返回值
    /// 返回项目文件列表
    pub fn scan_project_files(&self, project_path: &str) -> Vec<serde_json::Value> {
        let mut files = Vec::new();
        let project_dir = Path::new(project_path);

        // 扫描项目目录
        if let Ok(entries) = fs::read_dir(project_dir) {
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

        files
    }
}

impl Default for ProjectService {
    fn default() -> Self {
        Self::new()
    }
}
