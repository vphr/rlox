use crate::expr::*;

pub struct AstPrinter {}

impl ExprVisitor<String> for AstPrinter {
    fn visit_binary_expr(&self, expr: &BinaryExpr) -> String {
        self.parenthesize(
            String::from_utf8(expr.operator.lexeme.to_vec()).unwrap(),
            vec![expr.left.as_ref(), expr.right.as_ref()],
        )
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> String {
        self.parenthesize("grouping".to_string(), vec![&expr.expression])
    }

    fn visit_literal_expr(&self, expr: &LiteralExpr) -> String {
        if let Some(value) = &expr.value {
            match value {
                crate::scanner::Literal::Identifier(val) => val.to_string(),
                crate::scanner::Literal::Str(val) => val.to_string(),
                crate::scanner::Literal::Number(val) => val.to_string(),
            }
        } else {
            "nil".to_string()
        }
    }

    fn visit_unary_expr(&self, expr: &UnaryExpr) -> String {
        self.parenthesize(
            String::from_utf8(expr.operator.lexeme.to_vec()).unwrap(),
            vec![expr.right.as_ref()],
        )
    }
}
impl AstPrinter {
    pub fn print(&self, expr: &Expr) -> String {
        expr.accept(self)
    }

    fn parenthesize(&self, name: String, expressions: Vec<&Expr>) -> String {
        let mut output = String::new();
        output.push_str(&format!("({}", name));
        for expr in expressions {
            let expression = expr.accept(self);
            output.push_str(" ");
            output.push_str(&expression);
        }
            output.push_str(")");
        output
    }
}
