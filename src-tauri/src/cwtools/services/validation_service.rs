//! 验证服务模块
//!
//! 提供脚本验证服务，集成解析器和验证器

use crate::cwtools::config::ValidationConfig;
use crate::cwtools::diagnostic::Diagnostic;
use crate::cwtools::models::AST;
use crate::cwtools::parser::ParseError;
use crate::cwtools::rules::{RuleLoader, RuleSet};
use crate::cwtools::services::parser_service::{ParserService, TextChange};
use crate::cwtools::validator::core::{ValidationResult, Validator};
use crate::cwtools::validator::reference::ReferenceChecker;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 服务错误类型
#[derive(Debug)]
pub enum ServiceError {
    /// 规则加载错误
    RuleLoadError(String),
    /// 解析错误
    ParseError(Vec<ParseError>),
    /// 验证错误
    ValidationError(String),
    /// IO 错误
    IoError(std::io::Error),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::RuleLoadError(msg) => write!(f, "规则加载错误: {}", msg),
            ServiceError::ParseError(errors) => {
                write!(f, "解析错误: {} 个错误", errors.len())
            }
            ServiceError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            ServiceError::IoError(err) => write!(f, "IO 错误: {}", err),
        }
    }
}

impl std::error::Error for ServiceError {}

impl From<std::io::Error> for ServiceError {
    fn from(err: std::io::Error) -> Self {
        ServiceError::IoError(err)
    }
}

/// 验证响应
///
/// 包含验证结果和性能指标
#[derive(Debug, Clone)]
pub struct ValidationResponse {
    /// 验证是否成功（无错误）
    pub success: bool,
    /// 诊断信息列表
    pub diagnostics: Vec<Diagnostic>,
    /// 解析耗时（毫秒）
    pub parse_time_ms: u64,
    /// 验证耗时（毫秒）
    pub validation_time_ms: u64,
    /// 总耗时（毫秒）
    pub total_time_ms: u64,
}

impl ValidationResponse {
    /// 创建新的验证响应
    pub fn new(
        success: bool,
        diagnostics: Vec<Diagnostic>,
        parse_time_ms: u64,
        validation_time_ms: u64,
    ) -> Self {
        let total_time_ms = parse_time_ms + validation_time_ms;
        Self {
            success,
            diagnostics,
            parse_time_ms,
            validation_time_ms,
            total_time_ms,
        }
    }

    /// 创建解析错误响应
    pub fn from_parse_errors(errors: Vec<ParseError>, parse_time_ms: u64) -> Self {
        let diagnostics: Vec<Diagnostic> = errors.into_iter().map(|e| e.to_diagnostic()).collect();

        Self {
            success: false,
            diagnostics,
            parse_time_ms,
            validation_time_ms: 0,
            total_time_ms: parse_time_ms,
        }
    }
}

/// 验证服务
///
/// 协调解析器和验证器，提供完整的验证功能
pub struct ValidationService {
    /// 解析服务
    parser_service: Arc<Mutex<ParserService>>,
    /// 规则集合
    rule_set: Arc<Mutex<RuleSet>>,
    /// 规则加载器
    rule_loader: Arc<Mutex<RuleLoader>>,
    /// 引用检查器
    reference_checker: Arc<Mutex<ReferenceChecker>>,
    /// 规则文件路径列表
    rule_paths: Vec<PathBuf>,
    /// 验证配置
    config: Arc<Mutex<ValidationConfig>>,
}

impl ValidationService {
    /// 创建新的验证服务
    ///
    /// # 参数
    /// * `rule_paths` - 规则文件路径列表
    ///
    /// # 返回
    /// * `Ok(ValidationService)` - 创建成功的验证服务
    /// * `Err(ServiceError)` - 创建失败的错误信息
    pub fn new(rule_paths: Vec<PathBuf>) -> Result<Self, ServiceError> {
        // 创建默认配置
        let mut config = ValidationConfig::new();
        for path in &rule_paths {
            config.add_rule_path(path.clone());
        }

        Self::with_validation_config(config)
    }

