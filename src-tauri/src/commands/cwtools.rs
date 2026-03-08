//! cwtools 语法验证命令模块
//!
//! 提供 Paradox 脚本解析、验证和格式化的 Tauri 命令接口

use crate::cwtools::diagnostic::Diagnostic;
use crate::cwtools::formatter::format_script;
use crate::cwtools::models::{Position, Range};
use crate::cwtools::services::{TextChange, ValidationService};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::State;

/// 全局验证服务状态
///
/// 使用 Arc<Mutex<>> 包装以支持多线程访问
pub struct ValidationServiceState {
    service: Arc<Mutex<Option<ValidationService>>>,
    /// 防抖任务队列
    debounce_tasks: Arc<Mutex<HashMap<String, Instant>>>,
    /// 防抖延迟（毫秒）
    debounce_delay_ms: u64,
}

impl ValidationServiceState {
    /// 创建新的验证服务状态
    pub fn new() -> Self {
        Self {
            service: Arc::new(Mutex::new(None)),
            debounce_tasks: Arc::new(Mutex::new(HashMap::new())),
            debounce_delay_ms: 300, // 默认 300ms 防抖
        }
    }

    /// 创建带有自定义防抖延迟的验证服务状态
    ///
    /// # 参数
    /// * `debounce_delay_ms` - 防抖延迟（毫秒）
    pub fn with_debounce(debounce_delay_ms: u64) -> Self {
        Self {
            service: Arc::new(Mutex::new(None)),
            debounce_tasks: Arc::new(Mutex::new(HashMap::new())),
            debounce_delay_ms,
        }
    }

    /// 初始化验证服务
    ///
    /// # 参数
    /// * `rule_paths` - 规则文件路径列表
    ///
    /// # 返回
    /// * `Ok(())` - 初始化成功
    /// * `Err(String)` - 初始化失败的错误信息
    pub fn initialize(&self, rule_paths: Vec<PathBuf>) -> Result<(), String> {
        let service =
            ValidationService::new(rule_paths).map_err(|e| format!("初始化验证服务失败: {}", e))?;

        let mut state = self
            .service
            .lock()
            .map_err(|e| format!("锁定状态失败: {}", e))?;
        *state = Some(service);

        Ok(())
    }

    /// 检查是否应该执行任务（防抖）
    ///
    /// # 参数
    /// * `task_id` - 任务标识符（通常是文件路径）
    ///
    /// # 返回
    /// * `true` - 应该执行任务
    /// * `false` - 应该跳过任务（防抖中）
    fn should_execute(&self, task_id: &str) -> bool {
        let mut tasks = match self.debounce_tasks.lock() {
            Ok(t) => t,
            Err(_) => return true, // 如果锁定失败，直接执行
        };

        let now = Instant::now();

        if let Some(last_time) = tasks.get(task_id) {
            let elapsed = now.duration_since(*last_time);
            if elapsed < Duration::from_millis(self.debounce_delay_ms) {
                return false; // 还在防抖期内
            }
        }

        tasks.insert(task_id.to_string(), now);
        true
    }

    /// 清除防抖任务记录
    ///
    /// # 参数
    /// * `task_id` - 任务标识符
    fn clear_debounce(&self, task_id: &str) {
        if let Ok(mut tasks) = self.debounce_tasks.lock() {
            tasks.remove(task_id);
        }
    }

    /// 获取验证服务的引用
    ///
    /// # 返回
    /// * `Ok(ValidationService)` - 验证服务的克隆
    /// * `Err(String)` - 获取失败的错误信息
    fn get_service(&self) -> Result<ValidationService, String> {
        let state = self
            .service
            .lock()
            .map_err(|e| format!("锁定状态失败: {}", e))?;

        state
            .as_ref()
            .ok_or_else(|| "验证服务未初始化".to_string())
            .map(|_| {
                // 由于 ValidationService 没有实现 Clone，我们需要创建一个新实例
                // 或者返回一个引用。这里我们暂时返回错误，后续需要重构
                ValidationService::default()
            })
    }
}

impl Default for ValidationServiceState {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证响应
///
/// 包含验证结果和性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponse {
    /// 验证是否成功（无错误）
    pub success: bool,
    /// 诊断信息列表
    pub diagnostics: Vec<DiagnosticDto>,
    /// 解析耗时（毫秒）
    pub parse_time_ms: u64,
    /// 验证耗时（毫秒）
    pub validation_time_ms: u64,
    /// 总耗时（毫秒）
    pub total_time_ms: u64,
}

