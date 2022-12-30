use crate::error::RloxError;
use crate::expr::*;
use crate::scanner::*;

pub struct Interpreter {}
pub enum Value {
    Str(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl ExprVisitor<Value> for Interpreter {
    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<Value, RloxError> {
        let left = self.evaluate(*expr.left.clone())?;
        let right = self.evaluate(*expr.right.clone())?;
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => match expr.operator.token_type {
                TokenType::Minus => Ok(Value::Number(l - r)),
                TokenType::Slash => Ok(Value::Number(l / r)),
                TokenType::Star => Ok(Value::Number(l * r)),
                TokenType::Plus => Ok(Value::Number(l + r)),
                TokenType::Greater => Ok(Value::Bool(l.gt(&r))),
                TokenType::GreaterEqual => Ok(Value::Bool(l.ge(&r))),
                TokenType::Less => Ok(Value::Bool(l.lt(&r))),
                TokenType::LessEqual => Ok(Value::Bool(l.le(&r))),
                _ => Err(RloxError::InterpreterError),
            },
            (Value::Str(l), Value::Str(r)) => match expr.operator.token_type {
                TokenType::Plus => Ok(Value::Str(l + &r)),
                _ => Err(RloxError::InterpreterError),
            },
            (left, right) => match expr.operator.token_type {
                TokenType::BangEqual => self.is_equal(left, right),
                TokenType::EqualEqual => self.is_equal(left, right),
                _ => Err(RloxError::InterpreterError),
            },
        }
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<Value, RloxError> {
        self.evaluate(*expr.expression.clone())
    }

    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<Value, RloxError> {
        let expr = expr.value.clone().expect("Valid literal expression");
        Ok(match expr {
            Literal::Identifier(i) => Value::Str(i),
            Literal::Str(s) => Value::Str(s),
            Literal::Number(n) => Value::Number(n),
            Literal::True => Value::Bool(true),
            Literal::False => Value::Bool(false),
            Literal::Nil => Value::Nil,
        })
    }

    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<Value, RloxError> {
        let right = self.evaluate(*expr.right.clone())?;
        match expr.operator.token_type {
            TokenType::Minus => match right {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(RloxError::InterpreterError),
            },
            TokenType::Bang => Ok(Value::Bool(!self.is_truthy(right))),
            _ => Err(RloxError::InterpreterError),
        }
    }
}

impl Interpreter {
    pub fn interpret(&self, expr: Expr) {
        if let Ok(value) = self.evaluate(expr) {
            println!("{}", self.stringify(value))
        }
    }
    fn evaluate(&self, expr: Expr) -> Result<Value, RloxError> {
        expr.accept(self)
    }

    // anything except null and false is true
    fn is_truthy(&self, right: Value) -> bool {
        match right {
            Value::Bool(false) | Value::Nil => false,
            _ => true,
        }
    }

    fn is_equal(&self, left: Value, right: Value) -> Result<Value, RloxError> {
        match (left, right) {
            (Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l.eq(&r))),
            (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l.eq(&r))),
            (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l.eq(&r))),
            (Value::Nil, Value::Nil) => Ok(Value::Bool(true)),
            _ => Ok(Value::Bool(false)),
        }
    }

    fn stringify(&self, value: Value) -> String {
        match value {
            Value::Str(s) => s,
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
        }
    }
}
