//! 配置管理模块
//!
//! 提供 cwtools 验证系统的配置管理功能

use crate::cwtools::diagnostic::Severity;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// 配置错误类型
#[derive(Debug)]
pub enum ConfigError {
    /// IO 错误
    IoError(std::io::Error),
    /// 序列化/反序列化错误
    SerdeError(serde_json::Error),
    /// 配置验证错误
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(err) => write!(f, "IO 错误: {}", err),
            ConfigError::SerdeError(err) => write!(f, "序列化错误: {}", err),
            ConfigError::ValidationError(msg) => write!(f, "配置验证错误: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::SerdeError(err)
    }
}

/// 规则配置
///
/// 控制特定规则的启用状态和严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    /// 规则是否启用
    pub enabled: bool,
    /// 自定义严重程度（如果为 None，使用规则默认值）
    pub severity: Option<Severity>,
    /// 规则描述（可选）
    pub description: Option<String>,
}

impl RuleConfig {
    /// 创建启用的规则配置
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            severity: None,
            description: None,
        }
    }

    /// 创建禁用的规则配置
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            severity: None,
            description: None,
        }
    }

    /// 创建带有自定义严重程度的规则配置
    pub fn with_severity(severity: Severity) -> Self {
        Self {
            enabled: true,
            severity: Some(severity),
            description: None,
        }
    }

    /// 设置规则描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self::enabled()
    }
}

/// 验证配置
///
/// 包含所有验证相关的配置选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// 规则文件路径列表
    pub rule_paths: Vec<PathBuf>,

    /// 规则配置映射（规则名称 -> 规则配置）
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,

    /// 全局禁用的规则类型集合
    #[serde(default)]
    pub disabled_rule_types: HashSet<String>,

    /// 默认错误严重程度
    #[serde(default = "default_severity")]
    pub default_severity: Severity,

    /// 是否启用引用检查
    #[serde(default = "default_true")]
    pub enable_reference_check: bool,

    /// 是否启用作用域检查
    #[serde(default = "default_true")]
    pub enable_scope_check: bool,

    /// 是否启用修饰符检查
    #[serde(default = "default_true")]
    pub enable_modifier_check: bool,

    /// 项目根目录
    pub project_root: Option<PathBuf>,

    /// 游戏根目录
    pub game_root: Option<PathBuf>,

    /// 缓存配置
    #[serde(default)]
    pub cache: CacheConfig,
}

fn default_severity() -> Severity {
    Severity::Error
}

fn default_true() -> bool {
    true
}

impl ValidationConfig {
    /// 创建新的验证配置
    pub fn new() -> Self {
        Self {
            rule_paths: Vec::new(),
            rules: HashMap::new(),
            disabled_rule_types: HashSet::new(),
            default_severity: Severity::Error,
            enable_reference_check: true,
            enable_scope_check: true,
            enable_modifier_check: true,
            project_root: None,
            game_root: None,
            cache: CacheConfig::default(),
        }
    }

    /// 添加规则文件路径
    pub fn add_rule_path(&mut self, path: PathBuf) {
        if !self.rule_paths.contains(&path) {
            self.rule_paths.push(path);
        }
    }

    /// 移除规则文件路径
    pub fn remove_rule_path(&mut self, path: &Path) {
        self.rule_paths.retain(|p| p != path);
    }

    /// 启用规则
    pub fn enable_rule(&mut self, rule_name: String) {
        self.rules
            .entry(rule_name)
            .or_insert_with(RuleConfig::enabled)
            .enabled = true;
    }

    /// 禁用规则
    pub fn disable_rule(&mut self, rule_name: String) {
        self.rules
            .entry(rule_name)
            .or_insert_with(RuleConfig::disabled)
            .enabled = false;
    }

    /// 设置规则的严重程度
    pub fn set_rule_severity(&mut self, rule_name: String, severity: Severity) {
        self.rules
            .entry(rule_name)
            .or_insert_with(RuleConfig::enabled)
            .severity = Some(severity);
    }