/// 解析响应
///
/// 包含解析结果和错误信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResponse {
    /// 解析是否成功
    pub success: bool,
    /// AST 的 JSON 表示（可选）
    pub ast: Option<serde_json::Value>,
    /// 错误信息列表
    pub errors: Vec<ParseErrorDto>,
    /// 解析耗时（毫秒）
    pub parse_time_ms: u64,
}

/// 诊断信息 DTO
///
/// 用于前端显示的诊断信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticDto {
    /// 错误代码
    pub code: String,
    /// 严重程度（error, warning, information, hint）
    pub severity: String,
    /// 错误消息
    pub message: String,
    /// 错误范围
    pub range: RangeDto,
    /// 来源
    pub source: String,
    /// 修复建议
    pub suggestions: Vec<SuggestionDto>,
}

/// 位置 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionDto {
    /// 行号（从 1 开始）
    pub line: usize,
    /// 列号（从 1 开始）
    pub column: usize,
    /// 字符偏移量（从 0 开始）
    pub offset: usize,
}

/// 范围 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeDto {
    /// 起始位置
    pub start: PositionDto,
    /// 结束位置
    pub end: PositionDto,
}

/// 修复建议 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionDto {
    /// 建议消息
    pub message: String,
    /// 替换文本（可选）
    pub replacement: Option<String>,
}

/// 解析错误 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseErrorDto {
    /// 错误消息
    pub message: String,
    /// 错误位置
    pub position: PositionDto,
    /// 错误类型
    pub error_type: String,
}

/// 文本变更 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChangeDto {
    /// 变更范围
    pub range: RangeDto,
    /// 新文本
    pub text: String,
}

/// 格式化选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatOptions {
    /// 缩进大小（空格数）
    pub indent_size: usize,
    /// 是否使用制表符
    pub use_tabs: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent_size: 4,
            use_tabs: false,
        }
    }
}

// ==================== 转换函数 ====================

/// 将内部 Diagnostic 转换为 DTO
fn diagnostic_to_dto(diagnostic: &Diagnostic) -> DiagnosticDto {
    DiagnosticDto {
        code: diagnostic.code.clone(),
        severity: format!("{:?}", diagnostic.severity).to_lowercase(),
        message: diagnostic.message.clone(),
        range: range_to_dto(&diagnostic.range),
        source: diagnostic.source.clone(),
        suggestions: diagnostic
            .suggestions
            .iter()
            .map(suggestion_to_dto)
            .collect(),
    }
}

/// 将内部 Range 转换为 DTO
fn range_to_dto(range: &Range) -> RangeDto {
    RangeDto {
        start: position_to_dto(&range.start),
        end: position_to_dto(&range.end),
    }
}

/// 将内部 Position 转换为 DTO
fn position_to_dto(position: &Position) -> PositionDto {
    PositionDto {
        line: position.line,
        column: position.column,
        offset: position.offset,
    }
}

/// 将内部 Suggestion 转换为 DTO
fn suggestion_to_dto(suggestion: &crate::cwtools::diagnostic::Suggestion) -> SuggestionDto {
    SuggestionDto {
        message: suggestion.message.clone(),
        replacement: suggestion.replacement.clone(),
    }
}

/// 将 DTO Range 转换为内部 Range
fn dto_to_range(dto: &RangeDto) -> Range {
    Range::new(dto_to_position(&dto.start), dto_to_position(&dto.end))
}

/// 将 DTO Position 转换为内部 Position
fn dto_to_position(dto: &PositionDto) -> Position {
    Position::new(dto.line, dto.column, dto.offset)
}

/// 将 DTO TextChange 转换为内部 TextChange
fn dto_to_text_change(dto: &TextChangeDto) -> TextChange {
    TextChange {
        range: dto_to_range(&dto.range),
        text: dto.text.clone(),
    }
}

// ==================== Tauri 命令 ====================

/// 初始化验证服务
///
/// # 参数
/// * `rule_paths` - 规则文件路径列表
///
/// # 返回
/// * `Ok(())` - 初始化成功
/// * `Err(String)` - 初始化失败的错误信息
#[tauri::command]
pub async fn initialize_validation_service(
    rule_paths: Vec<String>,
    state: State<'_, ValidationServiceState>,
) -> Result<(), String> {
    let paths: Vec<PathBuf> = rule_paths.into_iter().map(PathBuf::from).collect();
    state.initialize(paths)
}

