//! 格式化模块
//!
//! 提供将 AST 转换回格式化文本的功能

use crate::cwtools::models::{KeyValue, Operator, Statement, Value, AST};

/// 格式化配置
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// 缩进字符串（空格或制表符）
    pub indent: String,
    /// 是否在操作符周围添加空格
    pub space_around_operator: bool,
    /// 是否在左花括号前添加空格
    pub space_before_brace: bool,
    /// 是否保留空行
    pub preserve_empty_lines: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent: "\t".to_string(),
            space_around_operator: true,
            space_before_brace: true,
            preserve_empty_lines: true,
        }
    }
}

impl FormatConfig {
    /// 创建使用空格缩进的配置
    pub fn with_spaces(spaces: usize) -> Self {
        Self {
            indent: " ".repeat(spaces),
            ..Default::default()
        }
    }

    /// 创建使用制表符缩进的配置
    pub fn with_tabs() -> Self {
        Self {
            indent: "\t".to_string(),
            ..Default::default()
        }
    }
}

/// 格式化器
///
/// 将 AST 转换为格式化的文本
pub struct Formatter {
    config: FormatConfig,
}

impl Formatter {
    /// 创建新的格式化器
    pub fn new(config: FormatConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建格式化器
    pub fn default() -> Self {
        Self {
            config: FormatConfig::default(),
        }
    }

    /// 格式化 AST 为文本
    ///
    /// # 参数
    /// * `ast` - 要格式化的抽象语法树
    ///
    /// # 返回
    /// 格式化后的文本字符串
    pub fn format(&self, ast: &AST) -> String {
        let mut output = String::new();
        self.format_statements(&ast.statements, 0, &mut output);
        output
    }

    /// 格式化语句列表
    ///
    /// # 参数
    /// * `statements` - 语句列表
    /// * `indent_level` - 当前缩进层级
    /// * `output` - 输出字符串缓冲区
    fn format_statements(
        &self,
        statements: &[Statement],
        indent_level: usize,
        output: &mut String,
    ) {
        for (i, statement) in statements.iter().enumerate() {
            self.format_statement(statement, indent_level, output);

            // 在语句之间添加换行，但不在最后一个语句后添加
            if i < statements.len() - 1 {
                output.push('\n');
            }
        }
    }

    /// 格式化单个语句
    ///
    /// # 参数
    /// * `statement` - 要格式化的语句
    /// * `indent_level` - 当前缩进层级
    /// * `output` - 输出字符串缓冲区
    fn format_statement(&self, statement: &Statement, indent_level: usize, output: &mut String) {
        match statement {
            Statement::KeyValue(kv) => {
                self.format_key_value(kv, indent_level, output);
            }
            Statement::ValueOnly(value, _) => {
                self.add_indent(indent_level, output);
                self.format_value(value, indent_level, output);
            }
            Statement::Comment(comment, _) => {
                self.add_indent(indent_level, output);
                output.push_str(comment);
            }
        }
    }

    /// 格式化键值对
    ///
    /// # 参数
    /// * `kv` - 键值对
    /// * `indent_level` - 当前缩进层级
    /// * `output` - 输出字符串缓冲区
    fn format_key_value(&self, kv: &KeyValue, indent_level: usize, output: &mut String) {
        self.add_indent(indent_level, output);
        output.push_str(&kv.key);

        // 添加操作符
        if self.config.space_around_operator {
            output.push(' ');
        }
        output.push_str(self.format_operator(&kv.operator));

        // 对于子句，根据配置决定是否添加空格
        // 对于其他值，在操作符后添加空格（如果配置允许）
        match &kv.value {
            Value::Clause(statements) => {
                // 对于子句，如果 space_around_operator 为 true 且 space_before_brace 为 false，
                // 我们不添加空格；否则按照 space_around_operator 的设置
                if self.config.space_around_operator && self.config.space_before_brace {
                    output.push(' ');
                }
                self.format_clause(statements, indent_level, output, false);
            }
            _ => {
                if self.config.space_around_operator {
                    output.push(' ');
                }
                self.format_value(&kv.value, indent_level, output);
            }
        }
    }

    /// 格式化值
    ///
    /// # 参数
    /// * `value` - 要格式化的值
    /// * `indent_level` - 当前缩进层级
    /// * `output` - 输出字符串缓冲区
    fn format_value(&self, value: &Value, indent_level: usize, output: &mut String) {
        match value {
            Value::String(s) => {
                output.push_str(s);
            }
            Value::QuotedString(s) => {
                output.push('"');
                output.push_str(s);
                output.push('"');
            }
            Value::Integer(i) => {
                output.push_str(&i.to_string());
            }
            Value::Float(f) => {
                output.push_str(&f.to_string());
            }
            Value::Boolean(b) => {
                output.push_str(if *b { "yes" } else { "no" });
            }
            Value::Clause(statements) => {
                // 不在这里添加空格，因为调用者（format_key_value）已经处理了操作符后的空格
                self.format_clause(statements, indent_level, output, false);
            }
        }
    }

    /// 格式化子句（花括号包围的语句块）
    ///
    /// # 参数
    /// * `statements` - 子句中的语句列表
    /// * `indent_level` - 当前缩进层级
    /// * `output` - 输出字符串缓冲区
    /// * `add_space_before` - 是否在花括号前添加空格（用于避免重复空格）
    fn format_clause(
        &self,
        statements: &[Statement],
        indent_level: usize,
        output: &mut String,
        add_space_before: bool,
    ) {
        if add_space_before && self.config.space_before_brace {
            output.push(' ');
        }
        output.push('{');

        if !statements.is_empty() {
            output.push('\n');
            self.format_statements(statements, indent_level + 1, output);
            output.push('\n');
            self.add_indent(indent_level, output);
        }

        output.push('}');
    }

    /// 格式化操作符
    ///
    /// # 参数
    /// * `operator` - 操作符
    ///
    /// # 返回
    /// 操作符的字符串表示
    fn format_operator(&self, operator: &Operator) -> &'static str {
        operator.as_str()
    }

