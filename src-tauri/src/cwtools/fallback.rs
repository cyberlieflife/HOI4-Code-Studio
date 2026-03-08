//! 降级模式管理
//!
//! 当系统遇到严重错误时，提供降级模式以确保基本功能可用

use crate::cwtools::error_logger::{ErrorLogger, LogLevel};
use crate::cwtools::rules::types::{
    FieldType, Rule, RuleOptions, RuleSet, RuleType, TypeDefinition,
};
use std::sync::{Arc, Mutex};

/// 降级模式类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackMode {
    /// 正常模式 - 使用完整的规则集
    Normal,
    /// 基础模式 - 使用简化的规则集
    Basic,
    /// 最小模式 - 只进行基本的语法检查
    Minimal,
    /// 禁用模式 - 不进行验证
    Disabled,
}

impl FallbackMode {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            FallbackMode::Normal => "normal",
            FallbackMode::Basic => "basic",
            FallbackMode::Minimal => "minimal",
            FallbackMode::Disabled => "disabled",
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "normal" => Some(FallbackMode::Normal),
            "basic" => Some(FallbackMode::Basic),
            "minimal" => Some(FallbackMode::Minimal),
            "disabled" => Some(FallbackMode::Disabled),
            _ => None,
        }
    }
}

impl std::fmt::Display for FallbackMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 降级模式管理器
///
/// 管理系统的降级模式，在遇到错误时自动切换到降级模式
pub struct FallbackManager {
    /// 当前模式
    current_mode: Arc<Mutex<FallbackMode>>,
    /// 错误日志记录器
    logger: Arc<ErrorLogger>,
    /// 错误计数器
    error_count: Arc<Mutex<usize>>,
    /// 错误阈值（超过此值自动降级）
    error_threshold: usize,
    /// 是否启用自动降级
    auto_fallback: bool,
}

impl FallbackManager {
    /// 创建新的降级模式管理器
    ///
    /// # 参数
    /// * `logger` - 错误日志记录器
    /// * `error_threshold` - 错误阈值
    /// * `auto_fallback` - 是否启用自动降级
    ///
    /// # 返回
    /// FallbackManager 实例
    pub fn new(logger: Arc<ErrorLogger>, error_threshold: usize, auto_fallback: bool) -> Self {
        Self {
            current_mode: Arc::new(Mutex::new(FallbackMode::Normal)),
            logger,
            error_count: Arc::new(Mutex::new(0)),
            error_threshold,
            auto_fallback,
        }
    }

    /// 创建默认的降级模式管理器
    pub fn default_with_logger(logger: Arc<ErrorLogger>) -> Self {
        Self::new(logger, 10, true)
    }

    /// 获取当前模式
    pub fn current_mode(&self) -> FallbackMode {
        *self.current_mode.lock().unwrap()
    }

    /// 设置模式
    ///
    /// # 参数
    /// * `mode` - 新的模式
    pub fn set_mode(&self, mode: FallbackMode) {
        let mut current = self.current_mode.lock().unwrap();
        if *current != mode {
            self.logger
                .log_info("fallback", format!("切换降级模式: {} -> {}", current, mode));
            *current = mode;
        }
    }

    /// 记录错误并检查是否需要降级
    ///
    /// # 参数
    /// * `error_message` - 错误消息
    pub fn record_error(&self, error_message: impl Into<String>) {
        let mut count = self.error_count.lock().unwrap();
        *count += 1;

        let message = error_message.into();
        self.logger
            .log_error("fallback", format!("错误 #{}: {}", *count, message));

        // 检查是否需要自动降级
        if self.auto_fallback && *count >= self.error_threshold {
            let current = self.current_mode();
            match current {
                FallbackMode::Normal => {
                    self.set_mode(FallbackMode::Basic);
                    self.logger.log_warning(
                        "fallback",
                        format!(
                            "错误次数达到阈值 {}，自动降级到基础模式",
                            self.error_threshold
                        ),
                    );
                }
                FallbackMode::Basic => {
                    self.set_mode(FallbackMode::Minimal);
                    self.logger.log_warning(
                        "fallback",
                        format!(
                            "错误次数达到阈值 {}，自动降级到最小模式",
                            self.error_threshold
                        ),
                    );
                }
                FallbackMode::Minimal => {
                    self.set_mode(FallbackMode::Disabled);
                    self.logger.log_warning(
                        "fallback",
                        format!("错误次数达到阈值 {}，自动禁用验证", self.error_threshold),
                    );
                }
                FallbackMode::Disabled => {
                    // 已经是最低级别，不再降级
                }
            }

            // 重置错误计数
            *count = 0;
        }
    }