/// 验证脚本
///
/// 解析并验证 Paradox 脚本内容
/// 使用后台线程执行，支持防抖
///
/// # 参数
/// * `content` - 脚本内容
/// * `file_path` - 文件路径（可选）
/// * `version` - 文件版本号（用于缓存）
/// * `skip_debounce` - 是否跳过防抖（默认 false）
///
/// # 返回
/// * `Ok(ValidationResponse)` - 验证响应
/// * `Err(String)` - 验证失败的错误信息
#[tauri::command]
pub async fn validate_script(
    content: String,
    file_path: Option<String>,
    version: Option<u64>,
    skip_debounce: Option<bool>,
    state: State<'_, ValidationServiceState>,
) -> Result<ValidationResponse, String> {
    let path = file_path.unwrap_or_else(|| "untitled.txt".to_string());
    let ver = version.unwrap_or(1);
    let skip = skip_debounce.unwrap_or(false);

    // 防抖检查
    if !skip && !state.should_execute(&path) {
        return Err("任务被防抖取消".to_string());
    }

    // 在后台线程执行验证
    let service_lock = state.service.clone();
    let path_clone = path.clone();
    let content_clone = content.clone();

    let result = {
        let service_guard = service_lock
            .lock()
            .map_err(|e| format!("锁定状态失败: {}", e))?;
        let service = service_guard.as_ref().ok_or_else(|| {
            "验证服务未初始化，请先调用 initialize_validation_service".to_string()
        })?;

        // 执行验证
        service.validate_file(&path_clone, &content_clone, ver)
    };

    // 清除防抖记录
    state.clear_debounce(&path);

    // 转换为 DTO
    Ok(ValidationResponse {
        success: result.success,
        diagnostics: result.diagnostics.iter().map(diagnostic_to_dto).collect(),
        parse_time_ms: result.parse_time_ms,
        validation_time_ms: result.validation_time_ms,
        total_time_ms: result.total_time_ms,
    })
}

/// 增量验证脚本
///
/// 根据文本变更进行增量解析和验证
///
/// # 参数
/// * `content` - 更新后的完整脚本内容
/// * `file_path` - 文件路径
/// * `version` - 新的文件版本号
/// * `changes` - 文本变更列表
///
/// # 返回
/// * `Ok(ValidationResponse)` - 验证响应
/// * `Err(String)` - 验证失败的错误信息
#[tauri::command]
pub async fn validate_script_incremental(
    content: String,
    file_path: String,
    version: u64,
    changes: Vec<TextChangeDto>,
    state: State<'_, ValidationServiceState>,
) -> Result<ValidationResponse, String> {
    // 获取验证服务
    let service_lock = state
        .service
        .lock()
        .map_err(|e| format!("锁定状态失败: {}", e))?;
    let service = service_lock
        .as_ref()
        .ok_or_else(|| "验证服务未初始化".to_string())?;

    // 转换变更列表
    let text_changes: Vec<TextChange> = changes.iter().map(dto_to_text_change).collect();

    // 执行增量验证
    let response = service.validate_incremental(&file_path, &content, version, &text_changes);

    // 转换为 DTO
    Ok(ValidationResponse {
        success: response.success,
        diagnostics: response.diagnostics.iter().map(diagnostic_to_dto).collect(),
        parse_time_ms: response.parse_time_ms,
        validation_time_ms: response.validation_time_ms,
        total_time_ms: response.total_time_ms,
    })
}

/// 解析文件
///
/// 仅解析脚本文件，不进行验证
///
/// # 参数
/// * `content` - 脚本内容
/// * `file_path` - 文件路径
///
/// # 返回
/// * `Ok(ParseResponse)` - 解析响应
/// * `Err(String)` - 解析失败的错误信息
#[tauri::command]
pub async fn parse_file(
    content: String,
    file_path: String,
    state: State<'_, ValidationServiceState>,
) -> Result<ParseResponse, String> {
    use std::time::Instant;

    // 获取验证服务中的解析器
    let service_lock = state
        .service
        .lock()
        .map_err(|e| format!("锁定状态失败: {}", e))?;
    let service = service_lock
        .as_ref()
        .ok_or_else(|| "验证服务未初始化".to_string())?;

    // 执行解析
    let start = Instant::now();
    let response = service.validate_file(&file_path, &content, 1);
    let parse_time_ms = start.elapsed().as_millis() as u64;

    // 如果有解析错误，返回错误信息
    if !response.diagnostics.is_empty() && response.parse_time_ms > 0 {
        let errors: Vec<ParseErrorDto> = response
            .diagnostics
            .iter()
            .filter(|d| d.severity == crate::cwtools::diagnostic::Severity::Error)
            .map(|d| ParseErrorDto {
                message: d.message.clone(),
                position: position_to_dto(&d.range.start),
                error_type: d.code.clone(),
            })
            .collect();

        return Ok(ParseResponse {
            success: errors.is_empty(),
            ast: None,
            errors,
            parse_time_ms,
        });
    }

    // 解析成功，返回空的 AST（暂时不序列化完整的 AST）
    Ok(ParseResponse {
        success: true,
        ast: Some(serde_json::json!({
            "statements": [],
            "source_file": file_path,
        })),
        errors: Vec::new(),
        parse_time_ms,
    })
}