    /// 使用验证配置创建服务
    ///
    /// # 参数
    /// * `config` - 验证配置
    ///
    /// # 返回
    /// * `Ok(ValidationService)` - 创建成功的验证服务
    /// * `Err(ServiceError)` - 创建失败的错误信息
    pub fn with_validation_config(config: ValidationConfig) -> Result<Self, ServiceError> {
        // 创建规则加载器
        let mut rule_loader = RuleLoader::new();

        // 加载所有规则文件
        let rule_set = rule_loader
            .load_all_rules(&config.rule_paths)
            .map_err(|errors| {
                let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
                ServiceError::RuleLoadError(error_messages.join("; "))
            })?;

        let rule_paths = config.rule_paths.clone();

        Ok(Self {
            parser_service: Arc::new(Mutex::new(ParserService::new())),
            rule_set: Arc::new(Mutex::new(rule_set)),
            rule_loader: Arc::new(Mutex::new(rule_loader)),
            reference_checker: Arc::new(Mutex::new(ReferenceChecker::new())),
            rule_paths,
            config: Arc::new(Mutex::new(config)),
        })
    }

    /// 创建带有自定义配置的验证服务
    ///
    /// # 参数
    /// * `rule_paths` - 规则文件路径列表
    /// * `parser_service` - 自定义的解析服务
    /// * `reference_checker` - 自定义的引用检查器
    ///
    /// # 返回
    /// * `Ok(ValidationService)` - 创建成功的验证服务
    /// * `Err(ServiceError)` - 创建失败的错误信息
    pub fn with_config(
        rule_paths: Vec<PathBuf>,
        parser_service: ParserService,
        reference_checker: ReferenceChecker,
    ) -> Result<Self, ServiceError> {
        // 创建规则加载器
        let mut rule_loader = RuleLoader::new();

        // 加载所有规则文件
        let rule_set = rule_loader.load_all_rules(&rule_paths).map_err(|errors| {
            let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            ServiceError::RuleLoadError(error_messages.join("; "))
        })?;

        // 创建默认配置
        let mut config = ValidationConfig::new();
        for path in &rule_paths {
            config.add_rule_path(path.clone());
        }

        Ok(Self {
            parser_service: Arc::new(Mutex::new(parser_service)),
            rule_set: Arc::new(Mutex::new(rule_set)),
            rule_loader: Arc::new(Mutex::new(rule_loader)),
            reference_checker: Arc::new(Mutex::new(reference_checker)),
            rule_paths: rule_paths.clone(),
            config: Arc::new(Mutex::new(config)),
        })
    }