    /// 检查规则是否启用
    pub fn is_rule_enabled(&self, rule_name: &str) -> bool {
        self.rules
            .get(rule_name)
            .map(|config| config.enabled)
            .unwrap_or(true) // 默认启用
    }

    /// 获取规则的严重程度
    pub fn get_rule_severity(&self, rule_name: &str) -> Option<Severity> {
        self.rules.get(rule_name).and_then(|config| config.severity)
    }

    /// 禁用规则类型
    pub fn disable_rule_type(&mut self, rule_type: String) {
        self.disabled_rule_types.insert(rule_type);
    }

    /// 启用规则类型
    pub fn enable_rule_type(&mut self, rule_type: &str) {
        self.disabled_rule_types.remove(rule_type);
    }

    /// 检查规则类型是否启用
    pub fn is_rule_type_enabled(&self, rule_type: &str) -> bool {
        !self.disabled_rule_types.contains(rule_type)
    }

    /// 从文件加载配置
    pub fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: ValidationConfig = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: &Path) -> Result<(), ConfigError> {
        self.validate()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), ConfigError> {
        // 检查规则文件路径是否存在
        for path in &self.rule_paths {
            if !path.exists() {
                return Err(ConfigError::ValidationError(format!(
                    "规则文件不存在: {}",
                    path.display()
                )));
            }
        }

        // 检查项目根目录
        if let Some(ref project_root) = self.project_root {
            if !project_root.exists() {
                return Err(ConfigError::ValidationError(format!(
                    "项目根目录不存在: {}",
                    project_root.display()
                )));
            }
        }

        // 检查游戏根目录
        if let Some(ref game_root) = self.game_root {
            if !game_root.exists() {
                return Err(ConfigError::ValidationError(format!(
                    "游戏根目录不存在: {}",
                    game_root.display()
                )));
            }
        }

        Ok(())
    }

    /// 导出配置为 JSON 字符串
    pub fn export_json(&self) -> Result<String, ConfigError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// 从 JSON 字符串导入配置
    pub fn import_json(json: &str) -> Result<Self, ConfigError> {
        let config: ValidationConfig = serde_json::from_str(json)?;
        config.validate()?;
        Ok(config)
    }

    /// 合并另一个配置
    ///
    /// 将另一个配置的设置合并到当前配置中
    pub fn merge(&mut self, other: ValidationConfig) {
        // 合并规则路径
        for path in other.rule_paths {
            self.add_rule_path(path);
        }

        // 合并规则配置
        for (rule_name, rule_config) in other.rules {
            self.rules.insert(rule_name, rule_config);
        }

        // 合并禁用的规则类型
        for rule_type in other.disabled_rule_types {
            self.disabled_rule_types.insert(rule_type);
        }

        // 更新其他配置（如果提供）
        if other.project_root.is_some() {
            self.project_root = other.project_root;
        }
        if other.game_root.is_some() {
            self.game_root = other.game_root;
        }

        // 合并缓存配置
        self.cache.merge(other.cache);
    }

    /// 重置为默认配置
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 是否启用缓存
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// 最大缓存条目数
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,

    /// 最大内存使用（字节）
    #[serde(default = "default_max_memory")]
    pub max_memory_bytes: usize,

    /// 缓存过期时间（秒）
    #[serde(default = "default_cache_ttl")]
    pub ttl_seconds: u64,
}

fn default_max_entries() -> usize {
    100
}

fn default_max_memory() -> usize {
    500 * 1024 * 1024 // 500 MB
}

fn default_cache_ttl() -> u64 {
    3600 // 1 小时
}

impl CacheConfig {
    /// 创建新的缓存配置
    pub fn new() -> Self {
        Self {
            enabled: true,
            max_entries: 100,
            max_memory_bytes: 500 * 1024 * 1024,
            ttl_seconds: 3600,
        }
    }

