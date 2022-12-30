use crate::expr::*;
use crate::scanner::*;

pub struct Interpreter {}

impl ExprVisitor<Option<Literal>> for Interpreter {
    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Option<Literal> {
        let left = self.evaluate(*expr.left.clone())?;
        let right = self.evaluate(*expr.right.clone())?;
        match (left, right) {
            (Literal::Number(l), Literal::Number(r)) => match expr.operator.token_type {
                TokenType::Minus => Some(Literal::Number(l - r)),
                TokenType::Slash => Some(Literal::Number(l / r)),
                TokenType::Star => Some(Literal::Number(l * r)),
                TokenType::Plus => Some(Literal::Number(l + r)),
                TokenType::Greater => {
                    if l > r {
                        Some(Literal::True)
                    } else {
                        Some(Literal::False)
                    }
                }
                TokenType::GreaterEqual => {
                    if l >= r {
                        Some(Literal::True)
                    } else {
                        Some(Literal::False)
                    }
                }
                TokenType::Less => {
                    if l < r {
                        Some(Literal::True)
                    } else {
                        Some(Literal::False)
                    }
                }
                TokenType::LessEqual => {
                    if l <= r {
                        Some(Literal::True)
                    } else {
                        Some(Literal::False)
                    }
                }
                _ => None,
            },
            (Literal::Str(l), Literal::Str(r)) => match expr.operator.token_type {
                TokenType::Plus => Some(Literal::Str(l + &r)),
                _ => None,
            },
            (left, right) => match expr.operator.token_type {
                TokenType::BangEqual => self.is_equal(left, right),
                TokenType::EqualEqual => self.is_equal(left, right),
                _ => None,
            },
        }
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Option<Literal> {
        self.evaluate(*expr.expression.clone())
    }

    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Option<Literal> {
        expr.value.clone()
    }

    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Option<Literal> {
        let right = self.evaluate(*expr.right.clone())?;
        match expr.operator.token_type {
            TokenType::Minus => match right {
                Literal::Number(n) => Some(Literal::Number(-n)),
                _ => None,
            },
            TokenType::Bang => match !self.is_truthy(right) {
                true => Some(Literal::True),
                false => Some(Literal::False),
            },
            _ => None,
        }
    }
}

impl Interpreter {
    pub fn interpret(&self, expr: Expr) {
        if let Some(value) = self.evaluate(expr) {
            println!("{}",self.stringify(value))
        }
    }
    fn evaluate(&self, expr: Expr) -> Option<Literal> {
        expr.accept(self)
    }

    // anything except null and false is true
    fn is_truthy(&self, right: Literal) -> bool {
        match right {
            Literal::False | Literal::Nil => false,
            _ => true,
        }
    }
    fn check_equality<T: PartialEq>(&self, left: &T, right: &T) -> Option<Literal> {
        match left.eq(right) {
            true => Some(Literal::True),
            false => Some(Literal::False),
        }
    }

    fn is_equal(&self, left: Literal, right: Literal) -> Option<Literal> {
        match (left, right) {
            (Literal::Identifier(l), Literal::Identifier(r)) => self.check_equality(&l, &r),
            (Literal::Str(l), Literal::Str(r)) => self.check_equality(&l, &r),
            (Literal::Number(l), Literal::Number(r)) => self.check_equality(&l, &r),
            (Literal::True, Literal::True) => Some(Literal::True),
            (Literal::False, Literal::False) => Some(Literal::True),
            (Literal::Nil, Literal::Nil) => None,
            _ => None,
        }
    }

    fn stringify(&self, value: Literal) -> String {
        match value {
            Literal::Identifier(i) => i,
            Literal::Str(s) => s,
            Literal::Number(n) => n.to_string(),
            Literal::True => "true".to_string(),
            Literal::False => "false".to_string(),
            Literal::Nil => "nil".to_string(),
        }
    }
}
