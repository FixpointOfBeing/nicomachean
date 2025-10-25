use std::{
    char::REPLACEMENT_CHARACTER,
    fmt::{self},
};
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    BoolLiteral(bool),
    NumberLiteral(String),
    BoolType,         // Bool
    NumberType,       // NumberType
    If,               // if
    Then,             // then
    Else,             // else
    Def,              // def
    Colon,            // :
    RightArrow,       // ->
    Equals,           // =
    Lambda,           // λ | \
    LeftParen,        // (
    RightParen,       // )
    RightArrowDouble, // =>
    EOF,
}
impl Token {
    pub fn size(&self) -> usize {
        match self {
            Token::Def => 3,
            Token::Identifier(s) => s.len(),
            Token::NumberLiteral(s) => s.len(),
            Token::BoolLiteral(true) => 4,
            Token::BoolLiteral(false) => 5,
            Token::BoolType => 4,
            Token::NumberType => 6,
            Token::If => 2,
            Token::Then => 4,
            Token::Else => 4,
            Token::Colon => 1,
            Token::RightArrow => 2,
            Token::Equals => 1,
            Token::Lambda => 1,
            Token::LeftParen => 1,
            Token::RightParen => 1,
            Token::RightArrowDouble => 2,
            Token::EOF => 0,
        }
    }
    pub fn is_identifier(&self) -> bool {
        matches!(self, Token::Identifier(_))
    }
}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Def => write!(f, "def"),
            Self::NumberLiteral(n) => write!(f, "{}", n),
            Self::Identifier(str) => write!(f, "{}", str),
            Self::BoolLiteral(true) => write!(f, "True"),
            Self::BoolLiteral(false) => write!(f, "False"),
            Self::BoolType => write!(f, "Bool"),
            Self::NumberType => write!(f, "Number"),
            Self::If => write!(f, "if"),
            Self::Then => write!(f, "then"),
            Self::Else => write!(f, "else"),
            Self::Colon => write!(f, ":"),
            Self::RightArrow => write!(f, "->"),
            Self::Equals => write!(f, "="),
            Self::Lambda => write!(f, "λ"),
            Self::LeftParen => write!(f, "("),
            Self::RightParen => write!(f, ")"),
            Self::RightArrowDouble => write!(f, "=>"),
            Self::EOF => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoordToken {
    pub token: Token,
    pub row: usize, // starts with 1
    pub col: usize, // starts with 1
    pub size: usize,
}

impl fmt::Display for CoordToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}, {})", self.token, self.row, self.col)
    }
}

impl CoordToken {
    pub fn is_token(&self, token: Token) -> bool {
        self.token == token
    }
}

#[derive(Debug, PartialEq)]
pub enum LexerError {
    UnknownChar { found: char },
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerError::UnknownChar { found } => {
                let display_char = if *found == REPLACEMENT_CHARACTER {
                    "\\u{FFFD}".to_string()
                } else {
                    found.to_string()
                };
                write!(f, "Unknown character: '{}'", display_char)
            }
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    row: usize, // starts with 1
    col: usize, // starts with 1
}

