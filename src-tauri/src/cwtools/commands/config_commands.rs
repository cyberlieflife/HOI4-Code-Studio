//! 配置管理相关的 Tauri 命令

use crate::cwtools::config::{ConfigManager, ValidationConfig};
use crate::cwtools::diagnostic::Severity;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

/// 全局配置管理器状态
pub struct ConfigManagerState {
    manager: Mutex<ConfigManager>,
}

impl ConfigManagerState {
    /// 创建新的配置管理器状态
    pub fn new() -> Self {
        Self {
            manager: Mutex::new(ConfigManager::new()),
        }
    }
}

/// 配置响应
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub message: Option<String>,
    pub config: Option<ValidationConfig>,
}

impl ConfigResponse {
    /// 创建成功响应
    pub fn success(config: Option<ValidationConfig>) -> Self {
        Self {
            success: true,
            message: None,
            config,
        }
    }

    /// 创建成功响应并附带消息
    pub fn success_with_message(message: String, config: Option<ValidationConfig>) -> Self {
        Self {
            success: true,
            message: Some(message),
            config,
        }
    }

    /// 创建错误响应
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message: Some(message),
            config: None,
        }
    }
}

/// 规则配置请求
#[derive(Debug, Deserialize)]
pub struct RuleConfigRequest {
    pub rule_name: String,
    pub enabled: Option<bool>,
    pub severity: Option<Severity>,
}

/// 加载配置文件
///
/// # 参数
/// * `state` - 配置管理器状态
/// * `path` - 配置文件路径
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn load_config(
    state: State<'_, ConfigManagerState>,
    path: String,
) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    let path_buf = PathBuf::from(path);
    manager.load(&path_buf).map_err(|e| e.to_string())?;

    let config = manager.config().clone();
    Ok(ConfigResponse::success_with_message(
        "配置加载成功".to_string(),
        Some(config),
    ))
}

/// 保存配置文件
///
/// # 参数
/// * `state` - 配置管理器状态
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn save_config(state: State<'_, ConfigManagerState>) -> Result<ConfigResponse, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;

    manager.save().map_err(|e| e.to_string())?;

    Ok(ConfigResponse::success_with_message(
        "配置保存成功".to_string(),
        None,
    ))
}

/// 保存配置到指定文件
///
/// # 参数
/// * `state` - 配置管理器状态
/// * `path` - 配置文件路径
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn save_config_as(
    state: State<'_, ConfigManagerState>,
    path: String,
) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    let path_buf = PathBuf::from(path);
    manager.save_as(&path_buf).map_err(|e| e.to_string())?;

    Ok(ConfigResponse::success_with_message(
        "配置保存成功".to_string(),
        None,
    ))
}

/// 获取当前配置
///
/// # 参数
/// * `state` - 配置管理器状态
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn get_config(state: State<'_, ConfigManagerState>) -> Result<ConfigResponse, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let config = manager.config().clone();

    Ok(ConfigResponse::success(Some(config)))
}

/// 更新配置
///
/// # 参数
/// * `state` - 配置管理器状态
/// * `config` - 新的验证配置
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn update_config(
    state: State<'_, ConfigManagerState>,
    config: ValidationConfig,
) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    // 验证配置
    config.validate().map_err(|e| e.to_string())?;

    // 更新配置
    *manager.config_mut() = config.clone();

    Ok(ConfigResponse::success_with_message(
        "配置更新成功".to_string(),
        Some(config),
    ))
}

/// 添加规则文件路径
///
/// # 参数
/// * `state` - 配置管理器状态
/// * `path` - 规则文件路径
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn add_rule_path(
    state: State<'_, ConfigManagerState>,
    path: String,
) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    let path_buf = PathBuf::from(path);
    manager.config_mut().add_rule_path(path_buf);

    let config = manager.config().clone();
    Ok(ConfigResponse::success_with_message(
        "规则路径添加成功".to_string(),
        Some(config),
    ))
}

/// 移除规则文件路径
///
/// # 参数
/// * `state` - 配置管理器状态
/// * `path` - 规则文件路径
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn remove_rule_path(
    state: State<'_, ConfigManagerState>,
    path: String,
) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    let path_buf = PathBuf::from(path);
    manager.config_mut().remove_rule_path(&path_buf);

    let config = manager.config().clone();
    Ok(ConfigResponse::success_with_message(
        "规则路径移除成功".to_string(),
        Some(config),
    ))
}

/// 配置规则
///
/// # 参数
/// * `state` - 配置管理器状态
/// * `request` - 规则配置请求
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn configure_rule(
    state: State<'_, ConfigManagerState>,
    request: RuleConfigRequest,
) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    let config = manager.config_mut();

    // 设置启用状态
    if let Some(enabled) = request.enabled {
        if enabled {
            config.enable_rule(request.rule_name.clone());
        } else {
            config.disable_rule(request.rule_name.clone());
        }
    }

    // 设置严重程度
    if let Some(severity) = request.severity {
        config.set_rule_severity(request.rule_name.clone(), severity);
    }

    let config = manager.config().clone();
    Ok(ConfigResponse::success_with_message(
        format!("规则 {} 配置成功", request.rule_name),
        Some(config),
    ))
}