    /// 合并另一个缓存配置
    pub fn merge(&mut self, other: CacheConfig) {
        self.enabled = other.enabled;
        self.max_entries = other.max_entries;
        self.max_memory_bytes = other.max_memory_bytes;
        self.ttl_seconds = other.ttl_seconds;
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 配置管理器
///
/// 管理验证配置的加载、保存和更新
pub struct ConfigManager {
    /// 当前配置
    config: ValidationConfig,
    /// 配置文件路径
    config_path: Option<PathBuf>,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Self {
        Self {
            config: ValidationConfig::new(),
            config_path: None,
        }
    }

    /// 从文件加载配置
    pub fn load(&mut self, path: &Path) -> Result<(), ConfigError> {
        self.config = ValidationConfig::load_from_file(path)?;
        self.config_path = Some(path.to_path_buf());
        Ok(())
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(ref path) = self.config_path {
            self.config.save_to_file(path)
        } else {
            Err(ConfigError::ValidationError(
                "未设置配置文件路径".to_string(),
            ))
        }
    }

    /// 保存配置到指定文件
    pub fn save_as(&mut self, path: &Path) -> Result<(), ConfigError> {
        self.config.save_to_file(path)?;
        self.config_path = Some(path.to_path_buf());
        Ok(())
    }

    /// 获取当前配置的引用
    pub fn config(&self) -> &ValidationConfig {
        &self.config
    }

    /// 获取当前配置的可变引用
    pub fn config_mut(&mut self) -> &mut ValidationConfig {
        &mut self.config
    }

    /// 重新加载配置
    pub fn reload(&mut self) -> Result<(), ConfigError> {
        if let Some(ref path) = self.config_path.clone() {
            self.load(path)
        } else {
            Err(ConfigError::ValidationError(
                "未设置配置文件路径".to_string(),
            ))
        }
    }

    /// 导出配置为 JSON
    pub fn export_json(&self) -> Result<String, ConfigError> {
        self.config.export_json()
    }

    /// 从 JSON 导入配置
    pub fn import_json(&mut self, json: &str) -> Result<(), ConfigError> {
        self.config = ValidationConfig::import_json(json)?;
        Ok(())
    }

    /// 重置配置为默认值
    pub fn reset(&mut self) {
        self.config.reset();
    }

    /// 获取配置文件路径
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_rule_config_creation() {
        let enabled = RuleConfig::enabled();
        assert!(enabled.enabled);
        assert!(enabled.severity.is_none());

        let disabled = RuleConfig::disabled();
        assert!(!disabled.enabled);

        let with_severity = RuleConfig::with_severity(Severity::Warning);
        assert!(with_severity.enabled);
        assert_eq!(with_severity.severity, Some(Severity::Warning));
    }

    #[test]
    fn test_rule_config_with_description() {
        let config = RuleConfig::enabled().with_description("Test rule".to_string());
        assert_eq!(config.description, Some("Test rule".to_string()));
    }

    #[test]
    fn test_validation_config_creation() {
        let config = ValidationConfig::new();
        assert!(config.rule_paths.is_empty());
        assert!(config.rules.is_empty());
        assert!(config.enable_reference_check);
        assert!(config.enable_scope_check);
        assert!(config.enable_modifier_check);
    }

    #[test]
    fn test_add_remove_rule_path() {
        let mut config = ValidationConfig::new();
        let path = PathBuf::from("test.cwt");

        config.add_rule_path(path.clone());
        assert_eq!(config.rule_paths.len(), 1);

        // 添加重复路径不应增加数量
        config.add_rule_path(path.clone());
        assert_eq!(config.rule_paths.len(), 1);

        config.remove_rule_path(&path);
        assert_eq!(config.rule_paths.len(), 0);
    }

    #[test]
    fn test_enable_disable_rule() {
        let mut config = ValidationConfig::new();

        config.disable_rule("test_rule".to_string());
        assert!(!config.is_rule_enabled("test_rule"));

        config.enable_rule("test_rule".to_string());
        assert!(config.is_rule_enabled("test_rule"));

        // 未配置的规则默认启用
        assert!(config.is_rule_enabled("unknown_rule"));
    }

    #[test]
    fn test_set_rule_severity() {
        let mut config = ValidationConfig::new();

        config.set_rule_severity("test_rule".to_string(), Severity::Warning);
        assert_eq!(
            config.get_rule_severity("test_rule"),
            Some(Severity::Warning)
        );

        // 未配置的规则返回 None
        assert_eq!(config.get_rule_severity("unknown_rule"), None);
    }

    #[test]
    fn test_disable_enable_rule_type() {
        let mut config = ValidationConfig::new();

        config.disable_rule_type("type_check".to_string());
        assert!(!config.is_rule_type_enabled("type_check"));

        config.enable_rule_type("type_check");
        assert!(config.is_rule_type_enabled("type_check"));
    }

    #[test]
    fn test_save_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let mut config = ValidationConfig::new();
        config.enable_reference_check = false;
        config.default_severity = Severity::Warning;

        // 保存配置
        config.save_to_file(&config_path).unwrap();
        assert!(config_path.exists());

        // 加载配置
        let loaded_config = ValidationConfig::load_from_file(&config_path).unwrap();
        assert!(!loaded_config.enable_reference_check);
        assert_eq!(loaded_config.default_severity, Severity::Warning);
    }

    #[test]
    fn test_export_import_json() {
        let mut config = ValidationConfig::new();
        config.enable_reference_check = false;
        config.default_severity = Severity::Warning;

        // 导出为 JSON
        let json = config.export_json().unwrap();
        assert!(json.contains("enable_reference_check"));

        // 从 JSON 导入
        let imported_config = ValidationConfig::import_json(&json).unwrap();
        assert!(!imported_config.enable_reference_check);
        assert_eq!(imported_config.default_severity, Severity::Warning);
    }

    #[test]
    fn test_config_merge() {
        let mut config1 = ValidationConfig::new();
        config1.add_rule_path(PathBuf::from("rule1.cwt"));
        config1.disable_rule("rule1".to_string());

        let mut config2 = ValidationConfig::new();
        config2.add_rule_path(PathBuf::from("rule2.cwt"));
        config2.disable_rule("rule2".to_string());

        config1.merge(config2);

        assert_eq!(config1.rule_paths.len(), 2);
        assert!(!config1.is_rule_enabled("rule1"));
        assert!(!config1.is_rule_enabled("rule2"));
    }

    #[test]
    fn test_config_reset() {
        let mut config = ValidationConfig::new();
        config.enable_reference_check = false;
        config.add_rule_path(PathBuf::from("test.cwt"));

        config.reset();

        assert!(config.enable_reference_check);
        assert!(config.rule_paths.is_empty());
    }

    #[test]
    fn test_cache_config() {
        let cache_config = CacheConfig::new();
        assert!(cache_config.enabled);
        assert_eq!(cache_config.max_entries, 100);
        assert_eq!(cache_config.max_memory_bytes, 500 * 1024 * 1024);
        assert_eq!(cache_config.ttl_seconds, 3600);
    }

    #[test]
    fn test_cache_config_merge() {
        let mut config1 = CacheConfig::new();
        let mut config2 = CacheConfig::new();
        config2.max_entries = 200;
        config2.enabled = false;

        config1.merge(config2);

        assert!(!config1.enabled);
        assert_eq!(config1.max_entries, 200);
    }

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        assert!(manager.config_path().is_none());
        assert!(manager.config().rule_paths.is_empty());
    }

