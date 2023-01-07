use std::collections::HashMap;

use crate::{error::*, expr::*, interpreter::*, scanner::*, stmt::*};

pub struct Resolver {
    pub interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![],
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    pub fn resolve_statements(&mut self, statements: Vec<Stmt>) -> Result<(), RloxError> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }
        Ok(())
    }

    fn resolve_statement(&mut self, statement: Stmt) -> Result<(), RloxError> {
        statement.accept(self)?;
        Ok(())
    }

    fn resolve_expression(&mut self, expression: Expr) -> Result<(), RloxError> {
        expression.accept(self)?;
        Ok(())
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        match self.scopes.last_mut() {
            Some(scope) => {
                let name = String::from_utf8(name.lexeme.clone()).expect("valid string");
                scope.insert(name, false);
            }
            None => {}
        }
    }

    fn define(&mut self, name: &Token) {
        match self.scopes.last_mut() {
            Some(scope) => {
                let name = String::from_utf8(name.lexeme.clone()).expect("valid string");
                scope.insert(name, true);
            }
            None => {}
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: Token) -> Result<(), RloxError> {
        for (index, scope) in self.scopes.iter().rev().enumerate() {
            let name_lexeme = String::from_utf8(name.lexeme.clone()).expect("valid string");
            if scope.contains_key(&name_lexeme) {
                self.interpreter.resolve(expr, index);
                return Ok(());
            }
        }
        Ok(())
    }

    fn resolve_function(&mut self, stmt: &FunctionStmt)-> Result<(), RloxError> {
        self.begin_scope();
        for token in &stmt.params {
            self.declare(token);
            self.define(token);
        }

        self.resolve_statements(stmt.body.clone())?;
        self.end_scope();
        Ok(())
    }
}

impl StmtVisitor<()> for Resolver {
    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> Result<(), RloxError> {
        self.begin_scope();
        self.resolve_statements(stmt.statements.clone())?;
        self.end_scope();

        Ok(())
    }

    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt) -> Result<(), RloxError> {
        self.resolve_expression(*stmt.expression.clone())
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> Result<(), RloxError> {
        self.resolve_expression(*stmt.condition.clone())?;
        self.resolve_statement(*stmt.then_branch.clone())?;
        if let Some(else_branch) = &stmt.else_branch {
            self.resolve_statement(*else_branch.clone())?;
        };
        Ok(())
    }

    fn visit_function_stmt(&mut self, stmt: &FunctionStmt) -> Result<(), RloxError> {
        self.declare(&stmt.name);
        self.define(&stmt.name);
        self.resolve_function(stmt)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, stmt: &PrintStmt) -> Result<(), RloxError> {
        self.resolve_expression(*stmt.expression.clone())
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> Result<(), RloxError> {
        if let Some(val) = &stmt.value {
            self.resolve_expression(*val.clone())?;
        };
        Ok(())
    }

    fn visit_var_stmt(&mut self, stmt: &VarStmt) -> Result<(), RloxError> {
        self.declare(&stmt.name);
        if let Some(init) = &stmt.initializer {
            self.resolve_expression(*init.clone())?;
        };
        self.define(&stmt.name);
        Ok(())
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> Result<(), RloxError> {
        self.resolve_expression(*stmt.condition.clone())?;
        self.resolve_statement(*stmt.body.clone())
    }
}
impl ExprVisitor<()> for Resolver {
    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> Result<(), RloxError> {
        self.resolve_expression(*expr.value.clone())?;
        self.resolve_local(&expr.value.clone(), expr.name.clone())
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Result<(), RloxError> {
        self.resolve_expression(*expr.left.clone())?;
        self.resolve_expression(*expr.right.clone())
    }

    fn visit_call_expr(&mut self, expr: &CallExpr) -> Result<(), RloxError> {
        self.resolve_expression(*expr.callee.clone())?;
        for arg in &expr.arguments {
            self.resolve_expression(*arg.clone())?;
        }
        Ok(())
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Result<(), RloxError> {
        self.resolve_expression(*expr.expression.clone())
    }

    fn visit_literal_expr(&mut self, _expr: &LiteralExpr) -> Result<(), RloxError> {
        Ok(())
    }

    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> Result<(), RloxError> {
        self.resolve_expression(*expr.left.clone())?;
        self.resolve_expression(*expr.right.clone())
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Result<(), RloxError> {
        self.resolve_expression(*expr.right.clone())
    }

    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> Result<(), RloxError> {
        if !self.scopes.is_empty() {
            let name = String::from_utf8(expr.name.lexeme.clone()).expect("valid string");
            if let Some(false) = self.scopes.last().expect("not empty").get(&name) {
                return Err(RloxError::RuntimeError {
                    lexeme: name,
                    message: "Can't read local variable in its own initializer.".to_string(),
                });
            }
        }
let tt = Expr::Variable( VariableExpr{ name: expr.name.clone() });
        self.resolve_local(&tt, expr.name.clone())?;

        Ok(())
    }
}
