use crate::expr;
use crate::expr::*;
use crate::scanner::*;

#[derive(Debug)]
pub struct Parser {
    pub tokens: Vec<Token>,
    pub current: usize,
}

impl Parser {
    pub fn parse(&mut self) -> Expr {
        self.expression()
    }
    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.match_token(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator: Token = self.previous();
            let right: Expr = self.comparison();
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }
        expr
    }

    fn match_token(&mut self, token_type: Vec<TokenType>) -> bool {
        for token in token_type {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        while self.match_token(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term();
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }
        expr
    }

    fn previous(&mut self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn advance(&mut self) -> Token {
        if !self.is_end() {
            self.current += 1;
        }
        self.previous()
    }
    fn is_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn check(&self, token: TokenType) -> bool {
        if self.is_end() {
            false;
        }
        self.peek().token_type == token
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        while self.match_token(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor();
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }
        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        while self.match_token(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary();
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }
        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_token(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary();
            Expr::Unary(UnaryExpr {
                operator,
                right: Box::new(right),
            });
        }
        self.primary().unwrap()
    }

    fn primary(&mut self) -> Result<expr::Expr, String> {
        if self.match_token(vec![TokenType::False]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: Some(Literal::Str("false".to_string())),
            }));
        }
        if self.match_token(vec![TokenType::True]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: Some(Literal::Str("true".to_string())),
            }));
        }
        if self.match_token(vec![TokenType::Nil]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: Some(Literal::Str("null".to_string())),
            }));
        }
        if self.match_token(vec![TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: self.previous().literal,
            }));
        }
        if self.match_token(vec![TokenType::LeftParen]) {
            let expr = self.expression();
            self.consume(TokenType::RightParen)?;
            return Ok(Expr::Grouping(GroupingExpr {
                expression: Box::new(expr),
            }));
        }
        if self.match_token(vec![TokenType::Identifier]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: self.previous().literal,
            }));
        }
        Err("Expected expression".to_string())
    }
    fn consume(&mut self, token: TokenType) -> Result<Token, String> {
        if self.check(token) {
            return Ok(self.advance());
        }
        Err("Expect ')' after expression.".to_string())
    }
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_end() {
            if self.previous().token_type == TokenType::Semicolon {
                break;
            }
            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => break,
                _ => {}
            }
            self.advance();
        }
    }
}
