//! 错误日志系统
//!
//! 负责记录解析错误、验证错误和系统错误，支持日志级别过滤和持久化

use crate::cwtools::parser::ParseError;
use crate::cwtools::rules::loader::RuleError;
use crate::cwtools::services::validation_service::ServiceError;
use crate::cwtools::validator::scope::ScopeError;
use chrono::{DateTime, Local};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// 调试信息
    Debug = 0,
    /// 一般信息
    Info = 1,
    /// 警告
    Warning = 2,
    /// 错误
    Error = 3,
    /// 严重错误
    Critical = 4,
}

impl LogLevel {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        }
    }

    /// 从字符串解析日志级别
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "DEBUG" => Some(LogLevel::Debug),
            "INFO" => Some(LogLevel::Info),
            "WARNING" | "WARN" => Some(LogLevel::Warning),
            "ERROR" => Some(LogLevel::Error),
            "CRITICAL" | "CRIT" => Some(LogLevel::Critical),
            _ => None,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// 时间戳
    pub timestamp: DateTime<Local>,
    /// 日志级别
    pub level: LogLevel,
    /// 日志类别
    pub category: String,
    /// 日志消息
    pub message: String,
    /// 源文件路径（可选）
    pub source_file: Option<String>,
    /// 行号（可选）
    pub line: Option<usize>,
    /// 列号（可选）
    pub column: Option<usize>,
}

impl LogEntry {
    /// 创建新的日志条目
    pub fn new(level: LogLevel, category: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            timestamp: Local::now(),
            level,
            category: category.into(),
            message: message.into(),
            source_file: None,
            line: None,
            column: None,
        }
    }

    /// 设置源文件信息
    pub fn with_source(mut self, file: impl Into<String>, line: usize, column: usize) -> Self {
        self.source_file = Some(file.into());
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// 格式化为日志字符串
    pub fn format(&self) -> String {
        let timestamp = self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
        let location = if let Some(file) = &self.source_file {
            if let (Some(line), Some(column)) = (self.line, self.column) {
                format!(" [{}:{}:{}]", file, line, column)
            } else {
                format!(" [{}]", file)
            }
        } else {
            String::new()
        };

        format!(
            "[{}] {} [{}]{}: {}",
            timestamp, self.level, self.category, location, self.message
        )
    }
}

/// 错误日志记录器
///
/// 负责记录各类错误和系统事件，支持日志级别过滤和文件持久化
pub struct ErrorLogger {
    /// 日志文件路径
    log_file: Option<PathBuf>,
    /// 最小日志级别（低于此级别的日志不会被记录）
    min_level: LogLevel,
    /// 日志缓冲区
    buffer: Arc<Mutex<Vec<LogEntry>>>,
    /// 是否启用控制台输出
    console_output: bool,
    /// 最大缓冲区大小
    max_buffer_size: usize,
}

impl ErrorLogger {
    /// 创建新的错误日志记录器
    ///
    /// # 参数
    /// * `log_file` - 日志文件路径（None 表示不写入文件）
    /// * `min_level` - 最小日志级别
    ///
    /// # 返回
    /// ErrorLogger 实例
    pub fn new(log_file: Option<PathBuf>, min_level: LogLevel) -> Self {
        Self {
            log_file,
            min_level,
            buffer: Arc::new(Mutex::new(Vec::new())),
            console_output: true,
            max_buffer_size: 1000,
        }
    }

    /// 创建默认的错误日志记录器
    ///
    /// 使用默认配置：不写入文件，最小级别为 Info，启用控制台输出
    pub fn default() -> Self {
        Self::new(None, LogLevel::Info)
    }

    /// 设置是否启用控制台输出
    pub fn set_console_output(&mut self, enabled: bool) {
        self.console_output = enabled;
    }

    /// 设置最大缓冲区大小
    pub fn set_max_buffer_size(&mut self, size: usize) {
        self.max_buffer_size = size;
    }

