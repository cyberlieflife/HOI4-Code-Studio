//! 诊断系统模块
//!
//! 提供错误、警告和信息的诊断功能，支持 CodeMirror 格式转换

use crate::cwtools::models::{Position, Range};
use serde::{Deserialize, Serialize};

/// 诊断信息
///
/// 表示一个错误、警告或提示信息，包含位置、严重程度和描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// 错误代码，用于文档查询
    pub code: String,
    /// 严重程度
    pub severity: Severity,
    /// 错误描述信息
    pub message: String,
    /// 错误位置范围
    pub range: Range,
    /// 错误来源（如 "parser", "validator"）
    pub source: String,
    /// 修复建议列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<Suggestion>,
}

impl Diagnostic {
    /// 创建新的诊断信息
    pub fn new(
        code: String,
        severity: Severity,
        message: String,
        range: Range,
        source: String,
    ) -> Self {
        Self {
            code,
            severity,
            message,
            range,
            source,
            suggestions: Vec::new(),
        }
    }

    /// 创建错误级别的诊断
    pub fn error(code: String, message: String, range: Range, source: String) -> Self {
        Self::new(code, Severity::Error, message, range, source)
    }

    /// 创建警告级别的诊断
    pub fn warning(code: String, message: String, range: Range, source: String) -> Self {
        Self::new(code, Severity::Warning, message, range, source)
    }

    /// 创建信息级别的诊断
    pub fn info(code: String, message: String, range: Range, source: String) -> Self {
        Self::new(code, Severity::Information, message, range, source)
    }

    /// 创建提示级别的诊断
    pub fn hint(code: String, message: String, range: Range, source: String) -> Self {
        Self::new(code, Severity::Hint, message, range, source)
    }

    /// 添加修复建议
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// 添加多个修复建议
    pub fn with_suggestions(mut self, suggestions: Vec<Suggestion>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    /// 转换为 CodeMirror 诊断格式
    pub fn to_codemirror_format(&self) -> CodeMirrorDiagnostic {
        CodeMirrorDiagnostic {
            from: self.range.start.offset,
            to: self.range.end.offset,
            severity: self.severity.to_codemirror_string().to_string(),
            message: self.message.clone(),
        }
    }
}

/// 诊断严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// 错误：必须修复的问题
    Error,
    /// 警告：建议修复的问题
    Warning,
    /// 信息：提供额外信息
    Information,
    /// 提示：轻微的建议
    Hint,
}

impl Severity {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Information => "information",
            Severity::Hint => "hint",
        }
    }

    /// 转换为 CodeMirror 使用的字符串格式
    pub fn to_codemirror_string(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Information => "info",
            Severity::Hint => "hint",
        }
    }

    /// 从字符串解析严重程度
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(Severity::Error),
            "warning" => Some(Severity::Warning),
            "information" | "info" => Some(Severity::Information),
            "hint" => Some(Severity::Hint),
            _ => None,
        }
    }
}

/// 修复建议
///
/// 提供可能的修复方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// 建议描述
    pub message: String,
    /// 替换文本（如果适用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
}

impl Suggestion {
    /// 创建新的建议
    pub fn new(message: String) -> Self {
        Self {
            message,
            replacement: None,
        }
    }

    /// 创建带替换文本的建议
    pub fn with_replacement(message: String, replacement: String) -> Self {
        Self {
            message,
            replacement: Some(replacement),
        }
    }
}

/// CodeMirror 诊断格式
///
/// 用于前端 CodeMirror 编辑器显示的诊断格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMirrorDiagnostic {
    /// 起始字节偏移量
    pub from: usize,
    /// 结束字节偏移量
    pub to: usize,
    /// 严重程度字符串
    pub severity: String,
    /// 错误消息
    pub message: String,
}

/// 诊断管理器
///
/// 负责收集、过滤和导出诊断信息
#[derive(Debug, Default)]
pub struct DiagnosticManager {
    /// 诊断信息列表
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticManager {
    /// 创建新的诊断管理器
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// 添加诊断信息
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// 批量添加诊断信息
    pub fn add_all(&mut self, diagnostics: Vec<Diagnostic>) {
        self.diagnostics.extend(diagnostics);
    }

    /// 清空所有诊断信息
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    /// 获取所有诊断信息
    pub fn get_all(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// 获取诊断信息数量
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// 按严重程度过滤诊断信息
    pub fn filter_by_severity(&self, severity: Severity) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == severity)
            .collect()
    }

