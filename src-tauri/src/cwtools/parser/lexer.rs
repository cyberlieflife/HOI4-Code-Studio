//! 词法分析器模块
//!
//! 负责将 Paradox 脚本文本转换为 Token 流

use crate::cwtools::models::{Operator, Position, Token};
use std::fmt;

/// 词法分析错误
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    /// 错误消息
    pub message: String,
    /// 错误位置
    pub position: Position,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lexer error at line {}, column {}: {}",
            self.position.line, self.position.column, self.message
        )
    }
}

impl std::error::Error for LexError {}

/// 词法分析器
///
/// 将输入文本转换为 Token 流，支持 Paradox 脚本的所有语法元素
pub struct Lexer<'a> {
    /// 输入文本
    input: &'a str,
    /// 输入字节数组（用于高效访问）
    bytes: &'a [u8],
    /// 当前字节位置
    position: usize,
    /// 当前行号（从 1 开始）
    line: usize,
    /// 当前列号（从 1 开始）
    column: usize,
    /// 是否已处理 UTF-8 BOM
    bom_processed: bool,
    /// 预读的 token（用于 peek）
    peeked: Option<Result<Token, LexError>>,
}

impl<'a> Lexer<'a> {
    /// 创建新的词法分析器
    ///
    /// # 参数
    /// * `input` - 输入文本
    ///
    /// # 返回
    /// 新的 Lexer 实例
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Self {
            input,
            bytes: input.as_bytes(),
            position: 0,
            line: 1,
            column: 1,
            bom_processed: false,
            peeked: None,
        };

        // 处理 UTF-8 BOM
        lexer.skip_bom();

