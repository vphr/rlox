use crate::{expr::*, error::RloxError};
use crate::scanner::*;

pub struct AstPrinter {}

impl ExprVisitor<String> for AstPrinter {
    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<String, RloxError> {
        self.parenthesize(
            String::from_utf8(expr.operator.lexeme.to_vec()).unwrap(),
            vec![expr.left.as_ref(), expr.right.as_ref()],
        )
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<String, RloxError>{
        self.parenthesize("grouping".to_string(), vec![&expr.expression])
    }

    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<String,RloxError> {
        if let Some(value) = &expr.value {
            match value {
                Literal::Identifier(val) => Ok( val.to_string() ),
                Literal::Str(val) => Ok(val.to_string()),
                Literal::Number(val) => Ok(val.to_string()),
                Literal::True => Ok("true".to_string()),
                Literal::False => Ok("false".to_string()),
                Literal::Nil => Ok("nil".to_string())
            }
        } else {
            Ok("nil".to_string())
        }
    }

    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<String,RloxError> {
        self.parenthesize(
            String::from_utf8(expr.operator.lexeme.to_vec()).unwrap(),
            vec![expr.right.as_ref()],
        )
    }

    fn visit_variable_expr(&self, variable: &VariableExpr) -> Result<String, RloxError> {
        todo!()
    }
}
impl AstPrinter {
    pub fn print(&self, expr: &Expr) -> Result<String,RloxError> {
        expr.accept(self)
    }

    fn parenthesize(&self, name: String, expressions: Vec<&Expr>) -> Result<String,RloxError> {
        let mut output = String::new();
        output.push_str(&format!("({}", name));
        for expr in expressions {
            let expression = expr.accept(self)?;
            output.push_str(" ");
            output.push_str(&expression);
        }
            output.push_str(")");
        Ok(output)
    }
}