    /// 添加缩进
    ///
    /// # 参数
    /// * `level` - 缩进层级
    /// * `output` - 输出字符串缓冲区
    fn add_indent(&self, level: usize, output: &mut String) {
        for _ in 0..level {
            output.push_str(&self.config.indent);
        }
    }
}

/// 格式化脚本的便捷函数
///
/// 使用默认配置格式化 AST
///
/// # 参数
/// * `ast` - 要格式化的抽象语法树
///
/// # 返回
/// 格式化后的文本字符串
pub fn format_script(ast: &AST) -> String {
    let formatter = Formatter::default();
    formatter.format(ast)
}

/// 使用自定义配置格式化脚本
///
/// # 参数
/// * `ast` - 要格式化的抽象语法树
/// * `config` - 格式化配置
///
/// # 返回
/// 格式化后的文本字符串
pub fn format_script_with_config(ast: &AST, config: FormatConfig) -> String {
    let formatter = Formatter::new(config);
    formatter.format(ast)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cwtools::models::Position;

    #[test]
    fn test_format_simple_key_value() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let kv = KeyValue::new(
            "key".to_string(),
            Operator::Equals,
            Value::String("value".to_string()),
            pos,
        );
        ast.add_statement(Statement::KeyValue(kv));

        let formatted = format_script(&ast);
        assert_eq!(formatted, "key = value");
    }

    #[test]
    fn test_format_integer_value() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let kv = KeyValue::new(
            "count".to_string(),
            Operator::Equals,
            Value::Integer(42),
            pos,
        );
        ast.add_statement(Statement::KeyValue(kv));

        let formatted = format_script(&ast);
        assert_eq!(formatted, "count = 42");
    }

    #[test]
    fn test_format_float_value() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let kv = KeyValue::new(
            "factor".to_string(),
            Operator::Equals,
            Value::Float(3.14),
            pos,
        );
        ast.add_statement(Statement::KeyValue(kv));

        let formatted = format_script(&ast);
        assert_eq!(formatted, "factor = 3.14");
    }

    #[test]
    fn test_format_boolean_value() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let kv1 = KeyValue::new(
            "enabled".to_string(),
            Operator::Equals,
            Value::Boolean(true),
            pos,
        );
        let kv2 = KeyValue::new(
            "disabled".to_string(),
            Operator::Equals,
            Value::Boolean(false),
            pos,
        );

        ast.add_statement(Statement::KeyValue(kv1));
        ast.add_statement(Statement::KeyValue(kv2));

        let formatted = format_script(&ast);
        assert_eq!(formatted, "enabled = yes\ndisabled = no");
    }

    #[test]
    fn test_format_quoted_string() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let kv = KeyValue::new(
            "name".to_string(),
            Operator::Equals,
            Value::QuotedString("Test Name".to_string()),
            pos,
        );
        ast.add_statement(Statement::KeyValue(kv));

        let formatted = format_script(&ast);
        assert_eq!(formatted, "name = \"Test Name\"");
    }

    #[test]
    fn test_format_empty_clause() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let kv = KeyValue::new(
            "block".to_string(),
            Operator::Equals,
            Value::Clause(vec![]),
            pos,
        );
        ast.add_statement(Statement::KeyValue(kv));

        let formatted = format_script(&ast);
        assert_eq!(formatted, "block = {}");
    }

    #[test]
    fn test_format_clause_with_content() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let inner_kv = KeyValue::new(
            "inner".to_string(),
            Operator::Equals,
            Value::Integer(42),
            pos,
        );

        let outer_kv = KeyValue::new(
            "outer".to_string(),
            Operator::Equals,
            Value::Clause(vec![Statement::KeyValue(inner_kv)]),
            pos,
        );

        ast.add_statement(Statement::KeyValue(outer_kv));

        let formatted = format_script(&ast);
        assert_eq!(formatted, "outer = {\n\tinner = 42\n}");
    }

    #[test]
    fn test_format_nested_clauses() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let innermost_kv = KeyValue::new(
            "value".to_string(),
            Operator::Equals,
            Value::String("test".to_string()),
            pos,
        );

        let middle_kv = KeyValue::new(
            "middle".to_string(),
            Operator::Equals,
            Value::Clause(vec![Statement::KeyValue(innermost_kv)]),
            pos,
        );

        let outer_kv = KeyValue::new(
            "outer".to_string(),
            Operator::Equals,
            Value::Clause(vec![Statement::KeyValue(middle_kv)]),
            pos,
        );

        ast.add_statement(Statement::KeyValue(outer_kv));

        let formatted = format_script(&ast);
        assert_eq!(
            formatted,
            "outer = {\n\tmiddle = {\n\t\tvalue = test\n\t}\n}"
        );
    }

    #[test]
    fn test_format_comment() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        ast.add_statement(Statement::Comment("# This is a comment".to_string(), pos));

        let formatted = format_script(&ast);
        assert_eq!(formatted, "# This is a comment");
    }

    #[test]
    fn test_format_multiple_statements() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let kv1 = KeyValue::new(
            "key1".to_string(),
            Operator::Equals,
            Value::String("value1".to_string()),
            pos,
        );
        let kv2 = KeyValue::new(
            "key2".to_string(),
            Operator::Equals,
            Value::Integer(42),
            pos,
        );

        ast.add_statement(Statement::KeyValue(kv1));
        ast.add_statement(Statement::Comment("# Comment".to_string(), pos));
        ast.add_statement(Statement::KeyValue(kv2));

        let formatted = format_script(&ast);
        assert_eq!(formatted, "key1 = value1\n# Comment\nkey2 = 42");
    }

    #[test]
    fn test_format_value_only() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        ast.add_statement(Statement::ValueOnly(
            Value::String("standalone".to_string()),
            pos,
        ));

        let formatted = format_script(&ast);
        assert_eq!(formatted, "standalone");
    }

    #[test]
    fn test_format_different_operators() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let operators = vec![
            Operator::Equals,
            Operator::GreaterThan,
            Operator::LessThan,
            Operator::GreaterEqual,
            Operator::LessEqual,
            Operator::NotEqual,
            Operator::EqualEqual,
            Operator::QuestionEqual,
        ];

        for (i, op) in operators.iter().enumerate() {
            let kv = KeyValue::new(format!("key{}", i), *op, Value::Integer(i as i64), pos);
            ast.add_statement(Statement::KeyValue(kv));
        }

        let formatted = format_script(&ast);
        let lines: Vec<&str> = formatted.lines().collect();

        assert_eq!(lines[0], "key0 = 0");
        assert_eq!(lines[1], "key1 > 1");
        assert_eq!(lines[2], "key2 < 2");
        assert_eq!(lines[3], "key3 >= 3");
        assert_eq!(lines[4], "key4 <= 4");
        assert_eq!(lines[5], "key5 != 5");
        assert_eq!(lines[6], "key6 == 6");
        assert_eq!(lines[7], "key7 ?= 7");
    }

    #[test]
    fn test_format_config_with_spaces() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let inner_kv = KeyValue::new(
            "inner".to_string(),
            Operator::Equals,
            Value::Integer(42),
            pos,
        );

        let outer_kv = KeyValue::new(
            "outer".to_string(),
            Operator::Equals,
            Value::Clause(vec![Statement::KeyValue(inner_kv)]),
            pos,
        );

        ast.add_statement(Statement::KeyValue(outer_kv));

        let config = FormatConfig::with_spaces(4);
        let formatted = format_script_with_config(&ast, config);

        assert_eq!(formatted, "outer = {\n    inner = 42\n}");
    }

    #[test]
    fn test_format_config_no_space_around_operator() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let kv = KeyValue::new(
            "key".to_string(),
            Operator::Equals,
            Value::String("value".to_string()),
            pos,
        );
        ast.add_statement(Statement::KeyValue(kv));

        let config = FormatConfig {
            space_around_operator: false,
            ..Default::default()
        };
        let formatted = format_script_with_config(&ast, config);

        assert_eq!(formatted, "key=value");
    }

    #[test]
    fn test_format_config_no_space_before_brace() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let inner_kv = KeyValue::new(
            "inner".to_string(),
            Operator::Equals,
            Value::Integer(42),
            pos,
        );

        let outer_kv = KeyValue::new(
            "outer".to_string(),
            Operator::Equals,
            Value::Clause(vec![Statement::KeyValue(inner_kv)]),
            pos,
        );

        ast.add_statement(Statement::KeyValue(outer_kv));

        let config = FormatConfig {
            space_before_brace: false,
            ..Default::default()
        };
        let formatted = format_script_with_config(&ast, config);

        assert_eq!(formatted, "outer ={\n\tinner = 42\n}");
    }
}