    /// 重置错误计数
    pub fn reset_error_count(&self) {
        let mut count = self.error_count.lock().unwrap();
        *count = 0;
    }

    /// 获取当前错误计数
    pub fn error_count(&self) -> usize {
        *self.error_count.lock().unwrap()
    }

    /// 尝试恢复到正常模式
    ///
    /// 如果当前处于降级模式，尝试恢复到正常模式
    pub fn try_recover(&self) {
        let current = self.current_mode();
        if current != FallbackMode::Normal {
            self.set_mode(FallbackMode::Normal);
            self.reset_error_count();
            self.logger.log_info("fallback", "尝试恢复到正常模式");
        }
    }

    /// 创建降级规则集
    ///
    /// 根据当前模式创建相应的规则集
    ///
    /// # 返回
    /// 降级规则集
    pub fn create_fallback_ruleset(&self) -> RuleSet {
        let mode = self.current_mode();

        match mode {
            FallbackMode::Normal => {
                // 正常模式不应该调用此方法
                self.logger
                    .log_warning("fallback", "在正常模式下创建降级规则集");
                RuleSet::new()
            }
            FallbackMode::Basic => self.create_basic_ruleset(),
            FallbackMode::Minimal => self.create_minimal_ruleset(),
            FallbackMode::Disabled => {
                // 禁用模式返回空规则集
                RuleSet::new()
            }
        }
    }

    /// 创建基础规则集
    ///
    /// 包含最常用的 HOI4 脚本结构的基本规则
    fn create_basic_ruleset(&self) -> RuleSet {
        let mut ruleset = RuleSet::new();

        // 添加基本的类型定义
        // country_event
        let mut country_event = TypeDefinition::new("country_event".to_string());
        country_event.add_rule(Rule::new(
            RuleType::LeafRule {
                left: FieldType::Specific("id".to_string()),
                right: FieldType::Scalar,
            },
            RuleOptions::default(),
        ));
        country_event.add_rule(Rule::new(
            RuleType::LeafRule {
                left: FieldType::Specific("title".to_string()),
                right: FieldType::Scalar,
            },
            RuleOptions::default(),
        ));
        country_event.add_rule(Rule::new(
            RuleType::LeafRule {
                left: FieldType::Specific("desc".to_string()),
                right: FieldType::Scalar,
            },
            RuleOptions::default(),
        ));
        ruleset.add_type("country_event".to_string(), country_event);

        // focus_tree
        let mut focus_tree = TypeDefinition::new("focus_tree".to_string());
        focus_tree.add_rule(Rule::new(
            RuleType::LeafRule {
                left: FieldType::Specific("id".to_string()),
                right: FieldType::Scalar,
            },
            RuleOptions::default(),
        ));
        ruleset.add_type("focus_tree".to_string(), focus_tree);

        // idea
        let mut idea = TypeDefinition::new("idea".to_string());
        idea.add_rule(Rule::new(
            RuleType::LeafRule {
                left: FieldType::Specific("name".to_string()),
                right: FieldType::Scalar,
            },
            RuleOptions::default(),
        ));
        ruleset.add_type("idea".to_string(), idea);

        self.logger.log_info("fallback", "创建基础规则集");
        ruleset
    }

    /// 创建最小规则集
    ///
    /// 只包含最基本的语法验证规则
    fn create_minimal_ruleset(&self) -> RuleSet {
        let ruleset = RuleSet::new();
        self.logger
            .log_info("fallback", "创建最小规则集（空规则集）");
        ruleset
    }

