//! 核心数据模型定义
//!
//! 包含 AST、Token、Position 等基础数据结构

use serde::{Deserialize, Serialize};

/// 位置信息，表示源代码中的一个点
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// 行号（从 1 开始）
    pub line: usize,
    /// 列号（从 1 开始）
    pub column: usize,
    /// 字节偏移量（从 0 开始）
    pub offset: usize,
}

#[allow(dead_code)]
impl Position {
    /// 创建新的位置
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// 创建起始位置
    pub fn start() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

/// 范围信息，表示源代码中的一个区间
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    /// 起始位置
    pub start: Position,
    /// 结束位置
    pub end: Position,
}

#[allow(dead_code)]
impl Range {
    /// 创建新的范围
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// 创建单点范围
    pub fn point(pos: Position) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }
}

/// 词法单元类型
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// 标识符
    Identifier(String),
    /// 不带引号的字符串
    String(String),
    /// 带引号的字符串
    QuotedString(String),
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 布尔值
    Boolean(bool),
    /// 操作符
    Operator(Operator),
    /// 左花括号 {
    LeftBrace,
    /// 右花括号 }
    RightBrace,
    /// 注释
    Comment(String),
    /// 换行符
    Newline,
    /// 文件结束
    Eof,
}

/// 操作符类型
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operator {
    /// = 赋值
    Equals,
    /// > 大于
    GreaterThan,
    /// < 小于
    LessThan,
    /// >= 大于等于
    GreaterEqual,
    /// <= 小于等于
    LessEqual,
    /// != 不等于
    NotEqual,
    /// == 等于
    EqualEqual,
    /// ?= 条件赋值
    QuestionEqual,
}

#[allow(dead_code)]
impl Operator {
    /// 从字符串解析操作符
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "=" => Some(Operator::Equals),
            ">" => Some(Operator::GreaterThan),
            "<" => Some(Operator::LessThan),
            ">=" => Some(Operator::GreaterEqual),
            "<=" => Some(Operator::LessEqual),
            "!=" => Some(Operator::NotEqual),
            "==" => Some(Operator::EqualEqual),
            "?=" => Some(Operator::QuestionEqual),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Operator::Equals => "=",
            Operator::GreaterThan => ">",
            Operator::LessThan => "<",
            Operator::GreaterEqual => ">=",
            Operator::LessEqual => "<=",
            Operator::NotEqual => "!=",
            Operator::EqualEqual => "==",
            Operator::QuestionEqual => "?=",
        }
    }
}

/// 抽象语法树
///
/// 表示解析后的 Paradox 脚本文件的完整结构
#[allow(dead_code, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AST {
    /// 顶层语句列表
    pub statements: Vec<Statement>,
    /// 源文件路径
    pub source_file: String,
}

#[allow(dead_code)]
impl AST {
    /// 创建新的 AST
    pub fn new(source_file: String) -> Self {
        Self {
            statements: Vec::new(),
            source_file,
        }
    }

    /// 添加语句
    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }

    /// 遍历所有语句
    ///
    /// 使用访问者模式遍历 AST 中的所有语句节点
    pub fn traverse<F>(&self, visitor: &mut F)
    where
        F: FnMut(&Statement),
    {
        for statement in &self.statements {
            self.traverse_statement(statement, visitor);
        }
    }

    /// 递归遍历单个语句及其子节点
    #[allow(clippy::only_used_in_recursion)]
    fn traverse_statement<F>(&self, statement: &Statement, visitor: &mut F)
    where
        F: FnMut(&Statement),
    {
        visitor(statement);

        match statement {
            Statement::KeyValue(kv) => {
                if let Value::Clause(statements) = &kv.value {
                    for stmt in statements {
                        self.traverse_statement(stmt, visitor);
                    }
                }
            }
            Statement::ValueOnly(value, _) => {
                if let Value::Clause(statements) = value {
                    for stmt in statements {
                        self.traverse_statement(stmt, visitor);
                    }
                }
            }
            Statement::Comment(_, _) => {}
        }
    }

    /// 查找指定位置的语句
    ///
    /// 返回包含指定位置的最内层语句
    pub fn find_at_position(&self, pos: Position) -> Option<&Statement> {
        self.find_at_position_in_statements(&self.statements, pos)
    }

    /// 在语句列表中查找指定位置的语句
    fn find_at_position_in_statements<'a>(
        &self,
        statements: &'a [Statement],
        pos: Position,
    ) -> Option<&'a Statement> {
        for statement in statements {
            if let Some(found) = self.find_at_position_in_statement(statement, pos) {
                return Some(found);
            }
        }
        None
    }

    /// 在单个语句中查找指定位置
    fn find_at_position_in_statement<'a>(
        &self,
        statement: &'a Statement,
        pos: Position,
    ) -> Option<&'a Statement> {
        let statement_pos = match statement {
            Statement::KeyValue(kv) => kv.position,
            Statement::ValueOnly(_, p) => *p,
            Statement::Comment(_, p) => *p,
        };

        if !self.position_in_range(pos, statement_pos) {
            return None;
        }

        match statement {
            Statement::KeyValue(kv) => {
                if let Value::Clause(statements) = &kv.value {
                    if let Some(found) = self.find_at_position_in_statements(statements, pos) {
                        return Some(found);
                    }
                }
            }
            Statement::ValueOnly(value, _) => {
                if let Value::Clause(statements) = value {
                    if let Some(found) = self.find_at_position_in_statements(statements, pos) {
                        return Some(found);
                    }
                }
            }
            Statement::Comment(_, _) => {}
        }

        Some(statement)
    }

    /// 检查位置是否在范围内（简化版本，只检查行号）
    fn position_in_range(&self, pos: Position, statement_pos: Position) -> bool {
        pos.line >= statement_pos.line
    }
}

