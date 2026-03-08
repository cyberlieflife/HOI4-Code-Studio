//! 语法分析器模块
//!
//! 负责将 Token 流转换为抽象语法树（AST）

use crate::cwtools::models::{KeyValue, Operator, Position, Statement, Token, Value, AST};
use crate::cwtools::parser::lexer::{LexError, Lexer};
use std::fmt;

/// 解析错误类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParseErrorType {
    /// 意外的 Token
    UnexpectedToken,
    /// 意外的文件结束
    UnexpectedEof,
    /// 无效的操作符
    InvalidOperator,
    /// 未闭合的花括号
    UnclosedBrace,
    /// 无效的值
    InvalidValue,
}

/// 解析错误
#[derive(Debug, Clone)]
pub struct ParseError {
    /// 错误消息
    pub message: String,
    /// 错误位置
    pub position: Position,
    /// 错误类型
    pub error_type: ParseErrorType,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}, column {}: {}",
            self.position.line, self.position.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    /// 创建新的解析错误
    ///
    /// # 参数
    /// * `message` - 错误消息
    /// * `position` - 错误位置
    /// * `error_type` - 错误类型
    ///
    /// # 返回
    /// 新的 ParseError 实例
    pub fn new(message: String, position: Position, error_type: ParseErrorType) -> Self {
        Self {
            message,
            position,
            error_type,
        }
    }

    /// 转换为诊断信息
    ///
    /// # 返回
    /// Diagnostic 实例
    pub fn to_diagnostic(&self) -> crate::cwtools::diagnostic::Diagnostic {
        use crate::cwtools::diagnostic::Severity;
        use crate::cwtools::models::Range;

        crate::cwtools::diagnostic::Diagnostic::new(
            format!("P{:03}", self.error_type as u32),
            Severity::Error,
            self.message.clone(),
            Range::point(self.position),
            "parser".to_string(),
        )
    }
}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        ParseError {
            message: err.message,
            position: err.position,
            error_type: ParseErrorType::InvalidValue,
        }
    }
}

/// 语法分析器
///
/// 将 Token 流转换为 AST，支持错误恢复
pub struct Parser<'a> {
    /// 词法分析器
    lexer: Lexer<'a>,
    /// 当前 Token
    current_token: Token,
    /// 收集的错误列表
    errors: Vec<ParseError>,
    /// 源文件路径
    source_file: String,
}

impl<'a> Parser<'a> {
    /// 创建新的语法分析器
    ///
    /// # 参数
    /// * `input` - 输入文本
    /// * `source_file` - 源文件路径
    ///
    /// # 返回
    /// 新的 Parser 实例
    pub fn new(input: &'a str, source_file: String) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token()?;

        Ok(Self {
            lexer,
            current_token,
            errors: Vec::new(),
            source_file,
        })
    }

    /// 解析输入文本，生成 AST
    ///
    /// # 返回
    /// * `Ok(AST)` - 成功解析的 AST
    /// * `Err(Vec<ParseError>)` - 解析错误列表
    pub fn parse(&mut self) -> Result<AST, Vec<ParseError>> {
        let mut ast = AST::new(self.source_file.clone());

        // 跳过开头的换行符
        self.skip_newlines();

        // 解析所有顶层语句
        while !self.is_at_end() {
            match self.parse_statement() {
                Ok(statement) => {
                    ast.add_statement(statement);
                }
                Err(err) => {
                    self.errors.push(err);
                    // 错误恢复：跳到下一个语句
                    self.synchronize();
                }
            }

            // 跳过语句之间的换行符
            self.skip_newlines();
        }

        // 如果有错误，返回错误列表
        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }

        Ok(ast)
    }

    /// 解析单个语句
    ///
    /// 语句可以是：
    /// - 键值对：key = value
    /// - 单独的值：value
    /// - 注释：# comment
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let start_pos = self.current_position();

        // 处理注释
        if let Token::Comment(text) = &self.current_token {
            let comment = text.clone();
            self.advance()?;
            return Ok(Statement::Comment(comment, start_pos));
        }

        // 尝试解析键值对
        // 预读下一个 token 来判断是否为键值对
        let next_token = self.lexer.peek_token()?;

        if matches!(next_token, Token::Operator(_)) {
            // 这是一个键值对
            return self.parse_key_value();
        }

        // 否则解析为单独的值
        let value = self.parse_value()?;
        Ok(Statement::ValueOnly(value, start_pos))
    }

    /// 解析键值对
    ///
    /// 格式：key operator value
    fn parse_key_value(&mut self) -> Result<Statement, ParseError> {
        let start_pos = self.current_position();

        // 解析键名
        let key = match &self.current_token {
            Token::Identifier(s) | Token::String(s) => s.clone(),
            _ => {
                return Err(ParseError {
                    message: format!(
                        "Expected identifier or string, found {:?}",
                        self.current_token
                    ),
                    position: start_pos,
                    error_type: ParseErrorType::UnexpectedToken,
                });
            }
        };

        self.advance()?;

        // 解析操作符
        let operator = match &self.current_token {
            Token::Operator(op) => *op,
            _ => {
                return Err(ParseError {
                    message: format!("Expected operator, found {:?}", self.current_token),
                    position: self.current_position(),
                    error_type: ParseErrorType::InvalidOperator,
                });
            }
        };

        self.advance()?;

        // 解析值
        let value = self.parse_value()?;

        let kv = KeyValue::new(key, operator, value, start_pos);
        Ok(Statement::KeyValue(kv))
    }

    /// 解析值
    ///
    /// 值可以是：
    /// - 字符串（带引号或不带引号）
    /// - 数值（整数或浮点数）
    /// - 布尔值
    /// - 子句（花括号包围的语句块）
    fn parse_value(&mut self) -> Result<Value, ParseError> {
        match &self.current_token {
            Token::String(s) => {
                let value = Value::String(s.clone());
                self.advance()?;
                Ok(value)
            }
            Token::QuotedString(s) => {
                let value = Value::QuotedString(s.clone());
                self.advance()?;
                Ok(value)
            }
            Token::Identifier(s) => {
                let value = Value::String(s.clone());
                self.advance()?;
                Ok(value)
            }
            Token::Integer(i) => {
                let value = Value::Integer(*i);
                self.advance()?;
                Ok(value)
            }
            Token::Float(f) => {
                let value = Value::Float(*f);
                self.advance()?;
                Ok(value)
            }
            Token::Boolean(b) => {
                let value = Value::Boolean(*b);
                self.advance()?;
                Ok(value)
            }
            Token::LeftBrace => {
                // 解析子句
                self.parse_clause()
            }
            _ => Err(ParseError {
                message: format!("Expected value, found {:?}", self.current_token),
                position: self.current_position(),
                error_type: ParseErrorType::InvalidValue,
            }),
        }
    }

    /// 解析子句（花括号包围的语句块）
    ///
    /// 格式：{ statement1 statement2 ... }
    fn parse_clause(&mut self) -> Result<Value, ParseError> {
        let start_pos = self.current_position();

        // 消费左花括号
        if !matches!(self.current_token, Token::LeftBrace) {
            return Err(ParseError {
                message: format!("Expected '{{', found {:?}", self.current_token),
                position: start_pos,
                error_type: ParseErrorType::UnexpectedToken,
            });
        }

        self.advance()?;
        self.skip_newlines();

        let mut statements = Vec::new();

        // 解析子句中的所有语句
        while !matches!(self.current_token, Token::RightBrace | Token::Eof) {
            match self.parse_statement() {
                Ok(statement) => {
                    statements.push(statement);
                }
                Err(err) => {
                    self.errors.push(err);
                    // 错误恢复：跳到下一个语句或右花括号
                    self.recover_from_error();
                }
            }

            self.skip_newlines();
        }

        // 消费右花括号
        if !matches!(self.current_token, Token::RightBrace) {
            return Err(ParseError {
                message: "Expected '}', found end of file".to_string(),
                position: self.current_position(),
                error_type: ParseErrorType::UnclosedBrace,
            });
        }

        self.advance()?;

        Ok(Value::Clause(statements))
    }

    /// 错误恢复：跳到下一个语句的开始
    ///
    /// 在子句中遇到错误时，跳到右花括号或下一个可能的语句开始
    fn recover_from_error(&mut self) {
        // 跳过当前行的剩余内容
        while !matches!(
            self.current_token,
            Token::Newline | Token::RightBrace | Token::Eof
        ) {
            if self.advance().is_err() {
                break;
            }
        }

        // 跳过换行符
        if matches!(self.current_token, Token::Newline) {
            let _ = self.advance();
        }
    }

    /// 同步到下一个顶层语句
    ///
    /// 在顶层遇到错误时使用，跳到下一个可能的语句开始
    fn synchronize(&mut self) {
        // 跳过当前语句的剩余部分
        let mut brace_depth = 0;

        while !self.is_at_end() {
            match &self.current_token {
                Token::LeftBrace => {
                    brace_depth += 1;
                    if self.advance().is_err() {
                        break;
                    }
                }
                Token::RightBrace => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                    }
                    if self.advance().is_err() {
                        break;
                    }
                }
                Token::Newline => {
                    if self.advance().is_err() {
                        break;
                    }
                    // 如果不在花括号内，换行后可以开始新语句
                    if brace_depth == 0 {
                        break;
                    }
                }
                _ => {
                    if self.advance().is_err() {
                        break;
                    }
                }
            }
        }
    }

    /// 前进到下一个 Token
    fn advance(&mut self) -> Result<(), ParseError> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    /// 跳过所有换行符
    fn skip_newlines(&mut self) {
        while matches!(self.current_token, Token::Newline) {
            if self.advance().is_err() {
                break;
            }
        }
    }

    /// 判断是否到达输入末尾
    fn is_at_end(&self) -> bool {
        matches!(self.current_token, Token::Eof)
    }

    /// 获取当前位置
    fn current_position(&self) -> Position {
        // 从 lexer 获取当前位置
        // 注意：这是一个简化实现，实际应该跟踪每个 token 的位置
        Position::new(1, 1, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_input() {
        let mut parser = Parser::new("", "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();
        assert_eq!(ast.statements.len(), 0);
    }

    #[test]
    fn test_parse_simple_key_value() {
        let input = "key = value";
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            Statement::KeyValue(kv) => {
                assert_eq!(kv.key, "key");
                assert_eq!(kv.operator, Operator::Equals);
                match &kv.value {
                    Value::String(s) => assert_eq!(s, "value"),
                    _ => panic!("Expected string value"),
                }
            }
            _ => panic!("Expected KeyValue statement"),
        }
    }

    #[test]
    fn test_parse_integer_value() {
        let input = "count = 42";
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            Statement::KeyValue(kv) => {
                assert_eq!(kv.key, "count");
                match &kv.value {
                    Value::Integer(i) => assert_eq!(*i, 42),
                    _ => panic!("Expected integer value"),
                }
            }
            _ => panic!("Expected KeyValue statement"),
        }
    }

    #[test]
    fn test_parse_float_value() {
        let input = "factor = 1.5";
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            Statement::KeyValue(kv) => {
                assert_eq!(kv.key, "factor");
                match &kv.value {
                    Value::Float(f) => assert_eq!(*f, 1.5),
                    _ => panic!("Expected float value"),
                }
            }
            _ => panic!("Expected KeyValue statement"),
        }
    }

    #[test]
    fn test_parse_boolean_value() {
        let input = "enabled = yes";
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            Statement::KeyValue(kv) => {
                assert_eq!(kv.key, "enabled");
                match &kv.value {
                    Value::Boolean(b) => assert_eq!(*b, true),
                    _ => panic!("Expected boolean value"),
                }
            }
            _ => panic!("Expected KeyValue statement"),
        }
    }

    #[test]
    fn test_parse_quoted_string() {
        let input = r#"title = "Test Title""#;
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            Statement::KeyValue(kv) => {
                assert_eq!(kv.key, "title");
                match &kv.value {
                    Value::QuotedString(s) => assert_eq!(s, "Test Title"),
                    _ => panic!("Expected quoted string value"),
                }
            }
            _ => panic!("Expected KeyValue statement"),
        }
    }

    #[test]
    fn test_parse_comment() {
        let input = "# This is a comment";
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            Statement::Comment(text, _) => {
                assert_eq!(text, "# This is a comment");
            }
            _ => panic!("Expected Comment statement"),
        }
    }

    #[test]
    fn test_parse_simple_clause() {
        let input = r#"
option = {
    name = test
    value = 42
}
"#;
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            Statement::KeyValue(kv) => {
                assert_eq!(kv.key, "option");
                match &kv.value {
                    Value::Clause(statements) => {
                        assert_eq!(statements.len(), 2);
                    }
                    _ => panic!("Expected clause value"),
                }
            }
            _ => panic!("Expected KeyValue statement"),
        }
    }

    #[test]
    fn test_parse_nested_clause() {
        let input = r#"
outer = {
    inner = {
        value = 42
    }
}
"#;
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            Statement::KeyValue(kv) => {
                assert_eq!(kv.key, "outer");
                match &kv.value {
                    Value::Clause(outer_statements) => {
                        assert_eq!(outer_statements.len(), 1);
                        match &outer_statements[0] {
                            Statement::KeyValue(inner_kv) => {
                                assert_eq!(inner_kv.key, "inner");
                                match &inner_kv.value {
                                    Value::Clause(inner_statements) => {
                                        assert_eq!(inner_statements.len(), 1);
                                    }
                                    _ => panic!("Expected inner clause"),
                                }
                            }
                            _ => panic!("Expected inner KeyValue"),
                        }
                    }
                    _ => panic!("Expected outer clause"),
                }
            }
            _ => panic!("Expected outer KeyValue"),
        }
    }

    #[test]
    fn test_parse_multiple_statements() {
        let input = r#"
key1 = value1
key2 = 42
key3 = yes
"#;
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 3);
    }

    #[test]
    fn test_parse_operators() {
        let input = r#"
a = 1
b > 2
c < 3
d >= 4
e <= 5
f != 6
g == 7
h ?= 8
"#;
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 8);

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
            match &ast.statements[i] {
                Statement::KeyValue(kv) => {
                    assert_eq!(kv.operator, *op);
                }
                _ => panic!("Expected KeyValue statement"),
            }
        }
    }

    #[test]
    fn test_parse_value_only() {
        let input = r#"
list = {
    value1
    value2
    42
}
"#;
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            Statement::KeyValue(kv) => {
                match &kv.value {
                    Value::Clause(statements) => {
                        assert_eq!(statements.len(), 3);
                        // 验证都是 ValueOnly 语句
                        for stmt in statements {
                            assert!(matches!(stmt, Statement::ValueOnly(_, _)));
                        }
                    }
                    _ => panic!("Expected clause"),
                }
            }
            _ => panic!("Expected KeyValue"),
        }
    }

    #[test]
    fn test_parse_with_comments() {
        let input = r#"
# Header comment
key1 = value1
# Middle comment
key2 = value2
"#;
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 4);
        assert!(matches!(ast.statements[0], Statement::Comment(_, _)));
        assert!(matches!(ast.statements[1], Statement::KeyValue(_)));
        assert!(matches!(ast.statements[2], Statement::Comment(_, _)));
        assert!(matches!(ast.statements[3], Statement::KeyValue(_)));
    }

    #[test]
    fn test_parse_error_recovery_unclosed_brace() {
        let input = r#"
key1 = {
    value = 42
# Missing closing brace
key2 = value2
"#;
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let result = parser.parse();

        // 应该有错误
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_parse_error_recovery_invalid_syntax() {
        let input = r#"
key1 = value1
= invalid
key2 = value2
"#;
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let result = parser.parse();

        // 应该有错误，但能继续解析
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_complex_script() {
        let input = r#"
country_event = {
    id = test.1
    title = "Test Event"
    desc = test.1.desc
    
    # Event option
    option = {
        name = test.1.a
        add_stability = 0.05
    }
    
    option = {
        name = test.1.b
        add_political_power = 50
    }
}
"#;
        let mut parser = Parser::new(input, "test.txt".to_string()).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            Statement::KeyValue(kv) => {
                assert_eq!(kv.key, "country_event");
                match &kv.value {
                    Value::Clause(statements) => {
                        // 应该有多个语句（包括注释）
                        assert!(statements.len() >= 5);
                    }
                    _ => panic!("Expected clause"),
                }
            }
            _ => panic!("Expected KeyValue"),
        }
    }
}