    /// 检查是否应该使用降级规则集
    ///
    /// # 返回
    /// 如果当前模式不是正常模式，返回 true
    pub fn should_use_fallback(&self) -> bool {
        self.current_mode() != FallbackMode::Normal
    }

    /// 获取降级模式的描述
    pub fn mode_description(&self) -> &'static str {
        match self.current_mode() {
            FallbackMode::Normal => "正常模式 - 使用完整的规则集进行验证",
            FallbackMode::Basic => "基础模式 - 使用简化的规则集，只验证常见结构",
            FallbackMode::Minimal => "最小模式 - 只进行基本的语法检查",
            FallbackMode::Disabled => "禁用模式 - 不进行验证",
        }
    }

    /// 设置错误阈值
    pub fn set_error_threshold(&mut self, threshold: usize) {
        self.error_threshold = threshold;
    }

    /// 设置是否启用自动降级
    pub fn set_auto_fallback(&mut self, enabled: bool) {
        self.auto_fallback = enabled;
    }

    /// 获取错误阈值
    pub fn error_threshold(&self) -> usize {
        self.error_threshold
    }

    /// 是否启用自动降级
    pub fn is_auto_fallback_enabled(&self) -> bool {
        self.auto_fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_logger() -> Arc<ErrorLogger> {
        Arc::new(ErrorLogger::new(None, LogLevel::Debug))
    }

    #[test]
    fn test_fallback_mode_from_str() {
        assert_eq!(FallbackMode::from_str("normal"), Some(FallbackMode::Normal));
        assert_eq!(FallbackMode::from_str("basic"), Some(FallbackMode::Basic));
        assert_eq!(
            FallbackMode::from_str("minimal"),
            Some(FallbackMode::Minimal)
        );
        assert_eq!(
            FallbackMode::from_str("disabled"),
            Some(FallbackMode::Disabled)
        );
        assert_eq!(FallbackMode::from_str("unknown"), None);
    }

    #[test]
    fn test_fallback_mode_display() {
        assert_eq!(FallbackMode::Normal.to_string(), "normal");
        assert_eq!(FallbackMode::Basic.to_string(), "basic");
        assert_eq!(FallbackMode::Minimal.to_string(), "minimal");
        assert_eq!(FallbackMode::Disabled.to_string(), "disabled");
    }

    #[test]
    fn test_fallback_manager_creation() {
        let logger = create_test_logger();
        let manager = FallbackManager::new(logger, 10, true);

        assert_eq!(manager.current_mode(), FallbackMode::Normal);
        assert_eq!(manager.error_count(), 0);
        assert_eq!(manager.error_threshold(), 10);
        assert!(manager.is_auto_fallback_enabled());
    }

    #[test]
    fn test_set_mode() {
        let logger = create_test_logger();
        let manager = FallbackManager::new(logger, 10, true);

        assert_eq!(manager.current_mode(), FallbackMode::Normal);

        manager.set_mode(FallbackMode::Basic);
        assert_eq!(manager.current_mode(), FallbackMode::Basic);

        manager.set_mode(FallbackMode::Minimal);
        assert_eq!(manager.current_mode(), FallbackMode::Minimal);
    }

    #[test]
    fn test_record_error_without_auto_fallback() {
        let logger = create_test_logger();
        let manager = FallbackManager::new(logger, 10, false);

        for i in 0..15 {
            manager.record_error(format!("Error {}", i));
        }

        // 不应该自动降级
        assert_eq!(manager.current_mode(), FallbackMode::Normal);
        assert_eq!(manager.error_count(), 15);
    }

    #[test]
    fn test_record_error_with_auto_fallback() {
        let logger = create_test_logger();
        let manager = FallbackManager::new(logger, 5, true);

        // 记录 5 个错误，应该降级到 Basic
        for i in 0..5 {
            manager.record_error(format!("Error {}", i));
        }
        assert_eq!(manager.current_mode(), FallbackMode::Basic);
        assert_eq!(manager.error_count(), 0); // 错误计数应该被重置

        // 再记录 5 个错误，应该降级到 Minimal
        for i in 0..5 {
            manager.record_error(format!("Error {}", i));
        }
        assert_eq!(manager.current_mode(), FallbackMode::Minimal);

        // 再记录 5 个错误，应该降级到 Disabled
        for i in 0..5 {
            manager.record_error(format!("Error {}", i));
        }
        assert_eq!(manager.current_mode(), FallbackMode::Disabled);
    }

    #[test]
    fn test_reset_error_count() {
        let logger = create_test_logger();
        let manager = FallbackManager::new(logger, 10, false);

        manager.record_error("Error 1");
        manager.record_error("Error 2");
        assert_eq!(manager.error_count(), 2);

        manager.reset_error_count();
        assert_eq!(manager.error_count(), 0);
    }

    #[test]
    fn test_try_recover() {
        let logger = create_test_logger();
        let manager = FallbackManager::new(logger, 10, true);

        manager.set_mode(FallbackMode::Basic);
        assert_eq!(manager.current_mode(), FallbackMode::Basic);

        manager.try_recover();
        assert_eq!(manager.current_mode(), FallbackMode::Normal);
        assert_eq!(manager.error_count(), 0);
    }

    #[test]
    fn test_should_use_fallback() {
        let logger = create_test_logger();
        let manager = FallbackManager::new(logger, 10, true);

        assert!(!manager.should_use_fallback());

        manager.set_mode(FallbackMode::Basic);
        assert!(manager.should_use_fallback());

        manager.set_mode(FallbackMode::Minimal);
        assert!(manager.should_use_fallback());

        manager.set_mode(FallbackMode::Disabled);
        assert!(manager.should_use_fallback());
    }

    #[test]
    fn test_create_fallback_ruleset() {
        let logger = create_test_logger();
        let manager = FallbackManager::new(logger, 10, true);

        // 基础模式
        manager.set_mode(FallbackMode::Basic);
        let basic_ruleset = manager.create_fallback_ruleset();
        assert!(!basic_ruleset.types.is_empty());

        // 最小模式
        manager.set_mode(FallbackMode::Minimal);
        let minimal_ruleset = manager.create_fallback_ruleset();
        assert!(minimal_ruleset.types.is_empty());

        // 禁用模式
        manager.set_mode(FallbackMode::Disabled);
        let disabled_ruleset = manager.create_fallback_ruleset();
        assert!(disabled_ruleset.types.is_empty());
    }

    #[test]
    fn test_mode_description() {
        let logger = create_test_logger();
        let manager = FallbackManager::new(logger, 10, true);

        assert!(manager.mode_description().contains("正常模式"));

        manager.set_mode(FallbackMode::Basic);
        assert!(manager.mode_description().contains("基础模式"));

        manager.set_mode(FallbackMode::Minimal);
        assert!(manager.mode_description().contains("最小模式"));

        manager.set_mode(FallbackMode::Disabled);
        assert!(manager.mode_description().contains("禁用模式"));
    }

    #[test]
    fn test_set_error_threshold() {
        let logger = create_test_logger();
        let mut manager = FallbackManager::new(logger, 10, true);

        assert_eq!(manager.error_threshold(), 10);

        manager.set_error_threshold(20);
        assert_eq!(manager.error_threshold(), 20);
    }

    #[test]
    fn test_set_auto_fallback() {
        let logger = create_test_logger();
        let mut manager = FallbackManager::new(logger, 10, true);

        assert!(manager.is_auto_fallback_enabled());

        manager.set_auto_fallback(false);
        assert!(!manager.is_auto_fallback_enabled());
    }

    #[test]
    fn test_default_with_logger() {
        let logger = create_test_logger();
        let manager = FallbackManager::default_with_logger(logger);

        assert_eq!(manager.current_mode(), FallbackMode::Normal);
        assert_eq!(manager.error_threshold(), 10);
        assert!(manager.is_auto_fallback_enabled());
    }
}