    #[test]
    fn test_config_manager_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let mut manager = ConfigManager::new();
        manager.config_mut().enable_reference_check = false;

        // 保存配置
        manager.save_as(&config_path).unwrap();
        assert!(config_path.exists());

        // 创建新的管理器并加载
        let mut new_manager = ConfigManager::new();
        new_manager.load(&config_path).unwrap();
        assert!(!new_manager.config().enable_reference_check);
    }

    #[test]
    fn test_config_manager_reload() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let mut manager = ConfigManager::new();
        manager.config_mut().enable_reference_check = false;
        manager.save_as(&config_path).unwrap();

        // 修改配置文件
        let mut config = ValidationConfig::load_from_file(&config_path).unwrap();
        config.enable_reference_check = true;
        config.save_to_file(&config_path).unwrap();

        // 重新加载
        manager.reload().unwrap();
        assert!(manager.config().enable_reference_check);
    }

    #[test]
    fn test_config_manager_export_import() {
        let mut manager = ConfigManager::new();
        manager.config_mut().enable_reference_check = false;

        // 导出
        let json = manager.export_json().unwrap();

        // 导入到新管理器
        let mut new_manager = ConfigManager::new();
        new_manager.import_json(&json).unwrap();
        assert!(!new_manager.config().enable_reference_check);
    }

    #[test]
    fn test_config_manager_reset() {
        let mut manager = ConfigManager::new();
        manager.config_mut().enable_reference_check = false;
        manager
            .config_mut()
            .add_rule_path(PathBuf::from("test.cwt"));

        manager.reset();

        assert!(manager.config().enable_reference_check);
        assert!(manager.config().rule_paths.is_empty());
    }

    #[test]
    fn test_config_validation_nonexistent_path() {
        let mut config = ValidationConfig::new();
        config.add_rule_path(PathBuf::from("/nonexistent/path/rule.cwt"));

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_validation_nonexistent_project_root() {
        let mut config = ValidationConfig::new();
        config.project_root = Some(PathBuf::from("/nonexistent/project"));

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::ValidationError("Test error".to_string());
        assert_eq!(error.to_string(), "配置验证错误: Test error");
    }

    #[test]
    fn test_config_manager_save_without_path() {
        let manager = ConfigManager::new();
        let result = manager.save();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_manager_reload_without_path() {
        let mut manager = ConfigManager::new();
        let result = manager.reload();
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_config_with_all_options() {
        let temp_dir = TempDir::new().unwrap();

        // 创建临时规则文件
        let rule_path = temp_dir.path().join("test.cwt");
        fs::write(&rule_path, "# test rule").unwrap();

        let mut config = ValidationConfig::new();
        config.add_rule_path(rule_path);
        config.disable_rule("test_rule".to_string());
        config.set_rule_severity("another_rule".to_string(), Severity::Warning);
        config.disable_rule_type("type_check".to_string());
        config.default_severity = Severity::Information;
        config.enable_reference_check = false;
        config.enable_scope_check = false;
        config.enable_modifier_check = false;
        config.project_root = Some(temp_dir.path().to_path_buf());
        config.game_root = Some(temp_dir.path().to_path_buf());
        config.cache.enabled = false;
        config.cache.max_entries = 50;

        // 验证配置
        assert!(config.validate().is_ok());

        // 保存并重新加载
        let config_path = temp_dir.path().join("full_config.json");
        config.save_to_file(&config_path).unwrap();

        let loaded_config = ValidationConfig::load_from_file(&config_path).unwrap();
        assert_eq!(loaded_config.rule_paths.len(), 1);
        assert!(!loaded_config.is_rule_enabled("test_rule"));
        assert_eq!(
            loaded_config.get_rule_severity("another_rule"),
            Some(Severity::Warning)
        );
        assert!(!loaded_config.is_rule_type_enabled("type_check"));
        assert_eq!(loaded_config.default_severity, Severity::Information);
        assert!(!loaded_config.enable_reference_check);
        assert!(!loaded_config.enable_scope_check);
        assert!(!loaded_config.enable_modifier_check);
        assert!(!loaded_config.cache.enabled);
        assert_eq!(loaded_config.cache.max_entries, 50);
    }
}
