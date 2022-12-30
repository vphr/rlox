use crate::{scanner::*, error::RloxError};
#[derive(Debug, Clone)]
pub enum Expr {
    Binary(BinaryExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Unary(UnaryExpr),
}

impl Expr {
    pub fn accept<T>(&self, expr_visitor: &dyn ExprVisitor<T>) -> Result<T,RloxError> {
        match self {
            Expr::Binary(exp) => exp.accept(expr_visitor),
            Expr::Grouping(exp) => exp.accept(expr_visitor),
            Expr::Literal(exp) => exp.accept(expr_visitor),
            Expr::Unary(exp) => exp.accept(expr_visitor),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct GroupingExpr {
    pub expression: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct LiteralExpr {
    pub value: Option<Literal>,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: Token,
    pub right: Box<Expr>,
}

pub trait ExprVisitor<T> {
    fn visit_binary_expr(&self, binary: &BinaryExpr) -> Result<T,RloxError>;
    fn visit_grouping_expr(&self, grouping: &GroupingExpr) -> Result<T,RloxError>;
    fn visit_literal_expr(&self, literal: &LiteralExpr) -> Result<T,RloxError>;
    fn visit_unary_expr(&self, unary: &UnaryExpr) -> Result<T,RloxError>;
}

impl BinaryExpr {
    pub fn accept<T>(&self, visitor: &dyn ExprVisitor<T>) -> Result<T,RloxError> {
        visitor.visit_binary_expr(self)
    }
}

impl GroupingExpr {
    fn accept<T>(&self, visitor: &dyn ExprVisitor<T>) -> Result<T,RloxError> {
        visitor.visit_grouping_expr(self)
    }
}

impl LiteralExpr {
    fn accept<T>(&self, visitor: &dyn ExprVisitor<T>) -> Result<T,RloxError> {
        visitor.visit_literal_expr(self)
    }
}

impl UnaryExpr {
    fn accept<T>(&self, visitor: &dyn ExprVisitor<T>) -> Result<T,RloxError> {
        visitor.visit_unary_expr(self)
    }
}
