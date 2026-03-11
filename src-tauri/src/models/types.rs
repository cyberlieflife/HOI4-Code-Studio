// 数据模型层 - 所有数据结构定义
// 本文件包含应用中使用的所有数据类型定义

use serde::{Deserialize, Serialize};

// ==================== 项目相关数据结构 ====================

/// 创建项目的返回结果结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProjectResult {
    pub success: bool,
    pub message: String,
    pub project_path: Option<String>,
}

/// 打开项目的返回结果结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenProjectResult {
    pub success: bool,
    pub message: String,
    pub project_data: Option<serde_json::Value>,
}

/// 最近打开的项目信息
#[derive(Debug, Serialize, Deserialize)]
pub struct RecentProject {
    pub name: String,
    pub path: String,
    pub last_opened: String,
}

/// 最近项目列表的返回结果
#[derive(Debug, Serialize, Deserialize)]
pub struct RecentProjectsResult {
    pub success: bool,
    pub projects: Vec<RecentProject>,
}

/// 项目统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStats {
    pub path: String,
    pub file_count: u64,
    pub total_size: u64,
    pub version: Option<String>,
}

/// 最近项目统计结果
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentProjectStatsResult {
    pub success: bool,
    pub stats: Vec<ProjectStats>,
}

// ==================== 文件操作相关数据结构 ====================

/// 文件对话框返回结果
#[derive(Debug, Serialize, Deserialize)]
pub struct FileDialogResult {
    pub success: bool,
    pub path: Option<String>,
}

// ==================== 游戏集成相关数据结构 ====================

/// 启动游戏的返回结果
#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchGameResult {
    pub success: bool,
    pub message: String,
}

// ==================== 打包相关数据结构 ====================

/// 打包项目的返回结果
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageResult {
    pub success: bool,
    pub message: String,
    pub output_path: Option<String>,
    pub file_size: Option<u64>,
}

/// 打包选项配置
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageOptions {
    pub project_path: String,
    pub output_name: String,
    pub exclude_dependencies: bool,
}

// ==================== 图片处理相关数据结构 ====================

/// 图片读取结果
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageReadResult {
    pub success: bool,
    pub message: Option<String>,
    pub base64: Option<String>,
    pub mime_type: Option<String>,
}

// ==================== GFX 预览相关数据结构 ====================

/// GFX 精灵预览项
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GfxSpritePreviewItem {
    pub name: String,
    pub texturefile: Option<String>,
    pub no_of_frames: i32,
    pub border_size: Option<serde_json::Value>,
    pub source_line: i32,
    pub resolved_path: Option<String>,
    pub cached_png_path: Option<String>,
    pub error: Option<String>,
}

// ==================== 搜索相关数据结构 ====================

/// 搜索结果结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    /// 文件路径
    pub file_path: String,
    /// 文件名
    pub file_name: String,
    /// 行号（从1开始）
    pub line: usize,
    /// 匹配的内容
    pub content: String,
    /// 匹配开始位置
    pub match_start: usize,
    /// 匹配结束位置
    pub match_end: usize,
}