impl Lexer {
    const RESERVED_WORDS: &'static [&'static str] = &[
        "Def", "True", "False", "Bool", "Number", "if", "then", "else",
    ];
    // todo : buffer
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        Self {
            input: chars,
            position: 0,
            row: 1,
            col: 1,
        }
    }

    fn wrap_token(&self, token: Token) -> CoordToken {
        CoordToken {
            token: token.clone(),
            row: self.row,
            col: self.col,
            size: token.size(),
        }
    }
    fn wrap_token_row_col(&self, token: Token, row: usize, col: usize) -> CoordToken {
        CoordToken {
            token: token.clone(),
            row: row,
            col: col,
            size: token.size(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) {
        // \n
        match self.peek() {
            Some(peek) => {
                if peek == '\n' {
                    self.position += 1;
                    self.newline();
                } else {
                    self.position += 1;
                    self.col += 1;
                }
            }
            None => {}
        }
    }
    fn newline(&mut self) {
        self.row += 1;
        self.col = 1;
    }

    fn peek_many(&self, n: usize) -> Option<String> {
        let end_pos = self.position + n;
        if end_pos > self.input.len() {
            return None;
        }
        Some(self.input[self.position..end_pos].iter().collect())
    }

    fn advance_many(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    fn skip_whitespaces(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        if self.peek() != Some('-') {
            return;
        }
        let next_pos = self.position + 1;
        if self.input.get(next_pos) == Some(&'-') {
            self.advance_many(2);
            while let Some(ch) = self.peek() {
                if ch == '\n' {
                    break;
                }
                self.advance();
            }
        }
    }
    fn skip_block_comment(&mut self) {
        if self.peek_many(2) != Some("/*".to_string()) {
            return;
        }
        self.advance_many(2);
        let mut depth = 0;
        loop {
            if self.peek() == None {
                return;
            } else if self.peek_many(2) == Some("/*".to_string()) {
                depth += 1;
                self.advance_many(2);
            } else if self.peek_many(2) == Some("*/".to_string()) {
                if depth == 0 {
                    self.advance_many(2);
                    return;
                } else {
                    self.advance_many(2);
                    depth -= 1;
                }
            } else {
                self.advance();
            }
        }
    }

    fn is_identifier_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_identifier_rest(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn lex_identifier(&mut self) -> Option<CoordToken> {
        let fst = self.peek()?;
        if !Self::is_identifier_start(fst) {
            return None;
        }
        let row = self.row;
        let col = self.col;
        let mut identifier = String::new();
        identifier.push(fst);
        self.advance();
        loop {
            match self.peek() {
                Some(c) => {
                    if Self::is_identifier_rest(c) {
                        identifier.push(c);

                        self.advance();
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        let token = match identifier.as_str() {
            "def" => Token::Def,
            "True" => Token::BoolLiteral(true),
            "False" => Token::BoolLiteral(false),
            "Bool" => Token::BoolType,
            "Number" => Token::NumberType,
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            _ => {
                assert!(!Self::RESERVED_WORDS.contains(&identifier.as_str()));
                Token::Identifier(identifier)
            }
        };
        let size = *&token.size();
        Some(CoordToken {
            token,
            row,
            col,
            size,
        })
    }

    fn lex_char(&mut self) -> Option<CoordToken> {
        match self.peek() {
            Some('=') => {
                let token = self.wrap_token(Token::Equals);
                self.advance();
                Some(token)
            }
            Some(':') => {
                let token = self.wrap_token(Token::Colon);
                self.advance();
                Some(token)
            }
            Some('\\') | Some('λ') => {
                let token = self.wrap_token(Token::Lambda);
                self.advance();
                Some(token)
            }
            Some('(') => {
                let token = self.wrap_token(Token::LeftParen);
                self.advance();
                Some(token)
            }
            Some(')') => {
                let token = self.wrap_token(Token::RightParen);
                self.advance();
                Some(token)
            }
            _ => None,
        }
    }

    fn lex_number(&mut self) -> Option<CoordToken> {
        let row = self.row;
        let col = self.col;
        let mut num_str = String::new();
        let mut decimal_point_seen = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else if c == '.' && !decimal_point_seen {
                decimal_point_seen = true;
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        let token = self.wrap_token_row_col(Token::NumberLiteral(num_str), row, col);
        Some(token)
    }

    fn lex_two_chars(&mut self) -> Option<CoordToken> {
        match self.peek_many(2) {
            Some(s) => match s.as_str() {
                "->" => {
                    let token = self.wrap_token(Token::RightArrow);
                    self.advance_many(2);
                    Some(token)
                }
                "=>" => {
                    let token = self.wrap_token(Token::RightArrowDouble);
                    self.advance_many(2);
                    Some(token)
                }
                _ => None,
            },
            None => None,
        }
    }

    fn lex_eof(&mut self) -> Option<CoordToken> {
        match self.peek() {
            Some(_) => None,
            None => Some(self.wrap_token(Token::EOF)),
        }
    }
    pub fn next_token(&mut self) -> Result<CoordToken, LexerError> {
        loop {
            self.skip_whitespaces();
            match self.peek_many(2) {
                Some(s) => match s.as_str() {
                    "--" => {
                        self.skip_line_comment();
                    }
                    "/*" => {
                        self.skip_block_comment();
                    }
                    _ => break,
                },
                None => break,
            }
        }

        // the order maters
        if let Some(token) = self.lex_eof() {
            return Ok(token);
        }
        if let Some(token) = self.lex_identifier() {
            return Ok(token);
        }
        if let Some(token) = self.lex_number() {
            return Ok(token);
        }
        if let Some(token) = self.lex_two_chars() {
            return Ok(token);
        }
        if let Some(token) = self.lex_char() {
            return Ok(token);
        }

        match self.peek() {
            Some(c) => Err(LexerError::UnknownChar { found: c }),
            None => Ok(self.wrap_token(Token::EOF)),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<CoordToken>, LexerError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = token.is_token(Token::EOF);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use std::iter::zip;

    use super::*;

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_whitespace_only() {
        let mut lexer = Lexer::new("   \t\n  ");
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_single_tokens() {
        let test_cases = vec![
            (":", Token::Colon),
            ("=", Token::Equals),
            ("λ", Token::Lambda),
            ("(", Token::LeftParen),
            (")", Token::RightParen),
            ("->", Token::RightArrow),
        ];

        for (input, expected) in test_cases {
            let mut lexer = Lexer::new(input);
            assert_eq!(
                lexer.next_token().unwrap().is_token(expected),
                true,
                "Failed for input: {}",
                input
            );
            assert!(lexer.next_token().unwrap().is_token(Token::EOF));
        }
    }

    #[test]
    fn test_identifiers() {
        let test_cases = vec![
            ("x", "x"),
            ("hello", "hello"),
            ("_var", "_var"),
            ("var123", "var123"),
            ("_123abc", "_123abc"),
            ("CamelCase", "CamelCase"),
        ];

        for (input, expected) in test_cases {
            let mut lexer = Lexer::new(input);
            match lexer.next_token().unwrap() {
                CoordToken {
                    token: Token::Identifier(name),
                    ..
                } => assert_eq!(name, expected),
                other => panic!("Expected identifier, got {:?}", other),
            }
            assert!(lexer.next_token().unwrap().is_token(Token::EOF));
        }
    }

    #[test]
    fn test_boolean_literals() {
        let mut lexer = Lexer::new("True");
        assert!(
            lexer
                .next_token()
                .unwrap()
                .is_token(Token::BoolLiteral(true))
        );
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));

        let mut lexer = Lexer::new("False");
        assert!(
            lexer
                .next_token()
                .unwrap()
                .is_token(Token::BoolLiteral(false))
        );
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }
    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("-- this is a comment\nx");
        assert!(
            lexer
                .next_token()
                .unwrap()
                .is_token(Token::Identifier("x".to_string()))
        );
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_comment_at_end() {
        let mut lexer = Lexer::new("x -- comment at end");
        assert!(
            lexer
                .next_token()
                .unwrap()
                .is_token(Token::Identifier("x".to_string()))
        );
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_multiple_comments() {
        let mut lexer = Lexer::new("-- comment 1\n-- comment 2\nx");
        assert_eq!(
            lexer
                .next_token()
                .unwrap()
                .is_token(Token::Identifier("x".to_string())),
            true
        );
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_lambda_expression() {
        let mut lexer = Lexer::new("λx -> x");
        let expected = vec![
            Token::Lambda,
            Token::Identifier("x".to_string()),
            Token::RightArrow,
            Token::Identifier("x".to_string()),
            Token::EOF,
        ];

        for expected_token in expected {
            assert!(lexer.next_token().unwrap().is_token(expected_token));
        }
    }

    #[test]
    fn test_function_definition() {
        let mut lexer = Lexer::new("f : Bool -> Bool\nf = λx -> x");
        let expected = vec![
            Token::Identifier("f".to_string()),
            Token::Colon,
            Token::BoolType,
            Token::RightArrow,
            Token::BoolType,
            Token::Identifier("f".to_string()),
            Token::Equals,
            Token::Lambda,
            Token::Identifier("x".to_string()),
            Token::RightArrow,
            Token::Identifier("x".to_string()),
            Token::EOF,
        ];

        for expected_token in expected {
            let next = lexer.next_token();
            println!("{:?}", next);
            assert!(next.unwrap().is_token(expected_token));
        }
    }

    #[test]
    fn test_complex() {
        let mut lexer = Lexer::new("(λx -> (λy -> x)) True False");
        let expected = vec![
            Token::LeftParen,
            Token::Lambda,
            Token::Identifier("x".to_string()),
            Token::RightArrow,
            Token::LeftParen,
            Token::Lambda,
            Token::Identifier("y".to_string()),
            Token::RightArrow,
            Token::Identifier("x".to_string()),
            Token::RightParen,
            Token::RightParen,
            Token::BoolLiteral(true),
            Token::BoolLiteral(false),
            Token::EOF,
        ];

        for expected_token in expected {
            assert!(lexer.next_token().unwrap().is_token(expected_token));
        }
    }

    #[test]
    fn test_tokenize_helper() {
        let mut lexer = Lexer::new("x : Bool");
        let tokens = lexer.tokenize().unwrap();
        let expecteds = vec![
            Token::Identifier("x".to_string()),
            Token::Colon,
            Token::BoolType,
            Token::EOF,
        ];
        for (token, expected) in zip(tokens, expecteds) {
            assert!(token.is_token(expected));
        }
    }

    #[test]
    fn test_whitespace_between_tokens() {
        let mut lexer = Lexer::new("  x   :   Bool  ->  Bool  ");
        let expected = vec![
            Token::Identifier("x".to_string()),
            Token::Colon,
            Token::BoolType,
            Token::RightArrow,
            Token::BoolType,
            Token::EOF,
        ];

        for expected_token in expected {
            assert!(lexer.next_token().unwrap().is_token(expected_token));
        }
    }

    #[test]
    fn test_mixed_whitespace() {
        let mut lexer = Lexer::new("x\t:\n\rBool");
        let expected = vec![
            Token::Identifier("x".to_string()),
            Token::Colon,
            Token::BoolType,
            Token::EOF,
        ];

        for expected_token in expected {
            assert!(lexer.next_token().unwrap().is_token(expected_token));
        }
    }

    #[test]
    fn test_identifier_with_numbers() {
        let mut lexer = Lexer::new("var123 _var456");
        assert_eq!(
            lexer
                .next_token()
                .unwrap()
                .is_token(Token::Identifier("var123".to_string())),
            true
        );
        assert_eq!(
            lexer
                .next_token()
                .unwrap()
                .is_token(Token::Identifier("_var456".to_string())),
            true
        );
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_underscore_identifier() {
        let mut lexer = Lexer::new("_");
        assert_eq!(
            lexer
                .next_token()
                .unwrap()
                .is_token(Token::Identifier("_".to_string())),
            true
        );
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_consecutive_tokens() {
        let mut lexer = Lexer::new("()λ->:");
        let expected = vec![
            Token::LeftParen,
            Token::RightParen,
            Token::Lambda,
            Token::RightArrow,
            Token::Colon,
            Token::EOF,
        ];

        for expected_token in expected {
            assert!(lexer.next_token().unwrap().is_token(expected_token));
        }
    }

    #[test]
    fn test_comment_without_newline() {
        let mut lexer = Lexer::new("x -- comment");
        assert_eq!(
            lexer
                .next_token()
                .unwrap()
                .is_token(Token::Identifier("x".to_string())),
            true
        );
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_unicode_lambda() {
        let mut lexer = Lexer::new("λ");
        assert!(lexer.next_token().unwrap().is_token(Token::Lambda));
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_real_world_example() {
        let input = r#"
        -- Identity function
        id : Bool -> Bool
        id = λx -> x

        -- Apply function
        apply : (Bool -> Bool) -> Bool -> Bool
        apply = λf -> λx -> f x
        "#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        // 验证一些关键 token
        assert!(
            tokens
                .clone()
                .into_iter()
                .map(|coord_token| coord_token.token)
                .collect::<Vec<Token>>()
                .contains(&Token::Identifier("id".to_string()))
        );
        assert!(
            tokens
                .clone()
                .into_iter()
                .map(|coord_token| coord_token.token)
                .collect::<Vec<Token>>()
                .contains(&Token::Identifier("apply".to_string()))
        );
        assert!(
            tokens
                .clone()
                .into_iter()
                .map(|coord_token| coord_token.token)
                .collect::<Vec<Token>>()
                .contains(&Token::Lambda)
        );
        assert!(
            tokens
                .clone()
                .into_iter()
                .map(|coord_token| coord_token.token)
                .collect::<Vec<Token>>()
                .contains(&Token::RightArrow)
        );
        assert!(
            tokens
                .clone()
                .into_iter()
                .map(|coord_token| coord_token.token)
                .collect::<Vec<Token>>()
                .contains(&Token::Colon)
        );
        assert!(
            tokens
                .clone()
                .into_iter()
                .map(|coord_token| coord_token.token)
                .collect::<Vec<Token>>()
                .contains(&Token::Equals)
        );
        assert!(tokens.last().unwrap().is_token(Token::EOF));
    }
    #[test]
    fn test_reserved_words_true() {
        let mut lexer = Lexer::new("True");
        match lexer.next_token().unwrap() {
            CoordToken {
                token: Token::BoolLiteral(true),
                ..
            } => {}
            other => panic!("Expected BooleanLiteral(true), got {:?}", other),
        }
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_reserved_words_false() {
        let mut lexer = Lexer::new("False");
        match lexer.next_token().unwrap() {
            CoordToken {
                token: Token::BoolLiteral(false),
                ..
            } => {}
            other => panic!("Expected BooleanLiteral(false), got {:?}", other),
        }
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_reserved_words_if() {
        let mut lexer = Lexer::new("if");
        match lexer.next_token().unwrap() {
            CoordToken {
                token: Token::If, ..
            } => {}
            other => panic!("Expected IF, got {:?}", other),
        }
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_reserved_words_then() {
        let mut lexer = Lexer::new("then");
        match lexer.next_token().unwrap() {
            CoordToken {
                token: Token::Then, ..
            } => {}
            other => panic!("Expected THEN, got {:?}", other),
        }
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_reserved_words_else() {
        let mut lexer = Lexer::new("else");
        match lexer.next_token().unwrap() {
            CoordToken {
                token: Token::Else, ..
            } => {}
            other => panic!("Expected ELSE, got {:?}", other),
        }
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_reserved_words_mixed() {
        let mut lexer = Lexer::new("if then else True False");
        let expected = vec![
            Token::If,
            Token::Then,
            Token::Else,
            Token::BoolLiteral(true),
            Token::BoolLiteral(false),
            Token::EOF,
        ];
        for expected_token in expected {
            assert!(lexer.next_token().unwrap().is_token(expected_token));
        }
    }

    #[test]
    fn test_block_comments() {
        let mut lexer = Lexer::new(
            "/* this is a comment /* this is a nested comment */ /* this is another nested comment */*/ x",
        );
        assert!(
            lexer
                .next_token()
                .unwrap()
                .is_token(Token::Identifier("x".to_string())),
        );
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_unterminated_block_comment() {
        let mut lexer = Lexer::new("/* this is an unterminated comment ");
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }
    #[test]
    fn test_lex_number() {
        let mut lexer = Lexer::new("  123 45.67 0.89");
        assert!(
            lexer
                .next_token()
                .unwrap()
                .is_token(Token::NumberLiteral("123".to_string()))
        );
        assert!(
            lexer
                .next_token()
                .unwrap()
                .is_token(Token::NumberLiteral("45.67".to_string()))
        );
        assert!(
            lexer
                .next_token()
                .unwrap()
                .is_token(Token::NumberLiteral("0.89".to_string()))
        );
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }

    #[test]
    fn test_number_type() {
        let mut lexer = Lexer::new("Number");
        match lexer.next_token().unwrap() {
            CoordToken {
                token: Token::NumberType,
                ..
            } => {}
            other => panic!("Expected NumberType, got {:?}", other),
        }
        assert!(lexer.next_token().unwrap().is_token(Token::EOF));
    }
}