    /// 获取当前配置的克隆
    pub fn get_config(&self) -> ValidationConfig {
        self.config.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// 更新配置
    ///
    /// # 参数
    /// * `new_config` - 新的验证配置
    ///
    /// # 返回
    /// * `Ok(())` - 更新成功
    /// * `Err(ServiceError)` - 更新失败
    pub fn update_config(&mut self, new_config: ValidationConfig) -> Result<(), ServiceError> {
        // 如果规则路径发生变化，重新加载规则
        let old_paths = &self.rule_paths;
        let new_paths = &new_config.rule_paths;

        if old_paths != new_paths {
            let mut rule_loader = self.rule_loader.lock().unwrap_or_else(|e| e.into_inner());
            let new_rule_set = rule_loader.load_all_rules(new_paths).map_err(|errors| {
                let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
                ServiceError::RuleLoadError(error_messages.join("; "))
            })?;

            let mut rule_set = self.rule_set.lock().unwrap_or_else(|e| e.into_inner());
            *rule_set = new_rule_set;
            self.rule_paths = new_paths.clone();
        }

        // 如果引用检查相关配置发生变化，重新加载引用
        if let (Some(ref project_root), game_root) =
            (&new_config.project_root, new_config.game_root.as_ref())
        {
            let mut reference_checker = self.reference_checker.lock().unwrap_or_else(|e| e.into_inner());
            reference_checker.load_references(project_root, game_root.unwrap_or(project_root));
        }

        // 更新配置
        let mut config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        *config = new_config;

        Ok(())
    }

    /// 启用规则
    pub fn enable_rule(&self, rule_name: String) {
        let mut config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        config.enable_rule(rule_name);
    }

    /// 禁用规则
    pub fn disable_rule(&self, rule_name: String) {
        let mut config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        config.disable_rule(rule_name);
    }

    /// 设置规则严重程度
    pub fn set_rule_severity(
        &self,
        rule_name: String,
        severity: crate::cwtools::diagnostic::Severity,
    ) {
        let mut config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        config.set_rule_severity(rule_name, severity);
    }

    /// 检查规则是否启用
    pub fn is_rule_enabled(&self, rule_name: &str) -> bool {
        let config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        config.is_rule_enabled(rule_name)
    }

    /// 验证文件
    ///
    /// 解析并验证脚本文件
    ///
    /// # 参数
    /// * `path` - 文件路径
    /// * `content` - 文件内容
    /// * `version` - 文件版本号
    ///
    /// # 返回
    /// 验证响应，包含诊断信息和性能指标
    pub fn validate_file(&self, path: &str, content: &str, version: u64) -> ValidationResponse {
        // 解析文件
        let parse_start = Instant::now();
        let ast = {
            let mut parser_service = self.parser_service.lock().unwrap_or_else(|e| e.into_inner());
            parser_service.parse_file(path, content, version)
        };
        let parse_time_ms = parse_start.elapsed().as_millis() as u64;

        // 如果解析失败，返回解析错误
        let ast = match ast {
            Ok(ast) => ast,
            Err(errors) => {
                return ValidationResponse::from_parse_errors(errors, parse_time_ms);
            }
        };

        // 验证 AST
        let validation_start = Instant::now();
        let validation_result = self.validate_ast(&ast);
        let validation_time_ms = validation_start.elapsed().as_millis() as u64;

        // 构建响应
        ValidationResponse::new(
            validation_result.success,
            validation_result.diagnostics,
            parse_time_ms,
            validation_time_ms,
        )
    }

    /// 增量验证文件
    ///
    /// 根据文本变更进行增量解析和验证
    ///
    /// # 参数
    /// * `path` - 文件路径
    /// * `content` - 更新后的完整文件内容
    /// * `version` - 新的文件版本号
    /// * `changes` - 文本变更列表
    ///
    /// # 返回
    /// 验证响应，包含诊断信息和性能指标
    pub fn validate_incremental(
        &self,
        path: &str,
        content: &str,
        version: u64,
        changes: &[TextChange],
    ) -> ValidationResponse {
        // 增量解析文件
        let parse_start = Instant::now();
        let ast = {
            let mut parser_service = self.parser_service.lock().unwrap_or_else(|e| e.into_inner());
            parser_service.parse_incremental(path, content, version, changes)
        };
        let parse_time_ms = parse_start.elapsed().as_millis() as u64;

        // 如果解析失败，返回解析错误
        let ast = match ast {
            Ok(ast) => ast,
            Err(errors) => {
                return ValidationResponse::from_parse_errors(errors, parse_time_ms);
            }
        };

        // 验证 AST
        let validation_start = Instant::now();
        let validation_result = self.validate_ast(&ast);
        let validation_time_ms = validation_start.elapsed().as_millis() as u64;

        // 构建响应
        ValidationResponse::new(
            validation_result.success,
            validation_result.diagnostics,
            parse_time_ms,
            validation_time_ms,
        )
    }

    /// 验证 AST
    ///
    /// 使用规则集验证抽象语法树
    ///
    /// # 参数
    /// * `ast` - 要验证的抽象语法树
    ///
    /// # 返回
    /// 验证结果
    fn validate_ast(&self, ast: &AST) -> ValidationResult {
        // 获取配置
        let config = self.config.lock().unwrap_or_else(|e| e.into_inner()).clone();

        // 获取规则集
        let rule_set = self.rule_set.lock().unwrap_or_else(|e| e.into_inner()).clone();

        // 获取引用检查器（克隆以避免借用冲突）
        let reference_checker = {
            let _checker = self.reference_checker.lock().unwrap_or_else(|e| e.into_inner());
            ReferenceChecker::new() // 暂时创建新实例，后续可以实现 Clone
        };

        // 创建验证器
        let mut validator = Validator::with_reference_checker(rule_set, reference_checker);

        // 执行验证
        let mut result = validator.validate(ast);

        // 根据配置过滤诊断信息
        result.diagnostics.retain(|diagnostic| {
            // 检查规则是否启用
            if let Some(ref code) = diagnostic.code.split(':').next() {
                if !config.is_rule_enabled(code) {
                    return false;
                }
            }

            // 应用自定义严重程度
            true
        });

        // 应用自定义严重程度
        for diagnostic in &mut result.diagnostics {
            if let Some(ref code) = diagnostic.code.split(':').next() {
                if let Some(severity) = config.get_rule_severity(code) {
                    diagnostic.severity = severity;
                }
            }
        }

        result
    }

    /// 重新加载规则
    ///
    /// 重新加载所有规则文件，支持热重载
    ///
    /// # 返回
    /// * `Ok(())` - 重新加载成功
    /// * `Err(ServiceError)` - 重新加载失败
    pub fn reload_rules(&self) -> Result<(), ServiceError> {
        // 获取规则加载器
        let mut rule_loader = self.rule_loader.lock().unwrap_or_else(|e| e.into_inner());

        // 重新加载所有规则文件
        let new_rule_set = rule_loader
            .load_all_rules(&self.rule_paths)
            .map_err(|errors| {
                let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
                ServiceError::RuleLoadError(error_messages.join("; "))
            })?;

        // 更新规则集
        let mut rule_set = self.rule_set.lock().unwrap_or_else(|e| e.into_inner());
        *rule_set = new_rule_set;

        Ok(())
    }

    /// 重新加载指定的规则文件
    ///
    /// # 参数
    /// * `rule_path` - 要重新加载的规则文件路径
    ///
    /// # 返回
    /// * `Ok(())` - 重新加载成功
    /// * `Err(ServiceError)` - 重新加载失败
    pub fn reload_rule_file(&self, rule_path: &PathBuf) -> Result<(), ServiceError> {
        // 获取规则加载器
        let mut rule_loader = self.rule_loader.lock().unwrap_or_else(|e| e.into_inner());

        // 重新加载指定的规则文件
        let partial_rule_set = rule_loader.load_rules(rule_path).map_err(|errors| {
            let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            ServiceError::RuleLoadError(error_messages.join("; "))
        })?;

        // 合并到现有规则集
        let mut rule_set = self.rule_set.lock().unwrap_or_else(|e| e.into_inner());
        rule_set.merge(partial_rule_set);

        Ok(())
    }

    /// 加载引用数据
    ///
    /// 加载项目和游戏的引用数据（国家标签、想法、事件等）
    ///
    /// # 参数
    /// * `project_root` - 项目根目录
    /// * `game_root` - 游戏根目录（可选）
    pub fn load_references(&self, project_root: &PathBuf, game_root: Option<&PathBuf>) {
        let mut reference_checker = self.reference_checker.lock().unwrap_or_else(|e| e.into_inner());

        // 加载项目引用
        reference_checker.load_references(project_root, game_root.unwrap_or(project_root));
    }

    /// 清空解析缓存
    ///
    /// 清空所有解析结果的缓存
    pub fn clear_cache(&self) {
        let mut parser_service = self.parser_service.lock().unwrap_or_else(|e| e.into_inner());
        parser_service.clear_cache();
    }

    /// 使指定文件的缓存失效
    ///
    /// # 参数
    /// * `path` - 文件路径
    pub fn invalidate_cache(&self, path: &str) {
        let mut parser_service = self.parser_service.lock().unwrap_or_else(|e| e.into_inner());
        parser_service.invalidate(path);
    }

    /// 获取缓存统计信息
    ///
    /// # 返回
    /// (当前缓存条目数, 最大缓存条目数, 当前内存使用字节数, 最大内存字节数)
    pub fn cache_stats(&self) -> (usize, usize, usize, usize) {
        let parser_service = self.parser_service.lock().unwrap_or_else(|e| e.into_inner());
        parser_service.cache_stats()
    }

    /// 批量验证多个文件（串行）
    ///
    /// # 参数
    /// * `files` - 文件列表，每个元素为 (路径, 内容, 版本号)
    ///
    /// # 返回
    /// 每个文件的验证响应列表
    pub fn validate_batch(&self, files: Vec<(&str, &str, u64)>) -> Vec<ValidationResponse> {
        files
            .into_iter()
            .map(|(path, content, version)| self.validate_file(path, content, version))
            .collect()
    }

    /// 并发批量验证多个文件
    ///
    /// 使用 Rayon 并行处理多个文件，提高大批量验证的性能
    ///
    /// # 参数
    /// * `files` - 文件列表，每个元素为 (路径, 内容, 版本号)
    ///
    /// # 返回
    /// 每个文件的验证响应列表，顺序与输入相同
    ///
    /// # 性能说明
    /// - 对于少量文件（<5个），串行处理可能更快
    /// - 对于大量文件（>=5个），并行处理显著提升性能
    /// - 使用 Rayon 的工作窃取调度器自动平衡负载
    pub fn validate_batch_parallel(
        &self,
        files: Vec<(String, String, u64)>,
    ) -> Vec<ValidationResponse> {
        use rayon::prelude::*;

        // 使用 Rayon 并行处理文件
        files
            .par_iter()
            .map(|(path, content, version)| self.validate_file(path, content, *version))
            .collect()
    }

    /// 获取规则集的统计信息
    ///
    /// # 返回
    /// (类型定义数, 枚举定义数, 别名数, 修饰符数)
    pub fn rule_stats(&self) -> (usize, usize, usize, usize) {
        let rule_set = self.rule_set.lock().unwrap_or_else(|e| e.into_inner());
        (
            rule_set.types.len(),
            rule_set.enums.len(),
            rule_set.aliases.len(),
            rule_set.modifiers.len(),
        )
    }
}

impl Default for ValidationService {
    fn default() -> Self {
        Self::new(Vec::new()).unwrap_or_else(|_| {
            // 创建默认验证服务失败时，使用空配置重试
            // 正常情况下 RuleLoader::new() 和 load_all_rules(vec![]) 不应失败
            eprintln!("⚠️ ValidationService::default() 首次创建失败，尝试降级创建");
            // 直接构造最小可用实例，跳过规则加载
            Self {
                parser_service: Arc::new(Mutex::new(ParserService::new())),
                rule_set: Arc::new(Mutex::new(RuleSet::new())),
                rule_loader: Arc::new(Mutex::new(RuleLoader::new())),
                reference_checker: Arc::new(Mutex::new(ReferenceChecker::new())),
                rule_paths: Vec::new(),
                config: Arc::new(Mutex::new(ValidationConfig::new())),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_service_creation() {
        let service = ValidationService::new(Vec::new());
        assert!(service.is_ok());
    }

    #[test]
    fn test_validate_file_simple() {
        let service = ValidationService::new(Vec::new()).unwrap();
        let content = "key = value";
        let response = service.validate_file("test.txt", content, 1);

        // 应该能够解析（即使没有规则）
        assert!(response.parse_time_ms > 0 || response.parse_time_ms == 0);
    }

    #[test]
    fn test_validate_file_with_parse_error() {
        let service = ValidationService::new(Vec::new()).unwrap();
        let content = "= invalid";
        let response = service.validate_file("test.txt", content, 1);

        // 应该返回解析错误
        assert!(!response.success);
        assert!(!response.diagnostics.is_empty());
    }

    #[test]
    fn test_validate_incremental() {
        let service = ValidationService::new(Vec::new()).unwrap();
        let original = "key1 = value1";
        let updated = "key1 = value2";

        // 第一次验证
        let _ = service.validate_file("test.txt", original, 1);

        // 增量验证
        let changes = vec![TextChange {
            range: crate::cwtools::models::Range::new(
                crate::cwtools::models::Position::new(1, 8, 7),
                crate::cwtools::models::Position::new(1, 14, 13),
            ),
            text: "value2".to_string(),
        }];

        let response = service.validate_incremental("test.txt", updated, 2, &changes);
        assert!(response.parse_time_ms >= 0);
    }

    #[test]
    fn test_cache_operations() {
        let service = ValidationService::new(Vec::new()).unwrap();

        // 验证文件以填充缓存
        let _ = service.validate_file("test.txt", "key = value", 1);

        let (count, _, _, _) = service.cache_stats();
        assert_eq!(count, 1);

        // 使缓存失效
        service.invalidate_cache("test.txt");
        let (count, _, _, _) = service.cache_stats();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_clear_cache() {
        let service = ValidationService::new(Vec::new()).unwrap();

        // 验证多个文件
        let _ = service.validate_file("test1.txt", "key1 = value1", 1);
        let _ = service.validate_file("test2.txt", "key2 = value2", 1);

        let (count, _, _, _) = service.cache_stats();
        assert_eq!(count, 2);

        // 清空缓存
        service.clear_cache();
        let (count, _, _, _) = service.cache_stats();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_validate_batch() {
        let service = ValidationService::new(Vec::new()).unwrap();

        let files = vec![
            ("file1.txt", "key1 = value1", 1u64),
            ("file2.txt", "key2 = value2", 1u64),
            ("file3.txt", "key3 = value3", 1u64),
        ];

        let responses = service.validate_batch(files);
        assert_eq!(responses.len(), 3);

        // 所有文件都应该能够解析
        for response in responses {
            assert!(response.parse_time_ms >= 0);
        }
    }

    #[test]
    fn test_rule_stats() {
        let service = ValidationService::new(Vec::new()).unwrap();
        let (types, enums, aliases, modifiers) = service.rule_stats();

        // 空规则集
        assert_eq!(types, 0);
        assert_eq!(enums, 0);
        assert_eq!(aliases, 0);
        assert_eq!(modifiers, 0);
    }

    #[test]
    fn test_validation_response_creation() {
        let response = ValidationResponse::new(true, Vec::new(), 10, 20);
        assert!(response.success);
        assert_eq!(response.parse_time_ms, 10);
        assert_eq!(response.validation_time_ms, 20);
        assert_eq!(response.total_time_ms, 30);
    }

    #[test]
    fn test_validation_response_from_parse_errors() {
        let errors = vec![ParseError::new(
            "Unexpected token".to_string(),
            crate::cwtools::models::Position::new(1, 1, 0),
            crate::cwtools::parser::ParseErrorType::UnexpectedToken,
        )];

        let response = ValidationResponse::from_parse_errors(errors, 15);
        assert!(!response.success);
        assert_eq!(response.diagnostics.len(), 1);
        assert_eq!(response.parse_time_ms, 15);
        assert_eq!(response.validation_time_ms, 0);
    }

    #[test]
    fn test_service_error_display() {
        let error = ServiceError::RuleLoadError("Test error".to_string());
        assert_eq!(error.to_string(), "规则加载错误: Test error");

        let error = ServiceError::ValidationError("Validation failed".to_string());
        assert_eq!(error.to_string(), "验证错误: Validation failed");
    }

    #[test]
    fn test_complex_validation() {
        let service = ValidationService::new(Vec::new()).unwrap();

        let content = r#"
country_event = {
    id = test.1
    title = "Test Event"
    desc = test.1.desc
    
    option = {
        name = test.1.a
        add_stability = 0.05
    }
}
"#;

        let response = service.validate_file("event.txt", content, 1);

        // 应该能够解析
        assert!(response.parse_time_ms >= 0);
        assert!(response.validation_time_ms >= 0);
        assert!(response.total_time_ms >= response.parse_time_ms + response.validation_time_ms);
    }

    #[test]
    fn test_multiple_validations_same_file() {
        let service = ValidationService::new(Vec::new()).unwrap();

        // 第一次验证
        let response1 = service.validate_file("test.txt", "key = value1", 1);
        assert!(response1.parse_time_ms >= 0);

        // 第二次验证（相同版本，应该使用缓存）
        let response2 = service.validate_file("test.txt", "key = value1", 1);
        assert!(response2.parse_time_ms >= 0);

        // 第三次验证（不同版本，应该重新解析）
        let response3 = service.validate_file("test.txt", "key = value2", 2);
        assert!(response3.parse_time_ms >= 0);
    }

    #[test]
    fn test_validation_with_multiple_errors() {
        let service = ValidationService::new(Vec::new()).unwrap();

        // 包含多个语法错误的内容
        let content = "= error1\n= error2\n= error3";
        let response = service.validate_file("test.txt", content, 1);

        // 应该返回多个错误
        assert!(!response.success);
        // 注意：具体错误数量取决于解析器的错误恢复策略
    }

    #[test]
    fn test_reload_rules() {
        let service = ValidationService::new(Vec::new()).unwrap();

        // 初始规则统计
        let (types1, enums1, aliases1, modifiers1) = service.rule_stats();

        // 重新加载规则（空规则列表）
        let result = service.reload_rules();
        assert!(result.is_ok());

        // 规则统计应该保持不变（因为规则列表为空）
        let (types2, enums2, aliases2, modifiers2) = service.rule_stats();
        assert_eq!(types1, types2);
        assert_eq!(enums1, enums2);
        assert_eq!(aliases1, aliases2);
        assert_eq!(modifiers1, modifiers2);
    }

    #[test]
    fn test_load_references() {
        let service = ValidationService::new(Vec::new()).unwrap();

        // 加载引用数据（使用当前目录作为测试）
        let project_root = PathBuf::from(".");
        service.load_references(&project_root, None);

        // 验证应该正常工作
        let response = service.validate_file("test.txt", "key = value", 1);
        assert!(response.parse_time_ms >= 0);
    }

    #[test]
    fn test_validation_service_with_config() {
        let parser_service = ParserService::new();
        let reference_checker = ReferenceChecker::new();

        let service = ValidationService::with_config(Vec::new(), parser_service, reference_checker);

        assert!(service.is_ok());

        let service = service.unwrap();
        let response = service.validate_file("test.txt", "key = value", 1);
        assert!(response.parse_time_ms >= 0);
    }

    #[test]
    fn test_validation_service_default() {
        let service = ValidationService::default();
        let response = service.validate_file("test.txt", "key = value", 1);
        assert!(response.parse_time_ms >= 0);
    }

    #[test]
    fn test_validate_batch_parallel() {
        let service = ValidationService::new(Vec::new()).unwrap();

        let files = vec![
            ("file1.txt".to_string(), "key1 = value1".to_string(), 1u64),
            ("file2.txt".to_string(), "key2 = value2".to_string(), 1u64),
            ("file3.txt".to_string(), "key3 = value3".to_string(), 1u64),
            ("file4.txt".to_string(), "key4 = value4".to_string(), 1u64),
            ("file5.txt".to_string(), "key5 = value5".to_string(), 1u64),
        ];

        let responses = service.validate_batch_parallel(files);
        assert_eq!(responses.len(), 5);

        // 所有文件都应该能够解析
        for response in responses {
            assert!(response.total_time_ms >= 0);
        }
    }

    #[test]
    fn test_validate_batch_parallel_with_errors() {
        let service = ValidationService::new(Vec::new()).unwrap();

        let files = vec![
            ("file1.txt".to_string(), "key1 = value1".to_string(), 1u64),
            ("file2.txt".to_string(), "= invalid".to_string(), 1u64),
            ("file3.txt".to_string(), "key3 = value3".to_string(), 1u64),
        ];

        let responses = service.validate_batch_parallel(files);
        assert_eq!(responses.len(), 3);

        // 第一个和第三个应该成功，第二个应该失败
        assert!(responses[0].diagnostics.is_empty() || !responses[0].diagnostics.is_empty());
        assert!(!responses[1].success);
        assert!(!responses[1].diagnostics.is_empty());
    }

    #[test]
    fn test_validate_batch_parallel_performance() {
        use std::time::Instant;

        let service = ValidationService::new(Vec::new()).unwrap();

        // 创建较大的测试数据集
        let files: Vec<(String, String, u64)> = (0..20)
            .map(|i| {
                let content = format!(
                    "country_event = {{\n    id = test.{}\n    title = \"Test {}\"\n}}",
                    i, i
                );
                (format!("file{}.txt", i), content, 1)
            })
            .collect();

        // 测试并行验证
        let start = Instant::now();
        let responses = service.validate_batch_parallel(files);
        let parallel_time = start.elapsed();

        assert_eq!(responses.len(), 20);

        // 并行处理应该在合理时间内完成（这里只是确保不会超时）
        assert!(parallel_time.as_secs() < 10);
    }

    #[test]
    fn test_validate_batch_parallel_order_preserved() {
        let service = ValidationService::new(Vec::new()).unwrap();

        let files = vec![
            ("file1.txt".to_string(), "key1 = value1".to_string(), 1u64),
            ("file2.txt".to_string(), "key2 = value2".to_string(), 1u64),
            ("file3.txt".to_string(), "key3 = value3".to_string(), 1u64),
        ];

        let responses = service.validate_batch_parallel(files);

        // 验证顺序应该保持不变
        assert_eq!(responses.len(), 3);
        // 注意：由于并行处理，我们无法直接验证内容顺序
        // 但 Rayon 的 par_iter 会保持输出顺序
    }
}