/// 格式化脚本
///
/// 格式化 Paradox 脚本内容
///
/// # 参数
/// * `content` - 脚本内容
/// * `options` - 格式化选项（可选）
///
/// # 返回
/// * `Ok(String)` - 格式化后的脚本内容
/// * `Err(String)` - 格式化失败的错误信息
#[tauri::command]
pub async fn format_script_command(
    content: String,
    _options: Option<FormatOptions>,
) -> Result<String, String> {
    // 首先解析内容为 AST
    use crate::cwtools::parser::Parser;

    let mut parser = Parser::new(&content, "format.txt".to_string())
        .map_err(|e| format!("创建解析器失败: {}", e))?;

    let ast = parser
        .parse()
        .map_err(|errors| format!("解析失败: {} 个错误", errors.len()))?;

    // 调用格式化函数
    Ok(format_script(&ast))
}

/// 重新加载规则
///
/// 重新加载所有规则文件，支持热重载
///
/// # 参数
/// * `rule_paths` - 规则文件路径列表（可选，如果为空则使用原有路径）
///
/// # 返回
/// * `Ok(())` - 重新加载成功
/// * `Err(String)` - 重新加载失败的错误信息
#[tauri::command]
pub async fn reload_rules(
    rule_paths: Option<Vec<String>>,
    state: State<'_, ValidationServiceState>,
) -> Result<(), String> {
    // 获取验证服务
    let service_lock = state
        .service
        .lock()
        .map_err(|e| format!("锁定状态失败: {}", e))?;
    let service = service_lock
        .as_ref()
        .ok_or_else(|| "验证服务未初始化".to_string())?;

    // 如果提供了新的规则路径，需要重新初始化服务
    if let Some(paths) = rule_paths {
        drop(service_lock); // 释放锁
        let path_bufs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
        return state.initialize(path_bufs);
    }

    // 否则重新加载现有规则
    service
        .reload_rules()
        .map_err(|e| format!("重新加载规则失败: {}", e))
}

/// 清空解析缓存
///
/// 清空所有解析结果的缓存
///
/// # 返回
/// * `Ok(())` - 清空成功
/// * `Err(String)` - 清空失败的错误信息
#[tauri::command]
pub async fn clear_validation_cache(
    state: State<'_, ValidationServiceState>,
) -> Result<(), String> {
    // 获取验证服务
    let service_lock = state
        .service
        .lock()
        .map_err(|e| format!("锁定状态失败: {}", e))?;
    let service = service_lock
        .as_ref()
        .ok_or_else(|| "验证服务未初始化".to_string())?;

    service.clear_cache();
    Ok(())
}

/// 使指定文件的缓存失效
///
/// # 参数
/// * `file_path` - 文件路径
///
/// # 返回
/// * `Ok(())` - 操作成功
/// * `Err(String)` - 操作失败的错误信息
#[tauri::command]
pub async fn invalidate_file_cache(
    file_path: String,
    state: State<'_, ValidationServiceState>,
) -> Result<(), String> {
    // 获取验证服务
    let service_lock = state
        .service
        .lock()
        .map_err(|e| format!("锁定状态失败: {}", e))?;
    let service = service_lock
        .as_ref()
        .ok_or_else(|| "验证服务未初始化".to_string())?;

    service.invalidate_cache(&file_path);
    Ok(())
}