/// 启用规则
///
/// # 参数
/// * `state` - 配置管理器状态
/// * `rule_name` - 规则名称
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn enable_rule(
    state: State<'_, ConfigManagerState>,
    rule_name: String,
) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    manager.config_mut().enable_rule(rule_name.clone());

    let config = manager.config().clone();
    Ok(ConfigResponse::success_with_message(
        format!("规则 {} 已启用", rule_name),
        Some(config),
    ))
}

/// 禁用规则
///
/// # 参数
/// * `state` - 配置管理器状态
/// * `rule_name` - 规则名称
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn disable_rule(
    state: State<'_, ConfigManagerState>,
    rule_name: String,
) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    manager.config_mut().disable_rule(rule_name.clone());

    let config = manager.config().clone();
    Ok(ConfigResponse::success_with_message(
        format!("规则 {} 已禁用", rule_name),
        Some(config),
    ))
}

/// 设置规则严重程度
///
/// # 参数
/// * `state` - 配置管理器状态
/// * `rule_name` - 规则名称
/// * `severity` - 严重程度
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn set_rule_severity(
    state: State<'_, ConfigManagerState>,
    rule_name: String,
    severity: Severity,
) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    manager
        .config_mut()
        .set_rule_severity(rule_name.clone(), severity);

    let config = manager.config().clone();
    Ok(ConfigResponse::success_with_message(
        format!("规则 {} 严重程度已设置", rule_name),
        Some(config),
    ))
}

/// 导出配置为 JSON
///
/// # 参数
/// * `state` - 配置管理器状态
///
/// # 返回
/// JSON 字符串
#[tauri::command]
pub async fn export_config_json(state: State<'_, ConfigManagerState>) -> Result<String, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;

    manager.export_json().map_err(|e| e.to_string())
}

/// 从 JSON 导入配置
///
/// # 参数
/// * `state` - 配置管理器状态
/// * `json` - JSON 字符串
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn import_config_json(
    state: State<'_, ConfigManagerState>,
    json: String,
) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    manager.import_json(&json).map_err(|e| e.to_string())?;

    let config = manager.config().clone();
    Ok(ConfigResponse::success_with_message(
        "配置导入成功".to_string(),
        Some(config),
    ))
}

/// 重置配置为默认值
///
/// # 参数
/// * `state` - 配置管理器状态
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn reset_config(state: State<'_, ConfigManagerState>) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    manager.reset();

    let config = manager.config().clone();
    Ok(ConfigResponse::success_with_message(
        "配置已重置为默认值".to_string(),
        Some(config),
    ))
}

/// 重新加载配置
///
/// # 参数
/// * `state` - 配置管理器状态
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn reload_config(state: State<'_, ConfigManagerState>) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    manager.reload().map_err(|e| e.to_string())?;

    let config = manager.config().clone();
    Ok(ConfigResponse::success_with_message(
        "配置重新加载成功".to_string(),
        Some(config),
    ))
}

/// 禁用规则类型
///
/// # 参数
/// * `state` - 配置管理器状态
/// * `rule_type` - 规则类型
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn disable_rule_type(
    state: State<'_, ConfigManagerState>,
    rule_type: String,
) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    manager.config_mut().disable_rule_type(rule_type.clone());

    let config = manager.config().clone();
    Ok(ConfigResponse::success_with_message(
        format!("规则类型 {} 已禁用", rule_type),
        Some(config),
    ))
}

/// 启用规则类型
///
/// # 参数
/// * `state` - 配置管理器状态
/// * `rule_type` - 规则类型
///
/// # 返回
/// 配置响应
#[tauri::command]
pub async fn enable_rule_type(
    state: State<'_, ConfigManagerState>,
    rule_type: String,
) -> Result<ConfigResponse, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    manager.config_mut().enable_rule_type(&rule_type);

    let config = manager.config().clone();
    Ok(ConfigResponse::success_with_message(
        format!("规则类型 {} 已启用", rule_type),
        Some(config),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_manager_state_creation() {
        let state = ConfigManagerState::new();
        let manager = state.manager.lock().unwrap();
        assert!(manager.config().rule_paths.is_empty());
    }

    #[test]
    fn test_config_response_success() {
        let response = ConfigResponse::success(None);
        assert!(response.success);
        assert!(response.message.is_none());
        assert!(response.config.is_none());
    }

    #[test]
    fn test_config_response_error() {
        let response = ConfigResponse::error("Test error".to_string());
        assert!(!response.success);
        assert_eq!(response.message, Some("Test error".to_string()));
        assert!(response.config.is_none());
    }
}