        lexer
    }

    /// 跳过 UTF-8 BOM（如果存在）
    ///
    /// UTF-8 BOM 是字节序列 0xEF 0xBB 0xBF
    fn skip_bom(&mut self) {
        if self.bom_processed {
            return;
        }

        if self.bytes.len() >= 3
            && self.bytes[0] == 0xEF
            && self.bytes[1] == 0xBB
            && self.bytes[2] == 0xBF
        {
            self.position = 3;
        }

        self.bom_processed = true;
    }

    /// 获取下一个 Token
    ///
    /// # 返回
    /// * `Ok(Token)` - 成功解析的 Token
    /// * `Err(LexError)` - 词法分析错误
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        // 如果有预读的 token，返回它
        if let Some(peeked) = self.peeked.take() {
            return peeked;
        }

        self.skip_whitespace_except_newline();

        if self.is_at_end() {
            return Ok(Token::Eof);
        }

        let current_char = self.current_char();

        // 处理换行符
        if current_char == '\n' {
            self.advance();
            return Ok(Token::Newline);
        }

        // 处理注释
        if current_char == '#' {
            return self.read_comment();
        }

        // 处理花括号
        if current_char == '{' {
            self.advance();
            return Ok(Token::LeftBrace);
        }

        if current_char == '}' {
            self.advance();
            return Ok(Token::RightBrace);
        }

        // 处理带引号的字符串
        if current_char == '"' {
            return self.read_quoted_string();
        }

        // 处理操作符
        if let Some(op) = self.try_read_operator() {
            return Ok(Token::Operator(op));
        }

        // 处理数值（负号开头或数字开头）
        if current_char == '-' || current_char.is_ascii_digit() {
            if let Some(token) = self.try_read_number() {
                return Ok(token);
            }
        }

        // 处理标识符和不带引号的字符串
        if self.is_identifier_start(current_char) {
            return self.read_identifier_or_string();
        }

        // 未知字符
        Err(LexError {
            message: format!("Unexpected character: '{}'", current_char),
            position: self.current_position(),
        })
    }

    /// 预读下一个 Token（不消费）
    ///
    /// # 返回
    /// * `Ok(Token)` - 下一个 Token
    /// * `Err(LexError)` - 词法分析错误
    pub fn peek_token(&mut self) -> Result<Token, LexError> {
        if self.peeked.is_none() {
            self.peeked = Some(self.next_token());
        }

        // 确保 peeked 不是 None
        self.peeked.as_ref().cloned().expect("peeked should be Some: 内部不变量违反，peek_token 在 peeked 为 None 时被调用")
    }

    /// 跳过空白字符（除了换行符）
    fn skip_whitespace_except_newline(&mut self) {
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// 读取注释
    ///
    /// 注释以 # 开头，到行尾结束
    fn read_comment(&mut self) -> Result<Token, LexError> {
        let start = self.position;

        // 跳过 #
        self.advance();

        // 读取到行尾
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }

        let comment_text = &self.input[start..self.position];
        Ok(Token::Comment(comment_text.to_string()))
    }

    /// 读取带引号的字符串
    fn read_quoted_string(&mut self) -> Result<Token, LexError> {
        let start_pos = self.current_position();

        // 跳过开始的引号
        self.advance();

        let mut result = String::new();

        while !self.is_at_end() {
            let ch = self.current_char();

            if ch == '"' {
                // 结束引号
                self.advance();
                return Ok(Token::QuotedString(result));
            }

            if ch == '\\' {
                // 转义字符
                self.advance();
                if !self.is_at_end() {
                    let escaped = self.current_char();
                    result.push(match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '"' => '"',
                        '\\' => '\\',
                        _ => escaped,
                    });
                    self.advance();
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }

        // 未闭合的字符串
        Err(LexError {
            message: "Unterminated quoted string".to_string(),
            position: start_pos,
        })
    }

    /// 尝试读取操作符
    fn try_read_operator(&mut self) -> Option<Operator> {
        let ch = self.current_char();

        // 尝试双字符操作符
        if self.position + 1 < self.bytes.len() {
            let next_ch = self.bytes[self.position + 1] as char;
            let two_char = format!("{}{}", ch, next_ch);

            if let Some(op) = Operator::from_str(&two_char) {
                self.advance();
                self.advance();
                return Some(op);
            }
        }

        // 尝试单字符操作符
        let one_char = ch.to_string();
        if let Some(op) = Operator::from_str(&one_char) {
            self.advance();
            return Some(op);
        }

        None
    }

    /// 尝试读取数值
    ///
    /// 支持整数、浮点数和百分比
    fn try_read_number(&mut self) -> Option<Token> {
        let start = self.position;

        // 处理负号
        if self.current_char() == '-' {
            // 负号后必须紧跟数字（不能有空格）
            if self.position + 1 >= self.bytes.len()
                || !(self.bytes[self.position + 1] as char).is_ascii_digit()
            {
                // 回退，这不是数字
                return None;
            }
            self.advance();
        }

        // 读取整数部分
        while !self.is_at_end() && self.current_char().is_ascii_digit() {
            self.advance();
        }

        let mut is_float = false;

        // 检查小数点
        if !self.is_at_end() && self.current_char() == '.' {
            // 预读下一个字符，确保是数字
            if self.position + 1 < self.bytes.len()
                && (self.bytes[self.position + 1] as char).is_ascii_digit()
            {
                is_float = true;
                self.advance(); // 跳过小数点

                // 读取小数部分
                while !self.is_at_end() && self.current_char().is_ascii_digit() {
                    self.advance();
                }
            }
        }

        let number_str = &self.input[start..self.position];

        // 解析数值
        if is_float {
            if let Ok(value) = number_str.parse::<f64>() {
                Some(Token::Float(value))
            } else {
                None
            }
        } else if let Ok(value) = number_str.parse::<i64>() {
            Some(Token::Integer(value))
        } else {
            None
        }
    }

    /// 读取标识符或不带引号的字符串
    ///
    /// 标识符可以包含字母、数字、下划线、冒号等
    fn read_identifier_or_string(&mut self) -> Result<Token, LexError> {
        let start = self.position;

        while !self.is_at_end() && self.is_identifier_char(self.current_char()) {
            self.advance();
        }

        let text = &self.input[start..self.position];

        // 检查是否为布尔值
        match text {
            "yes" => Ok(Token::Boolean(true)),
            "no" => Ok(Token::Boolean(false)),
            _ => {
                // 检查是否为纯标识符（只包含字母、数字、下划线）
                if text.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    Ok(Token::Identifier(text.to_string()))
                } else {
                    Ok(Token::String(text.to_string()))
                }
            }
        }
    }

    /// 判断字符是否可以作为标识符的开始
    fn is_identifier_start(&self, ch: char) -> bool {
        ch.is_alphabetic() || ch == '_' || ch == '@' || ch == '$'
    }

    /// 判断字符是否可以作为标识符的一部分
    fn is_identifier_char(&self, ch: char) -> bool {
        ch.is_alphanumeric()
            || ch == '_'
            || ch == ':'
            || ch == '.'
            || ch == '@'
            || ch == '$'
            || ch == '-'
            || ch == '\''
    }

    /// 获取当前字符
    fn current_char(&self) -> char {
        self.bytes[self.position] as char
    }

    /// 前进一个字符
    fn advance(&mut self) {
        if self.is_at_end() {
            return;
        }

        let ch = self.current_char();

        self.position += 1;

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }

    /// 判断是否到达输入末尾
    fn is_at_end(&self) -> bool {
        self.position >= self.bytes.len()
    }

    /// 获取当前位置
    fn current_position(&self) -> Position {
        Position::new(self.line, self.column, self.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_empty_input() {
        let mut lexer = Lexer::new("");
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_whitespace() {
        let mut lexer = Lexer::new("   \t  ");
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_newline() {
        let mut lexer = Lexer::new("\n");
        assert_eq!(lexer.next_token().unwrap(), Token::Newline);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_comment() {
        let mut lexer = Lexer::new("# this is a comment");
        match lexer.next_token().unwrap() {
            Token::Comment(text) => assert_eq!(text, "# this is a comment"),
            _ => panic!("Expected comment token"),
        }
    }

    #[test]
    fn test_lexer_braces() {
        let mut lexer = Lexer::new("{ }");
        assert_eq!(lexer.next_token().unwrap(), Token::LeftBrace);
        assert_eq!(lexer.next_token().unwrap(), Token::RightBrace);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_operators() {
        let input = "= > < >= <= != == ?=";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Operator(Operator::Equals)
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Operator(Operator::GreaterThan)
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Operator(Operator::LessThan)
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Operator(Operator::GreaterEqual)
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Operator(Operator::LessEqual)
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Operator(Operator::NotEqual)
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Operator(Operator::EqualEqual)
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Operator(Operator::QuestionEqual)
        );
    }

    #[test]
    fn test_lexer_integers() {
        let mut lexer = Lexer::new("42 -10 0");

        assert_eq!(lexer.next_token().unwrap(), Token::Integer(42));
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(-10));
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(0));
    }

    #[test]
    fn test_lexer_floats() {
        let mut lexer = Lexer::new("3.14 -2.5 0.0");

        assert_eq!(lexer.next_token().unwrap(), Token::Float(3.14));
        assert_eq!(lexer.next_token().unwrap(), Token::Float(-2.5));
        assert_eq!(lexer.next_token().unwrap(), Token::Float(0.0));
    }

    #[test]
    fn test_lexer_booleans() {
        let mut lexer = Lexer::new("yes no");

        assert_eq!(lexer.next_token().unwrap(), Token::Boolean(true));
        assert_eq!(lexer.next_token().unwrap(), Token::Boolean(false));
    }

    #[test]
    fn test_lexer_identifiers() {
        let mut lexer = Lexer::new("test_id my_var _private");

        match lexer.next_token().unwrap() {
            Token::Identifier(s) => assert_eq!(s, "test_id"),
            _ => panic!("Expected identifier"),
        }
        match lexer.next_token().unwrap() {
            Token::Identifier(s) => assert_eq!(s, "my_var"),
            _ => panic!("Expected identifier"),
        }
        match lexer.next_token().unwrap() {
            Token::Identifier(s) => assert_eq!(s, "_private"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_lexer_quoted_strings() {
        let mut lexer = Lexer::new(r#""hello world" "test""#);

        match lexer.next_token().unwrap() {
            Token::QuotedString(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected quoted string"),
        }
        match lexer.next_token().unwrap() {
            Token::QuotedString(s) => assert_eq!(s, "test"),
            _ => panic!("Expected quoted string"),
        }
    }

    #[test]
    fn test_lexer_quoted_string_with_escapes() {
        let mut lexer = Lexer::new(r#""hello\nworld" "test\"quote""#);

        match lexer.next_token().unwrap() {
            Token::QuotedString(s) => assert_eq!(s, "hello\nworld"),
            _ => panic!("Expected quoted string"),
        }
        match lexer.next_token().unwrap() {
            Token::QuotedString(s) => assert_eq!(s, "test\"quote"),
            _ => panic!("Expected quoted string"),
        }
    }

    #[test]
    fn test_lexer_unquoted_strings() {
        let mut lexer = Lexer::new("some-value test.txt");

        match lexer.next_token().unwrap() {
            Token::String(s) => assert_eq!(s, "some-value"),
            _ => panic!("Expected string"),
        }
        match lexer.next_token().unwrap() {
            Token::String(s) => assert_eq!(s, "test.txt"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_lexer_peek_token() {
        let mut lexer = Lexer::new("test 42");

        // Peek 不消费 token
        match lexer.peek_token().unwrap() {
            Token::Identifier(s) => assert_eq!(s, "test"),
            _ => panic!("Expected identifier"),
        }

        // 再次 peek 应该返回相同的 token
        match lexer.peek_token().unwrap() {
            Token::Identifier(s) => assert_eq!(s, "test"),
            _ => panic!("Expected identifier"),
        }

        // next_token 消费 token
        match lexer.next_token().unwrap() {
            Token::Identifier(s) => assert_eq!(s, "test"),
            _ => panic!("Expected identifier"),
        }

        // 现在 peek 应该返回下一个 token
        assert_eq!(lexer.peek_token().unwrap(), Token::Integer(42));
    }

    #[test]
    fn test_lexer_utf8_bom() {
        // UTF-8 BOM: 0xEF 0xBB 0xBF
        let input_with_bom = "\u{FEFF}test";
        let mut lexer = Lexer::new(input_with_bom);

        match lexer.next_token().unwrap() {
            Token::Identifier(s) => assert_eq!(s, "test"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_lexer_complex_script() {
        let input = r#"
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
        let mut lexer = Lexer::new(input);

        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token().unwrap();
            if token == Token::Eof {
                break;
            }
            tokens.push(token);
        }

        // 验证至少解析出了一些 token
        assert!(tokens.len() > 10);
    }

    #[test]
    fn test_lexer_unterminated_string() {
        let mut lexer = Lexer::new(r#""unterminated"#);

        match lexer.next_token() {
            Err(LexError { message, .. }) => {
                assert!(message.contains("Unterminated"));
            }
            _ => panic!("Expected error for unterminated string"),
        }
    }

    #[test]
    fn test_lexer_position_tracking() {
        let input = "test\n42";
        let mut lexer = Lexer::new(input);

        lexer.next_token().unwrap(); // test
        lexer.next_token().unwrap(); // newline

        let pos = lexer.current_position();
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);
    }

    #[test]
    fn test_lexer_negative_number_vs_minus() {
        // 测试负号紧跟数字的情况
        let mut lexer = Lexer::new("-42");
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(-42));

        // 测试负号后有空格的情况（在 Paradox 脚本中，- 不是操作符）
        // 这种情况下，- 会被当作标识符/字符串的一部分或导致错误
        let mut lexer2 = Lexer::new("value = -10");
        match lexer2.next_token().unwrap() {
            Token::Identifier(s) => assert_eq!(s, "value"),
            _ => panic!("Expected identifier"),
        }
        assert_eq!(
            lexer2.next_token().unwrap(),
            Token::Operator(Operator::Equals)
        );
        assert_eq!(lexer2.next_token().unwrap(), Token::Integer(-10));
    }

    #[test]
    fn test_lexer_special_identifiers() {
        let mut lexer = Lexer::new("@variable $scope");

        match lexer.next_token().unwrap() {
            Token::String(s) => assert_eq!(s, "@variable"),
            _ => panic!("Expected string"),
        }
        match lexer.next_token().unwrap() {
            Token::String(s) => assert_eq!(s, "$scope"),
            _ => panic!("Expected string"),
        }
    }
}