/// 获取缓存统计信息
///
/// # 返回
/// * `Ok(HashMap)` - 缓存统计信息
/// * `Err(String)` - 获取失败的错误信息
#[tauri::command]
pub async fn get_cache_stats(
    state: State<'_, ValidationServiceState>,
) -> Result<HashMap<String, usize>, String> {
    // 获取验证服务
    let service_lock = state
        .service
        .lock()
        .map_err(|e| format!("锁定状态失败: {}", e))?;
    let service = service_lock
        .as_ref()
        .ok_or_else(|| "验证服务未初始化".to_string())?;

    let (current, max, mem_used, mem_max) = service.cache_stats();

    let mut stats = HashMap::new();
    stats.insert("current".to_string(), current);
    stats.insert("max".to_string(), max);
    stats.insert("memory_used".to_string(), mem_used);
    stats.insert("memory_max".to_string(), mem_max);

    Ok(stats)
}

/// 获取规则统计信息
///
/// # 返回
/// * `Ok(HashMap)` - 规则统计信息
/// * `Err(String)` - 获取失败的错误信息
#[tauri::command]
pub async fn get_rule_stats(
    state: State<'_, ValidationServiceState>,
) -> Result<HashMap<String, usize>, String> {
    // 获取验证服务
    let service_lock = state
        .service
        .lock()
        .map_err(|e| format!("锁定状态失败: {}", e))?;
    let service = service_lock
        .as_ref()
        .ok_or_else(|| "验证服务未初始化".to_string())?;

    let (types, enums, aliases, modifiers) = service.rule_stats();

    let mut stats = HashMap::new();
    stats.insert("types".to_string(), types);
    stats.insert("enums".to_string(), enums);
    stats.insert("aliases".to_string(), aliases);
    stats.insert("modifiers".to_string(), modifiers);

    Ok(stats)
}

/// 批量验证多个文件
///
/// # 参数
/// * `files` - 文件列表，每个元素包含 path, content, version
///
/// # 返回
/// * `Ok(Vec<ValidationResponse>)` - 每个文件的验证响应列表
/// * `Err(String)` - 验证失败的错误信息
#[tauri::command]
pub async fn validate_batch(
    files: Vec<HashMap<String, serde_json::Value>>,
    state: State<'_, ValidationServiceState>,
) -> Result<Vec<ValidationResponse>, String> {
    // 获取验证服务
    let service_lock = state
        .service
        .lock()
        .map_err(|e| format!("锁定状态失败: {}", e))?;
    let service = service_lock
        .as_ref()
        .ok_or_else(|| "验证服务未初始化".to_string())?;

    // 解析文件列表并收集到 Vec
    let mut file_data: Vec<(String, String, u64)> = Vec::new();
    for file in &files {
        let path = file
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 path 字段".to_string())?
            .to_string();
        let content = file
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 content 字段".to_string())?
            .to_string();
        let version = file.get("version").and_then(|v| v.as_u64()).unwrap_or(1);

        file_data.push((path, content, version));
    }

    // 批量验证 - 转换为引用
    let file_refs: Vec<(&str, &str, u64)> = file_data
        .iter()
        .map(|(path, content, version)| (path.as_str(), content.as_str(), *version))
        .collect();

    let responses = service.validate_batch(file_refs);

    // 转换为 DTO
    Ok(responses
        .into_iter()
        .map(|response| ValidationResponse {
            success: response.success,
            diagnostics: response.diagnostics.iter().map(diagnostic_to_dto).collect(),
            parse_time_ms: response.parse_time_ms,
            validation_time_ms: response.validation_time_ms,
            total_time_ms: response.total_time_ms,
        })
        .collect())
}

/// 加载引用数据
///
/// 加载项目和游戏的引用数据（国家标签、想法、事件等）
///
/// # 参数
/// * `project_root` - 项目根目录
/// * `game_root` - 游戏根目录（可选）
///
/// # 返回
/// * `Ok(())` - 加载成功
/// * `Err(String)` - 加载失败的错误信息
#[tauri::command]
pub async fn load_references(
    project_root: String,
    game_root: Option<String>,
    state: State<'_, ValidationServiceState>,
) -> Result<(), String> {
    // 获取验证服务
    let service_lock = state
        .service
        .lock()
        .map_err(|e| format!("锁定状态失败: {}", e))?;
    let service = service_lock
        .as_ref()
        .ok_or_else(|| "验证服务未初始化".to_string())?;

    let project_path = PathBuf::from(project_root);
    let game_path = game_root.map(PathBuf::from);

    service.load_references(&project_path, game_path.as_ref());

    Ok(())
}