    /// 设置最小日志级别
    pub fn set_min_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }

    /// 记录日志条目
    ///
    /// # 参数
    /// * `entry` - 日志条目
    pub fn log(&self, entry: LogEntry) {
        // 检查日志级别
        if entry.level < self.min_level {
            return;
        }

        // 格式化日志
        let formatted = entry.format();

        // 输出到控制台
        if self.console_output {
            match entry.level {
                LogLevel::Error | LogLevel::Critical => {
                    eprintln!("{}", formatted);
                }
                _ => {
                    println!("{}", formatted);
                }
            }
        }

        // 添加到缓冲区
        {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.push(entry);

            // 如果缓冲区满了，刷新到文件
            if buffer.len() >= self.max_buffer_size {
                if let Err(e) = self.flush_buffer_internal(&mut buffer) {
                    eprintln!("Failed to flush log buffer: {}", e);
                }
            }
        }
    }

    /// 记录解析错误
    ///
    /// # 参数
    /// * `error` - 解析错误
    pub fn log_parse_error(&self, error: &ParseError) {
        let entry = LogEntry::new(
            LogLevel::Error,
            "parser",
            format!("Parse error: {}", error.message),
        )
        .with_source(
            "unknown".to_string(),
            error.position.line,
            error.position.column,
        );

        self.log(entry);
    }

    /// 记录规则加载错误
    ///
    /// # 参数
    /// * `error` - 规则错误
    pub fn log_rule_error(&self, error: &RuleError) {
        let mut entry = LogEntry::new(
            LogLevel::Error,
            "rules",
            format!("Rule error: {}", error.message),
        );

        if let Some(pos) = &error.position {
            if let Some(file) = &error.file {
                entry = entry.with_source(file.clone(), pos.line, pos.column);
            }
        }

        self.log(entry);
    }

    /// 记录验证错误
    ///
    /// # 参数
    /// * `message` - 错误消息
    /// * `file` - 源文件路径（可选）
    pub fn log_validation_error(&self, message: impl Into<String>, file: Option<&str>) {
        let mut entry = LogEntry::new(LogLevel::Error, "validator", message);

        if let Some(f) = file {
            entry.source_file = Some(f.to_string());
        }

        self.log(entry);
    }

    /// 记录作用域错误
    ///
    /// # 参数
    /// * `error` - 作用域错误
    /// * `file` - 源文件路径（可选）
    pub fn log_scope_error(&self, error: &ScopeError, file: Option<&str>) {
        let mut entry = LogEntry::new(LogLevel::Error, "scope", format!("Scope error: {}", error));

        if let Some(f) = file {
            entry.source_file = Some(f.to_string());
        }

        self.log(entry);
    }

    /// 记录服务错误
    ///
    /// # 参数
    /// * `error` - 服务错误
    pub fn log_service_error(&self, error: &ServiceError) {
        let entry = LogEntry::new(
            LogLevel::Error,
            "service",
            format!("Service error: {}", error),
        );

        self.log(entry);
    }

    /// 记录系统错误
    ///
    /// # 参数
    /// * `message` - 错误消息
    pub fn log_system_error(&self, message: impl Into<String>) {
        let entry = LogEntry::new(LogLevel::Critical, "system", message);
        self.log(entry);
    }

    /// 记录警告
    ///
    /// # 参数
    /// * `category` - 日志类别
    /// * `message` - 警告消息
    pub fn log_warning(&self, category: impl Into<String>, message: impl Into<String>) {
        let entry = LogEntry::new(LogLevel::Warning, category, message);
        self.log(entry);
    }

    /// 记录错误
    ///
    /// # 参数
    /// * `category` - 日志类别
    /// * `message` - 错误消息
    pub fn log_error(&self, category: impl Into<String>, message: impl Into<String>) {
        let entry = LogEntry::new(LogLevel::Error, category, message);
        self.log(entry);
    }

    /// 记录信息
    ///
    /// # 参数
    /// * `category` - 日志类别
    /// * `message` - 信息消息
    pub fn log_info(&self, category: impl Into<String>, message: impl Into<String>) {
        let entry = LogEntry::new(LogLevel::Info, category, message);
        self.log(entry);
    }

    /// 记录调试信息
    ///
    /// # 参数
    /// * `category` - 日志类别
    /// * `message` - 调试消息
    pub fn log_debug(&self, category: impl Into<String>, message: impl Into<String>) {
        let entry = LogEntry::new(LogLevel::Debug, category, message);
        self.log(entry);
    }

    /// 刷新缓冲区到文件
    ///
    /// # 返回
    /// * `Ok(())` - 刷新成功
    /// * `Err(io::Error)` - 刷新失败
    pub fn flush(&self) -> io::Result<()> {
        let mut buffer = self.buffer.lock().unwrap();
        self.flush_buffer_internal(&mut buffer)
    }

    /// 内部刷新缓冲区实现
    fn flush_buffer_internal(&self, buffer: &mut Vec<LogEntry>) -> io::Result<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        if let Some(log_file) = &self.log_file {
            // 确保日志目录存在
            if let Some(parent) = log_file.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // 打开日志文件（追加模式）
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)?;

            // 写入所有缓冲的日志
            for entry in buffer.iter() {
                writeln!(file, "{}", entry.format())?;
            }

            file.flush()?;
        }

        // 清空缓冲区
        buffer.clear();

        Ok(())
    }

    /// 获取缓冲区中的日志条目数量
    pub fn buffer_size(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    /// 获取缓冲区中的所有日志条目（副本）
    pub fn get_buffered_logs(&self) -> Vec<LogEntry> {
        self.buffer.lock().unwrap().clone()
    }

    /// 清空缓冲区（不写入文件）
    pub fn clear_buffer(&self) {
        self.buffer.lock().unwrap().clear();
    }

    /// 获取指定级别的日志条目
    ///
    /// # 参数
    /// * `level` - 日志级别
    ///
    /// # 返回
    /// 匹配指定级别的日志条目列表
    pub fn get_logs_by_level(&self, level: LogLevel) -> Vec<LogEntry> {
        self.buffer
            .lock()
            .unwrap()
            .iter()
            .filter(|entry| entry.level == level)
            .cloned()
            .collect()
    }

    /// 获取指定类别的日志条目
    ///
    /// # 参数
    /// * `category` - 日志类别
    ///
    /// # 返回
    /// 匹配指定类别的日志条目列表
    pub fn get_logs_by_category(&self, category: &str) -> Vec<LogEntry> {
        self.buffer
            .lock()
            .unwrap()
            .iter()
            .filter(|entry| entry.category == category)
            .cloned()
            .collect()
    }

    /// 读取日志文件内容
    ///
    /// # 返回
    /// * `Ok(String)` - 日志文件内容
    /// * `Err(io::Error)` - 读取失败
    pub fn read_log_file(&self) -> io::Result<String> {
        if let Some(log_file) = &self.log_file {
            std::fs::read_to_string(log_file)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No log file configured",
            ))
        }
    }

    /// 清空日志文件
    ///
    /// # 返回
    /// * `Ok(())` - 清空成功
    /// * `Err(io::Error)` - 清空失败
    pub fn clear_log_file(&self) -> io::Result<()> {
        if let Some(log_file) = &self.log_file {
            File::create(log_file)?;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No log file configured",
            ))
        }
    }

    /// 获取日志统计信息
    ///
    /// # 返回
    /// (总数, 调试, 信息, 警告, 错误, 严重错误)
    pub fn get_stats(&self) -> (usize, usize, usize, usize, usize, usize) {
        let buffer = self.buffer.lock().unwrap();
        let total = buffer.len();
        let debug = buffer.iter().filter(|e| e.level == LogLevel::Debug).count();
        let info = buffer.iter().filter(|e| e.level == LogLevel::Info).count();
        let warning = buffer
            .iter()
            .filter(|e| e.level == LogLevel::Warning)
            .count();
        let error = buffer.iter().filter(|e| e.level == LogLevel::Error).count();
        let critical = buffer
            .iter()
            .filter(|e| e.level == LogLevel::Critical)
            .count();

        (total, debug, info, warning, error, critical)
    }
}

