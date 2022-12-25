use std::collections::HashMap;

use crate::error::*;

#[derive(Debug, Clone)]
pub struct Scanner {
    source: Vec<u8>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<String, TokenType>,
}
impl Default for Scanner {
    fn default() -> Self {
        Scanner {
            source: vec![],
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            keywords: vec![
                ("and", TokenType::And),
                ("class", TokenType::Class),
                ("else", TokenType::Else),
                ("false", TokenType::False),
                ("for", TokenType::For),
                ("fun", TokenType::Fun),
                ("if", TokenType::If),
                ("nil", TokenType::Nil),
                ("or", TokenType::Or),
                ("print", TokenType::Print),
                ("return", TokenType::Return),
                ("super", TokenType::Super),
                ("this", TokenType::This),
                ("true", TokenType::True),
                ("var", TokenType::Var),
                ("while", TokenType::While),
            ]
            .into_iter()
            .map(|(k, v)| (String::from(k), v))
            .collect(),
        }
    }
}
impl Scanner {
    pub fn scan_tokens(&mut self, input: String) -> Result<Vec<Token>, RloxError> {
        self.source = input.into_bytes();
        while !self.is_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push({
            Token {
                token_type: TokenType::Eof,
                lexeme: vec![],
                literal: None,
                line: self.line,
            }
        });
        Ok(self.tokens.to_vec())
    }
    fn is_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<(), RloxError> {
        let token = self.advance();
        match token {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '!' => {
                if self.match_next_token('=') {
                    self.add_token(TokenType::BangEqual, None)
                } else {
                    self.add_token(TokenType::Bang, None)
                }
            }
            '=' => {
                if self.match_next_token('=') {
                    self.add_token(TokenType::EqualEqual, None)
                } else {
                    self.add_token(TokenType::Equal, None)
                }
            }
            '<' => {
                if self.match_next_token('=') {
                    self.add_token(TokenType::LessEqual, None)
                } else {
                    self.add_token(TokenType::Less, None)
                }
            }
            '>' => {
                if self.match_next_token('=') {
                    self.add_token(TokenType::GreaterEqual, None)
                } else {
                    self.add_token(TokenType::Greater, None)
                }
            }
            '/' if self.match_next_token('/') => {
                while self.peek() != '\n' && !self.is_end() {
                    self.advance();
                };
                Ok(())
            }
            '"' => {
                self.string();
                Ok(())
            }
            '0'..='9' => {
                self.number();
                Ok(())
            }
            '/' => self.add_token(TokenType::Slash, None),
            ' ' | '\r' | '\t' => Ok(()),
            '\n' => {
                self.line += 1;
                Ok(())
            }
            _ => {
                if token.is_alphabetic() {
                    self.identifier();
                    Ok(())
                } else {
                    Err(RloxError::ParseError { character: token, message: "unhandled token {}".to_string() })
                }
            }
        }
    }
    fn advance(&mut self) -> char {
        self.current += 1;
        char::from(self.source[self.current - 1])
    }

    fn add_token(&mut self, token: TokenType, literal: Option<Literal>) -> Result<(), RloxError> {
        let lexeme = self.source[self.start..self.current].to_owned();
        self.tokens.push(Token {
            token_type: token,
            lexeme,
            literal,
            line: self.line,
        });
        return Ok(())
    }
    fn match_next_token(&mut self, match_token: char) -> bool {
        if self.is_end() {
            false;
        }
        if self.source.get(self.current).unwrap().to_owned() != match_token as u8 {
            false;
        }
        self.current = self.current + 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_end() {
            return '\0';
        }
        char::from(self.source[self.current])
    }
    fn string(&mut self) {
        while self.peek() != '"' && !self.is_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_end() {
            unimplemented!("unhandled: unterminated string");
        }

        self.advance();
        let value = self.source[self.start + 1..self.current - 1].to_owned();
        let string_value = String::from_utf8(value).map(|v| Literal::Str(v)).unwrap();
        self.add_token(TokenType::String, Some(string_value));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let value = self.source[self.start..self.current].to_owned();
        let number_value: f64 = String::from_utf8(value).unwrap().parse().unwrap();
        self.add_token(TokenType::Number, Some(Literal::Number(number_value)));
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        char::from(self.source[self.current + 1])
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() {
            self.advance();
        }
        let value = self.source[self.start..self.current].to_owned();
        let string_value = String::from_utf8(value).unwrap();
        match self.keywords.get(&string_value) {
            Some(keyword) => self.add_token(keyword.to_owned(), None),
            None => self.add_token(
                TokenType::Identifier,
                Some(Literal::Identifier(string_value)),
            ),
        };
    }
}

// pub fn scan_tokens(input: String) -> Result<Vec<Token>, RloxError> {
//     let mut scanner = Scanner::default();
//     scanner.scan_tokens(input)
// }
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Identifier(String),
    Str(String),
    Number(f64),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Vec<u8>,
    pub literal: Option<Literal>,
    pub line: usize,
}
