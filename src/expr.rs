use crate::scanner::*;
#[derive(Debug)]
pub enum Expr {
    Binary(BinaryExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Unary(UnaryExpr),
}

impl Expr {
    pub fn accept<T>(&self, expr_visitor: &dyn ExprVisitor<T>) -> T {
        match self {
            Expr::Binary(exp) => exp.accept(expr_visitor),
            Expr::Grouping(exp) => exp.accept(expr_visitor),
            Expr::Literal(exp) => exp.accept(expr_visitor),
            Expr::Unary(exp) => exp.accept(expr_visitor),
        }
    }
}

#[derive(Debug)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug)]
pub struct GroupingExpr {
    pub expression: Box<Expr>,
}

#[derive(Debug)]
pub struct LiteralExpr {
    pub value: Option<Literal>,
}

#[derive(Debug)]
pub struct UnaryExpr {
    pub operator: Token,
    pub right: Box<Expr>,
}

pub trait ExprVisitor<T> {
    fn visit_binary_expr(&self, binary: &BinaryExpr) -> T;
    fn visit_grouping_expr(&self, grouping: &GroupingExpr) -> T;
    fn visit_literal_expr(&self, literal: &LiteralExpr) -> T;
    fn visit_unary_expr(&self, unary: &UnaryExpr) -> T;
}

impl BinaryExpr {
    pub fn accept<T>(&self, visitor: &dyn ExprVisitor<T>) -> T {
        visitor.visit_binary_expr(self)
    }
}

impl GroupingExpr {
    fn accept<T>(&self, visitor: &dyn ExprVisitor<T>) -> T {
        visitor.visit_grouping_expr(self)
    }
}

impl LiteralExpr {
    fn accept<T>(&self, visitor: &dyn ExprVisitor<T>) -> T {
        visitor.visit_literal_expr(self)
    }
}

impl UnaryExpr {
    fn accept<T>(&self, visitor: &dyn ExprVisitor<T>) -> T {
        visitor.visit_unary_expr(self)
    }
}