    /// 按最低严重程度过滤（包含该级别及更严重的）
    pub fn filter_by_min_severity(&self, min_severity: Severity) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity <= min_severity)
            .collect()
    }

    /// 按来源过滤诊断信息
    pub fn filter_by_source(&self, source: &str) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.source == source)
            .collect()
    }

    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    /// 获取警告数量
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }

    /// 检查是否有错误
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// 检查是否有警告
    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Warning)
    }

    /// 转换为 CodeMirror 格式
    pub fn to_codemirror_format(&self) -> Vec<CodeMirrorDiagnostic> {
        self.diagnostics
            .iter()
            .map(|d| d.to_codemirror_format())
            .collect()
    }

    /// 按位置排序诊断信息
    pub fn sort_by_position(&mut self) {
        self.diagnostics.sort_by(|a, b| {
            a.range
                .start
                .line
                .cmp(&b.range.start.line)
                .then_with(|| a.range.start.column.cmp(&b.range.start.column))
        });
    }

    /// 导出为 JSON 格式
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.diagnostics)
    }

    /// 从 JSON 导入
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let diagnostics: Vec<Diagnostic> = serde_json::from_str(json)?;
        Ok(Self { diagnostics })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_range() -> Range {
        Range::new(Position::new(1, 1, 0), Position::new(1, 10, 9))
    }

    #[test]
    fn test_diagnostic_creation() {
        let range = create_test_range();
        let diag = Diagnostic::new(
            "E001".to_string(),
            Severity::Error,
            "Test error".to_string(),
            range,
            "test".to_string(),
        );

        assert_eq!(diag.code, "E001");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "Test error");
        assert_eq!(diag.source, "test");
        assert!(diag.suggestions.is_empty());
    }

    #[test]
    fn test_diagnostic_error() {
        let range = create_test_range();
        let diag = Diagnostic::error(
            "E001".to_string(),
            "Test error".to_string(),
            range,
            "test".to_string(),
        );

        assert_eq!(diag.severity, Severity::Error);
    }

    #[test]
    fn test_diagnostic_warning() {
        let range = create_test_range();
        let diag = Diagnostic::warning(
            "W001".to_string(),
            "Test warning".to_string(),
            range,
            "test".to_string(),
        );

        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_diagnostic_info() {
        let range = create_test_range();
        let diag = Diagnostic::info(
            "I001".to_string(),
            "Test info".to_string(),
            range,
            "test".to_string(),
        );

        assert_eq!(diag.severity, Severity::Information);
    }

    #[test]
    fn test_diagnostic_hint() {
        let range = create_test_range();
        let diag = Diagnostic::hint(
            "H001".to_string(),
            "Test hint".to_string(),
            range,
            "test".to_string(),
        );

        assert_eq!(diag.severity, Severity::Hint);
    }

    #[test]
    fn test_diagnostic_with_suggestion() {
        let range = create_test_range();
        let suggestion = Suggestion::new("Try this fix".to_string());
        let diag = Diagnostic::error(
            "E001".to_string(),
            "Test error".to_string(),
            range,
            "test".to_string(),
        )
        .with_suggestion(suggestion);

        assert_eq!(diag.suggestions.len(), 1);
        assert_eq!(diag.suggestions[0].message, "Try this fix");
    }

    #[test]
    fn test_diagnostic_with_suggestions() {
        let range = create_test_range();
        let suggestions = vec![
            Suggestion::new("Fix 1".to_string()),
            Suggestion::new("Fix 2".to_string()),
        ];
        let diag = Diagnostic::error(
            "E001".to_string(),
            "Test error".to_string(),
            range,
            "test".to_string(),
        )
        .with_suggestions(suggestions);

        assert_eq!(diag.suggestions.len(), 2);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error < Severity::Warning);
        assert!(Severity::Warning < Severity::Information);
        assert!(Severity::Information < Severity::Hint);
    }

    #[test]
    fn test_severity_to_codemirror_string() {
        assert_eq!(Severity::Error.to_codemirror_string(), "error");
        assert_eq!(Severity::Warning.to_codemirror_string(), "warning");
        assert_eq!(Severity::Information.to_codemirror_string(), "info");
        assert_eq!(Severity::Hint.to_codemirror_string(), "hint");
    }

    #[test]
    fn test_severity_from_str() {
        assert_eq!(Severity::from_str("error"), Some(Severity::Error));
        assert_eq!(Severity::from_str("warning"), Some(Severity::Warning));
        assert_eq!(Severity::from_str("info"), Some(Severity::Information));
        assert_eq!(
            Severity::from_str("information"),
            Some(Severity::Information)
        );
        assert_eq!(Severity::from_str("hint"), Some(Severity::Hint));
        assert_eq!(Severity::from_str("invalid"), None);
    }

    #[test]
    fn test_suggestion_creation() {
        let suggestion = Suggestion::new("Test suggestion".to_string());
        assert_eq!(suggestion.message, "Test suggestion");
        assert!(suggestion.replacement.is_none());
    }

    #[test]
    fn test_suggestion_with_replacement() {
        let suggestion =
            Suggestion::with_replacement("Replace with this".to_string(), "new_value".to_string());
        assert_eq!(suggestion.message, "Replace with this");
        assert_eq!(suggestion.replacement, Some("new_value".to_string()));
    }

    #[test]
    fn test_diagnostic_to_codemirror_format() {
        let range = create_test_range();
        let diag = Diagnostic::error(
            "E001".to_string(),
            "Test error".to_string(),
            range,
            "test".to_string(),
        );

        let cm_diag = diag.to_codemirror_format();
        assert_eq!(cm_diag.from, 0);
        assert_eq!(cm_diag.to, 9);
        assert_eq!(cm_diag.severity, "error");
        assert_eq!(cm_diag.message, "Test error");
    }

    #[test]
    fn test_diagnostic_manager_new() {
        let manager = DiagnosticManager::new();
        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_diagnostic_manager_add() {
        let mut manager = DiagnosticManager::new();
        let range = create_test_range();
        let diag = Diagnostic::error(
            "E001".to_string(),
            "Test error".to_string(),
            range,
            "test".to_string(),
        );

        manager.add(diag);
        assert_eq!(manager.len(), 1);
        assert!(!manager.is_empty());
    }

    #[test]
    fn test_diagnostic_manager_add_all() {
        let mut manager = DiagnosticManager::new();
        let range = create_test_range();
        let diags = vec![
            Diagnostic::error(
                "E001".to_string(),
                "Error 1".to_string(),
                range,
                "test".to_string(),
            ),
            Diagnostic::warning(
                "W001".to_string(),
                "Warning 1".to_string(),
                range,
                "test".to_string(),
            ),
        ];

        manager.add_all(diags);
        assert_eq!(manager.len(), 2);
    }

    #[test]
    fn test_diagnostic_manager_clear() {
        let mut manager = DiagnosticManager::new();
        let range = create_test_range();
        let diag = Diagnostic::error(
            "E001".to_string(),
            "Test error".to_string(),
            range,
            "test".to_string(),
        );

        manager.add(diag);
        assert_eq!(manager.len(), 1);

        manager.clear();
        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_diagnostic_manager_get_all() {
        let mut manager = DiagnosticManager::new();
        let range = create_test_range();
        let diag = Diagnostic::error(
            "E001".to_string(),
            "Test error".to_string(),
            range,
            "test".to_string(),
        );

        manager.add(diag);
        let all = manager.get_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].code, "E001");
    }

    #[test]
    fn test_diagnostic_manager_filter_by_severity() {
        let mut manager = DiagnosticManager::new();
        let range = create_test_range();

        manager.add(Diagnostic::error(
            "E001".to_string(),
            "Error".to_string(),
            range,
            "test".to_string(),
        ));
        manager.add(Diagnostic::warning(
            "W001".to_string(),
            "Warning".to_string(),
            range,
            "test".to_string(),
        ));
        manager.add(Diagnostic::info(
            "I001".to_string(),
            "Info".to_string(),
            range,
            "test".to_string(),
        ));

        let errors = manager.filter_by_severity(Severity::Error);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E001");

        let warnings = manager.filter_by_severity(Severity::Warning);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W001");
    }

    #[test]
    fn test_diagnostic_manager_filter_by_min_severity() {
        let mut manager = DiagnosticManager::new();
        let range = create_test_range();

        manager.add(Diagnostic::error(
            "E001".to_string(),
            "Error".to_string(),
            range,
            "test".to_string(),
        ));
        manager.add(Diagnostic::warning(
            "W001".to_string(),
            "Warning".to_string(),
            range,
            "test".to_string(),
        ));
        manager.add(Diagnostic::info(
            "I001".to_string(),
            "Info".to_string(),
            range,
            "test".to_string(),
        ));

        let errors_and_warnings = manager.filter_by_min_severity(Severity::Warning);
        assert_eq!(errors_and_warnings.len(), 2);
    }

    #[test]
    fn test_diagnostic_manager_filter_by_source() {
        let mut manager = DiagnosticManager::new();
        let range = create_test_range();

        manager.add(Diagnostic::error(
            "E001".to_string(),
            "Error".to_string(),
            range,
            "parser".to_string(),
        ));
        manager.add(Diagnostic::warning(
            "W001".to_string(),
            "Warning".to_string(),
            range,
            "validator".to_string(),
        ));

        let parser_diags = manager.filter_by_source("parser");
        assert_eq!(parser_diags.len(), 1);
        assert_eq!(parser_diags[0].source, "parser");
    }

    #[test]
    fn test_diagnostic_manager_error_count() {
        let mut manager = DiagnosticManager::new();
        let range = create_test_range();

        manager.add(Diagnostic::error(
            "E001".to_string(),
            "Error 1".to_string(),
            range,
            "test".to_string(),
        ));
        manager.add(Diagnostic::error(
            "E002".to_string(),
            "Error 2".to_string(),
            range,
            "test".to_string(),
        ));
        manager.add(Diagnostic::warning(
            "W001".to_string(),
            "Warning".to_string(),
            range,
            "test".to_string(),
        ));

        assert_eq!(manager.error_count(), 2);
    }

    #[test]
    fn test_diagnostic_manager_warning_count() {
        let mut manager = DiagnosticManager::new();
        let range = create_test_range();

        manager.add(Diagnostic::error(
            "E001".to_string(),
            "Error".to_string(),
            range,
            "test".to_string(),
        ));
        manager.add(Diagnostic::warning(
            "W001".to_string(),
            "Warning 1".to_string(),
            range,
            "test".to_string(),
        ));
        manager.add(Diagnostic::warning(
            "W002".to_string(),
            "Warning 2".to_string(),
            range,
            "test".to_string(),
        ));

        assert_eq!(manager.warning_count(), 2);
    }

    #[test]
    fn test_diagnostic_manager_has_errors() {
        let mut manager = DiagnosticManager::new();
        let range = create_test_range();

        assert!(!manager.has_errors());

        manager.add(Diagnostic::warning(
            "W001".to_string(),
            "Warning".to_string(),
            range,
            "test".to_string(),
        ));
        assert!(!manager.has_errors());

        manager.add(Diagnostic::error(
            "E001".to_string(),
            "Error".to_string(),
            range,
            "test".to_string(),
        ));
        assert!(manager.has_errors());
    }

    #[test]
    fn test_diagnostic_manager_has_warnings() {
        let mut manager = DiagnosticManager::new();
        let range = create_test_range();

        assert!(!manager.has_warnings());

        manager.add(Diagnostic::error(
            "E001".to_string(),
            "Error".to_string(),
            range,
            "test".to_string(),
        ));
        assert!(!manager.has_warnings());

        manager.add(Diagnostic::warning(
            "W001".to_string(),
            "Warning".to_string(),
            range,
            "test".to_string(),
        ));
        assert!(manager.has_warnings());
    }

    #[test]
    fn test_diagnostic_manager_to_codemirror_format() {
        let mut manager = DiagnosticManager::new();
        let range = create_test_range();

        manager.add(Diagnostic::error(
            "E001".to_string(),
            "Error".to_string(),
            range,
            "test".to_string(),
        ));
        manager.add(Diagnostic::warning(
            "W001".to_string(),
            "Warning".to_string(),
            range,
            "test".to_string(),
        ));

        let cm_diags = manager.to_codemirror_format();
        assert_eq!(cm_diags.len(), 2);
        assert_eq!(cm_diags[0].severity, "error");
        assert_eq!(cm_diags[1].severity, "warning");
    }

    #[test]
    fn test_diagnostic_manager_sort_by_position() {
        let mut manager = DiagnosticManager::new();

        let range1 = Range::new(Position::new(2, 1, 10), Position::new(2, 5, 14));
        let range2 = Range::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
        let range3 = Range::new(Position::new(1, 10, 9), Position::new(1, 15, 14));

        manager.add(Diagnostic::error(
            "E001".to_string(),
            "Error 1".to_string(),
            range1,
            "test".to_string(),
        ));
        manager.add(Diagnostic::error(
            "E002".to_string(),
            "Error 2".to_string(),
            range2,
            "test".to_string(),
        ));
        manager.add(Diagnostic::error(
            "E003".to_string(),
            "Error 3".to_string(),
            range3,
            "test".to_string(),
        ));

        manager.sort_by_position();

        let all = manager.get_all();
        assert_eq!(all[0].code, "E002"); // Line 1, column 1
        assert_eq!(all[1].code, "E003"); // Line 1, column 10
        assert_eq!(all[2].code, "E001"); // Line 2, column 1
    }

    #[test]
    fn test_diagnostic_manager_to_json() {
        let mut manager = DiagnosticManager::new();
        let range = create_test_range();

        manager.add(Diagnostic::error(
            "E001".to_string(),
            "Error".to_string(),
            range,
            "test".to_string(),
        ));

        let json = manager.to_json();
        assert!(json.is_ok());
        assert!(json.unwrap().contains("E001"));
    }

    #[test]
    fn test_diagnostic_manager_from_json() {
        let json = r#"[
            {
                "code": "E001",
                "severity": "Error",
                "message": "Test error",
                "range": {
                    "start": {"line": 1, "column": 1, "offset": 0},
                    "end": {"line": 1, "column": 10, "offset": 9}
                },
                "source": "test",
                "suggestions": []
            }
        ]"#;

        let result = DiagnosticManager::from_json(json);
        assert!(result.is_ok());

        let manager = result.unwrap();
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.get_all()[0].code, "E001");
    }
}
