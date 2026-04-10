// 设置相关命令
//
// 包含应用设置的加载、保存等命令函数

use crate::json_decoder::JsonResult;

/// 加载设置
#[tauri::command]
pub fn load_settings() -> JsonResult {
    use std::fs;

    let config_path = get_config_path();

    if !config_path.exists() {
        return JsonResult {
            success: true,
            message: "使用默认设置".to_string(),
            data: Some(serde_json::json!({
                "gameDirectory": "",
                "autoSave": true,
                "showGrid": false,
                "syntaxHighlight": true,
            })),
        };
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(settings) => JsonResult {
                success: true,
                message: "设置加载成功".to_string(),
                data: Some(settings),
            },
            Err(e) => JsonResult {
                success: false,
                message: format!("解析设置失败: {}", e),
                data: None,
            },
        },
        Err(e) => JsonResult {
            success: false,
            message: format!("读取设置失败: {}", e),
            data: None,
        },
    }
}

/// 保存设置
#[tauri::command]
pub fn save_settings(settings: serde_json::Value) -> JsonResult {
    use std::fs;

    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return JsonResult {
                success: false,
                message: format!("创建配置目录失败: {}", e),
                data: None,
            };
        }
    }

    match serde_json::to_string_pretty(&settings) {
        Ok(content) => match fs::write(&config_path, content) {
            Ok(_) => JsonResult {
                success: true,
                message: "设置保存成功".to_string(),
                data: None,
            },
            Err(e) => JsonResult {
                success: false,
                message: format!("写入设置失败: {}", e),
                data: None,
            },
        },
        Err(e) => JsonResult {
            success: false,
            message: format!("序列化设置失败: {}", e),
            data: None,
        },
    }
}

/// 获取默认缓存目录
#[tauri::command]
pub fn get_default_cache_directory() -> JsonResult {
    use crate::services::CacheService;
    let service = CacheService::new();
    let default_path = service.get_default_cache_root();

    JsonResult {
        success: true,
        message: "获取默认缓存目录成功".to_string(),
        data: Some(serde_json::Value::String(
            default_path.to_string_lossy().to_string(),
        )),
    }
}

/// 迁移缓存目录
///
/// 将旧缓存目录下的所有内容移动到新目录
#[tauri::command]
pub fn migrate_cache_directory(new_cache_dir: String) -> JsonResult {
    use crate::services::CacheService;
    let service = CacheService::new();
    let result = service.migrate_cache_directory(&new_cache_dir);

    JsonResult {
        success: result["success"].as_bool().unwrap_or(false),
        message: result["message"].as_str().unwrap_or("").to_string(),
        data: Some(result),
    }
}

/// 退出应用程序
#[tauri::command]
pub fn exit_application() {
    println!("退出应用程序");
    std::process::exit(0);
}

/// 打开设置页面（由前端处理跳转）
#[tauri::command]
pub fn open_settings() -> serde_json::Value {
    println!("打开设置");
    serde_json::json!({
        "success": true,
        "message": "设置页面"
    })
}

// ==================== 辅助函数 ====================

/// 获取配置文件路径
///
/// 使用 ProjectService 获取路径（设置和项目共享同一个配置目录）
fn get_config_path() -> std::path::PathBuf {
    use crate::services::ProjectService;
    let service = ProjectService::new();
    service.get_config_path()
}
