use crate::error::*;
use crate::expr::*;
use crate::scanner::*;
use crate::stmt::*;

#[derive(Debug)]
pub struct Parser {
    pub tokens: Vec<Token>,
    pub current: usize,
}

impl Parser {
    pub fn parse(&mut self) -> Result<Vec<Stmt>, RloxError> {
        let mut statements: Vec<Stmt> = vec![];
        while !self.is_end() {
            statements.push(self.declaration()?)
        }
        Ok(statements)
    }
    fn expression(&mut self) -> Result<Expr, RloxError> {
        self.assignment()
    }

    fn equality(&mut self) -> Result<Expr, RloxError> {
        let mut expr = self.comparison()?;

        while self.match_token(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator: Token = self.previous();
            let right: Expr = self.comparison()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }
        Ok(expr)
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

    fn comparison(&mut self) -> Result<Expr, RloxError> {
        let mut expr = self.term()?;
        while self.match_token(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }
        Ok(expr)
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

    fn term(&mut self) -> Result<Expr, RloxError> {
        let mut expr = self.factor()?;
        while self.match_token(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, RloxError> {
        let mut expr = self.unary()?;
        while self.match_token(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, RloxError> {
        if self.match_token(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            Expr::Unary(UnaryExpr {
                operator,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, RloxError> {
        if self.match_token(vec![TokenType::False]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: Some(Literal::False),
            }));
        }
        if self.match_token(vec![TokenType::True]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: Some(Literal::True),
            }));
        }
        if self.match_token(vec![TokenType::Nil]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: Some(Literal::Nil),
            }));
        }
        if self.match_token(vec![TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: self.previous().literal,
            }));
        }
        if self.match_token(vec![TokenType::Identifier]) {
            return Ok(Expr::Variable(VariableExpr {
                name: self.previous(),
            }));
        }
        if self.match_token(vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(
                TokenType::RightParen,
                "Expect ')' after expression.".to_string(),
            )?;
            return Ok(Expr::Grouping(GroupingExpr {
                expression: Box::new(expr),
            }));
        }
        if self.match_token(vec![TokenType::Identifier]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: self.previous().literal,
            }));
        }
        Err(RloxError::ParseError {
            token: self.tokens[self.current].clone(),
            current: self.current,
            message: "failed to parse".to_string(),
        })
    }
    fn consume(&mut self, token: TokenType, message: String) -> Result<Token, RloxError> {
        if self.check(token) {
            return Ok(self.advance());
        }
        Err(RloxError::ParseError {
            token: self.tokens[self.current].clone(),
            current: self.current,
            message,
        })
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

    fn statement(&mut self) -> Result<Stmt, RloxError> {
        if self.match_token(vec![TokenType::If]) {
            return self.if_statement();
        }
        if self.match_token(vec![TokenType::Print]) {
            return self.print_statement();
        }
        if self.match_token(vec![TokenType::LeftBrace]) {
            return Ok(Stmt::Block(BlockStmt {
                statements: self.block()?,
            }));
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt, RloxError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.".to_string())?;
        return Ok(Stmt::Print(PrintStmt {
            expression: Box::new(value),
        }));
    }

    fn expression_statement(&mut self) -> Result<Stmt, RloxError> {
        let value = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after expression.".to_string(),
        )?;
        return Ok(Stmt::Expression(ExpressionStmt {
            expression: Box::new(value),
        }));
    }

    fn declaration(&mut self) -> Result<Stmt, RloxError> {
        let res = if self.match_token(vec![TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };
        if res.is_err() {
            self.synchronize();
        }
        res
    }

    fn var_declaration(&mut self) -> Result<Stmt, RloxError> {
        let name = self.consume(TokenType::Identifier, "expect variable name".to_string())?;
        let initializer = if self.match_token(vec![TokenType::Equal]) {
            let res = self.expression()?;
            Some(Box::new(res))
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after value.".to_string())?;

        Ok(Stmt::Var(VarStmt { name, initializer }))
    }

    fn assignment(&mut self) -> Result<Expr, RloxError> {
        let expr = self.or()?;

        if self.match_token(vec![TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(v) = expr {
                return Ok(Expr::Assign(AssignExpr {
                    name: v.name,
                    value: Box::new(value),
                }));
            };

            return Err(RloxError::ParseError {
                current: self.current,
                token: equals,
                message: "Invalid assignment target.".to_string(),
            });
        }
        Ok(expr)
    }

    fn block(&mut self) -> Result<Vec<Stmt>, RloxError> {
        let mut statements: Vec<Stmt> = vec![];

        while !self.check(TokenType::RightBrace) && !self.is_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.".to_string())?;

        Ok(statements)
    }

    fn if_statement(&mut self) -> Result<Stmt, RloxError> {
        self.consume(TokenType::LeftParen, "Expect '(' after block.".to_string())?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after block.".to_string())?;

        let then_branch = self.statement()?;
        let else_branch: Option<Box<Stmt>> = if self.match_token(vec![TokenType::Else]) {
            let inner_statement = self.statement()?;
            Some(Box::new(inner_statement))
        } else {
            None
        };
        Ok(Stmt::If(IfStmt {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        }))
    }

    fn or(&mut self) -> Result<Expr, RloxError> {
        let mut expr = self.and()?;
        while self.match_token(vec![TokenType::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical(LogicalExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, RloxError> {
        let mut expr = self.equality()?;
        while self.match_token(vec![TokenType::And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical(LogicalExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }
        Ok(expr)
    }
}