/// 语句类型
///
/// 表示 Paradox 脚本中的一个语句，可以是键值对、单独的值或注释
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    /// 键值对语句，如 `key = value`
    KeyValue(KeyValue),
    /// 仅包含值的语句，如列表中的单个值
    ValueOnly(Value, Position),
    /// 注释语句
    Comment(String, Position),
}

/// 键值对
///
/// 表示 Paradox 脚本中的键值对结构
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
    /// 键名
    pub key: String,
    /// 操作符
    pub operator: Operator,
    /// 值
    pub value: Value,
    /// 位置信息
    pub position: Position,
}

#[allow(dead_code)]
impl KeyValue {
    /// 创建新的键值对
    pub fn new(key: String, operator: Operator, value: Value, position: Position) -> Self {
        Self {
            key,
            operator,
            value,
            position,
        }
    }
}

/// 值类型
///
/// 表示 Paradox 脚本中的各种值类型
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    /// 不带引号的字符串
    String(String),
    /// 带引号的字符串
    QuotedString(String),
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 布尔值
    Boolean(bool),
    /// 子句（花括号包围的语句块）
    Clause(Vec<Statement>),
}

#[allow(dead_code)]
impl Value {
    /// 判断值是否为子句
    pub fn is_clause(&self) -> bool {
        matches!(self, Value::Clause(_))
    }