impl Drop for ErrorLogger {
    fn drop(&mut self) {
        // 在销毁时刷新缓冲区
        if let Err(e) = self.flush() {
            eprintln!("Failed to flush log buffer on drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Critical);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("DEBUG"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("info"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("WARNING"), Some(LogLevel::Warning));
        assert_eq!(LogLevel::from_str("error"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("CRITICAL"), Some(LogLevel::Critical));
        assert_eq!(LogLevel::from_str("unknown"), None);
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogLevel::Error, "test", "Test message");
        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.category, "test");
        assert_eq!(entry.message, "Test message");
        assert!(entry.source_file.is_none());
    }

    #[test]
    fn test_log_entry_with_source() {
        let entry =
            LogEntry::new(LogLevel::Error, "test", "Test message").with_source("test.txt", 10, 5);

        assert_eq!(entry.source_file, Some("test.txt".to_string()));
        assert_eq!(entry.line, Some(10));
        assert_eq!(entry.column, Some(5));
    }

    #[test]
    fn test_log_entry_format() {
        let entry =
            LogEntry::new(LogLevel::Error, "test", "Test message").with_source("test.txt", 10, 5);

        let formatted = entry.format();
        assert!(formatted.contains("ERROR"));
        assert!(formatted.contains("[test]"));
        assert!(formatted.contains("[test.txt:10:5]"));
        assert!(formatted.contains("Test message"));
    }

    #[test]
    fn test_error_logger_creation() {
        let logger = ErrorLogger::new(None, LogLevel::Info);
        assert_eq!(logger.min_level, LogLevel::Info);
        assert_eq!(logger.buffer_size(), 0);
    }

    #[test]
    fn test_error_logger_default() {
        let logger = ErrorLogger::default();
        assert_eq!(logger.min_level, LogLevel::Info);
        assert!(logger.console_output);
    }

    #[test]
    fn test_log_filtering_by_level() {
        let logger = ErrorLogger::new(None, LogLevel::Warning);

        logger.log_debug("test", "Debug message");
        logger.log_info("test", "Info message");
        logger.log_warning("test", "Warning message");
        logger.log_error("test", "Error message");

        // 只有 Warning 和 Error 应该被记录
        assert_eq!(logger.buffer_size(), 2);
    }

    #[test]
    fn test_log_parse_error() {
        let logger = ErrorLogger::new(None, LogLevel::Error);

        let error = ParseError::new(
            "Test parse error".to_string(),
            crate::cwtools::models::Position::new(10, 5, 100),
            crate::cwtools::parser::ParseErrorType::UnexpectedToken,
        );

        logger.log_parse_error(&error);
        assert_eq!(logger.buffer_size(), 1);

        let logs = logger.get_logs_by_category("parser");
        assert_eq!(logs.len(), 1);
        assert!(logs[0].message.contains("Test parse error"));
    }

    #[test]
    fn test_get_logs_by_level() {
        let logger = ErrorLogger::new(None, LogLevel::Debug);

        logger.log_info("test", "Info 1");
        logger.log_warning("test", "Warning 1");
        logger.log_error("test", "Error 1");
        logger.log_info("test", "Info 2");

        let errors = logger.get_logs_by_level(LogLevel::Error);
        assert_eq!(errors.len(), 1);

        let infos = logger.get_logs_by_level(LogLevel::Info);
        assert_eq!(infos.len(), 2);
    }

    #[test]
    fn test_get_logs_by_category() {
        let logger = ErrorLogger::new(None, LogLevel::Debug);

        logger.log_info("parser", "Parser message");
        logger.log_info("validator", "Validator message");
        logger.log_info("parser", "Another parser message");

        let parser_logs = logger.get_logs_by_category("parser");
        assert_eq!(parser_logs.len(), 2);

        let validator_logs = logger.get_logs_by_category("validator");
        assert_eq!(validator_logs.len(), 1);
    }

    #[test]
    fn test_clear_buffer() {
        let logger = ErrorLogger::new(None, LogLevel::Info);

        logger.log_info("test", "Message 1");
        logger.log_info("test", "Message 2");
        assert_eq!(logger.buffer_size(), 2);

        logger.clear_buffer();
        assert_eq!(logger.buffer_size(), 0);
    }

    #[test]
    fn test_get_stats() {
        let logger = ErrorLogger::new(None, LogLevel::Debug);

        logger.log_debug("test", "Debug");
        logger.log_info("test", "Info");
        logger.log_info("test", "Info 2");
        logger.log_warning("test", "Warning");
        logger.log_error("test", "Error");
        logger.log_system_error("Critical");

        let (total, debug, info, warning, error, critical) = logger.get_stats();
        assert_eq!(total, 6);
        assert_eq!(debug, 1);
        assert_eq!(info, 2);
        assert_eq!(warning, 1);
        assert_eq!(error, 1);
        assert_eq!(critical, 1);
    }

    #[test]
    fn test_file_logging() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("test.log");

        let logger = ErrorLogger::new(Some(log_file.clone()), LogLevel::Info);

        logger.log_info("test", "Test message 1");
        logger.log_error("test", "Test message 2");

        // 刷新到文件
        logger.flush().unwrap();

        // 读取文件内容
        let content = fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("Test message 1"));
        assert!(content.contains("Test message 2"));
    }

    #[test]
    fn test_auto_flush_on_buffer_full() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("test.log");

        let mut logger = ErrorLogger::new(Some(log_file.clone()), LogLevel::Info);
        logger.set_max_buffer_size(5);

        // 添加超过缓冲区大小的日志
        for i in 0..10 {
            logger.log_info("test", format!("Message {}", i));
        }

        // 缓冲区应该已经被刷新
        assert!(logger.buffer_size() < 10);

        // 文件应该包含一些日志
        let content = fs::read_to_string(&log_file).unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_clear_log_file() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("test.log");

        let logger = ErrorLogger::new(Some(log_file.clone()), LogLevel::Info);

        logger.log_info("test", "Test message");
        logger.flush().unwrap();

        // 确认文件有内容
        let content = fs::read_to_string(&log_file).unwrap();
        assert!(!content.is_empty());

        // 清空文件
        logger.clear_log_file().unwrap();

        // 确认文件已清空
        let content = fs::read_to_string(&log_file).unwrap();
        assert!(content.is_empty());
    }

    #[test]
    fn test_set_min_level() {
        let mut logger = ErrorLogger::new(None, LogLevel::Info);

        logger.log_debug("test", "Debug message");
        assert_eq!(logger.buffer_size(), 0);

        logger.set_min_level(LogLevel::Debug);
        logger.log_debug("test", "Debug message");
        assert_eq!(logger.buffer_size(), 1);
    }

    #[test]
    fn test_console_output_toggle() {
        let mut logger = ErrorLogger::new(None, LogLevel::Info);
        assert!(logger.console_output);

        logger.set_console_output(false);
        assert!(!logger.console_output);

        // 日志仍然应该被记录到缓冲区
        logger.log_info("test", "Test message");
        assert_eq!(logger.buffer_size(), 1);
    }
}