    /// 获取字符串值（如果是字符串类型）
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) | Value::QuotedString(s) => Some(s),
            _ => None,
        }
    }

    /// 获取整数值（如果是整数类型）
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// 获取浮点数值（如果是浮点数类型）
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// 获取布尔值（如果是布尔类型）
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// 获取子句（如果是子句类型）
    pub fn as_clause(&self) -> Option<&[Statement]> {
        match self {
            Value::Clause(statements) => Some(statements),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(10, 5, 100);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.offset, 100);
    }

    #[test]
    fn test_position_start() {
        let pos = Position::start();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 0);
    }

    #[test]
    fn test_range_creation() {
        let start = Position::new(1, 1, 0);
        let end = Position::new(1, 10, 9);
        let range = Range::new(start, end);
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    fn test_range_point() {
        let pos = Position::new(5, 10, 50);
        let range = Range::point(pos);
        assert_eq!(range.start, pos);
        assert_eq!(range.end, pos);
    }

    #[test]
    fn test_operator_from_str() {
        assert_eq!(Operator::from_str("="), Some(Operator::Equals));
        assert_eq!(Operator::from_str(">"), Some(Operator::GreaterThan));
        assert_eq!(Operator::from_str("<"), Some(Operator::LessThan));
        assert_eq!(Operator::from_str(">="), Some(Operator::GreaterEqual));
        assert_eq!(Operator::from_str("<="), Some(Operator::LessEqual));
        assert_eq!(Operator::from_str("!="), Some(Operator::NotEqual));
        assert_eq!(Operator::from_str("=="), Some(Operator::EqualEqual));
        assert_eq!(Operator::from_str("?="), Some(Operator::QuestionEqual));
        assert_eq!(Operator::from_str("invalid"), None);
    }

    #[test]
    fn test_operator_as_str() {
        assert_eq!(Operator::Equals.as_str(), "=");
        assert_eq!(Operator::GreaterThan.as_str(), ">");
        assert_eq!(Operator::LessThan.as_str(), "<");
        assert_eq!(Operator::GreaterEqual.as_str(), ">=");
        assert_eq!(Operator::LessEqual.as_str(), "<=");
        assert_eq!(Operator::NotEqual.as_str(), "!=");
        assert_eq!(Operator::EqualEqual.as_str(), "==");
        assert_eq!(Operator::QuestionEqual.as_str(), "?=");
    }

    #[test]
    fn test_token_types() {
        let tokens = vec![
            Token::Identifier("test".to_string()),
            Token::String("value".to_string()),
            Token::QuotedString("quoted".to_string()),
            Token::Integer(42),
            Token::Float(3.14),
            Token::Boolean(true),
            Token::Operator(Operator::Equals),
            Token::LeftBrace,
            Token::RightBrace,
            Token::Comment("# comment".to_string()),
            Token::Newline,
            Token::Eof,
        ];

        assert_eq!(tokens.len(), 12);
    }

    #[test]
    fn test_ast_creation() {
        let ast = AST::new("test.txt".to_string());
        assert_eq!(ast.source_file, "test.txt");
        assert_eq!(ast.statements.len(), 0);
    }

    #[test]
    fn test_ast_add_statement() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);
        let statement = Statement::Comment("test comment".to_string(), pos);
        ast.add_statement(statement);
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_keyvalue_creation() {
        let pos = Position::new(1, 1, 0);
        let kv = KeyValue::new(
            "key".to_string(),
            Operator::Equals,
            Value::String("value".to_string()),
            pos,
        );
        assert_eq!(kv.key, "key");
        assert_eq!(kv.operator, Operator::Equals);
        assert_eq!(kv.position, pos);
    }

    #[test]
    fn test_value_is_clause() {
        let clause = Value::Clause(vec![]);
        let string = Value::String("test".to_string());
        assert!(clause.is_clause());
        assert!(!string.is_clause());
    }

    #[test]
    fn test_value_as_string() {
        let string = Value::String("test".to_string());
        let quoted = Value::QuotedString("quoted".to_string());
        let integer = Value::Integer(42);

        assert_eq!(string.as_string(), Some("test"));
        assert_eq!(quoted.as_string(), Some("quoted"));
        assert_eq!(integer.as_string(), None);
    }

    #[test]
    fn test_value_as_integer() {
        let integer = Value::Integer(42);
        let string = Value::String("test".to_string());

        assert_eq!(integer.as_integer(), Some(42));
        assert_eq!(string.as_integer(), None);
    }

    #[test]
    fn test_value_as_float() {
        let float = Value::Float(3.14);
        let string = Value::String("test".to_string());

        assert_eq!(float.as_float(), Some(3.14));
        assert_eq!(string.as_float(), None);
    }

    #[test]
    fn test_value_as_boolean() {
        let boolean = Value::Boolean(true);
        let string = Value::String("test".to_string());

        assert_eq!(boolean.as_boolean(), Some(true));
        assert_eq!(string.as_boolean(), None);
    }

    #[test]
    fn test_value_as_clause() {
        let clause = Value::Clause(vec![]);
        let string = Value::String("test".to_string());

        assert!(clause.as_clause().is_some());
        assert_eq!(clause.as_clause().unwrap().len(), 0);
        assert!(string.as_clause().is_none());
    }

    #[test]
    fn test_ast_traverse() {
        let mut ast = AST::new("test.txt".to_string());
        let pos = Position::new(1, 1, 0);

        let kv = KeyValue::new(
            "key".to_string(),
            Operator::Equals,
            Value::String("value".to_string()),
            pos,
        );
        ast.add_statement(Statement::KeyValue(kv));
        ast.add_statement(Statement::Comment("comment".to_string(), pos));

        let mut count = 0;
        ast.traverse(&mut |_| {
            count += 1;
        });

        assert_eq!(count, 2);
    }

    #[test]
    fn test_ast_traverse_nested() {
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

        let mut count = 0;
        ast.traverse(&mut |_| {
            count += 1;
        });

        assert_eq!(count, 2);
    }

    #[test]
    fn test_ast_find_at_position() {
        let mut ast = AST::new("test.txt".to_string());
        let pos1 = Position::new(1, 1, 0);
        let pos2 = Position::new(2, 1, 10);

        let kv1 = KeyValue::new(
            "key1".to_string(),
            Operator::Equals,
            Value::String("value1".to_string()),
            pos1,
        );
        let kv2 = KeyValue::new(
            "key2".to_string(),
            Operator::Equals,
            Value::String("value2".to_string()),
            pos2,
        );

        ast.add_statement(Statement::KeyValue(kv1));
        ast.add_statement(Statement::KeyValue(kv2));

        let found = ast.find_at_position(Position::new(1, 5, 5));
        assert!(found.is_some());
    }

    #[test]
    fn test_statement_types() {
        let pos = Position::new(1, 1, 0);

        let kv = Statement::KeyValue(KeyValue::new(
            "key".to_string(),
            Operator::Equals,
            Value::String("value".to_string()),
            pos,
        ));

        let value_only = Statement::ValueOnly(Value::Integer(42), pos);
        let comment = Statement::Comment("test".to_string(), pos);

        match kv {
            Statement::KeyValue(_) => {}
            _ => panic!("Expected KeyValue"),
        }

        match value_only {
            Statement::ValueOnly(_, _) => {}
            _ => panic!("Expected ValueOnly"),
        }

        match comment {
            Statement::Comment(_, _) => {}
            _ => panic!("Expected Comment"),
        }
    }
}
