use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use crate::{error::*, expr::*, interpreter::*, stmt::*};

#[derive(Copy, Clone, PartialEq, Eq)]
enum FunctionType {
    None,
    Function,
}
pub struct Resolver {
    pub interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    depth_map: HashMap<usize, usize>,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            depth_map: HashMap::new(),
            current_function: FunctionType::None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }
    pub fn resolve(&mut self, statements: &Vec<Stmt>) -> Result<(), RloxError> {
        self.resolve_statements(statements)?;
        let scopes = std::mem::take(&mut self.depth_map);
        self.interpreter.add_scopes(scopes);
        Ok(())
    }
    pub fn resolve_statements(&mut self, statements: &Vec<Stmt>) -> Result<(), RloxError> {
        for statement in statements.deref() {
            self.resolve_statement(statement)?;
        }
        Ok(())
    }

    fn resolve_statement(&mut self, statement: &Stmt) -> Result<(), RloxError> {
        match statement {
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve_statements(statements.as_ref())?;
                self.end_scope();
            }
            Stmt::Expression { expression } => {
                self.resolve_expression(expression)?;
            }
            Stmt::Print { expression } => {
                self.resolve_expression(expression)?;
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);
                if let Some(init) = initializer {
                    self.resolve_expression(init)?;
                };
                self.define(name);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(then_branch.as_ref())?;
                if let Some(stmt) = else_branch {
                    self.resolve_statement(stmt)?;
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body.as_ref())?;
            }
            Stmt::Function {
                name,
                parameters,
                body,
            } => {
                self.declare(name);
                self.define(name);
                self.resolve_function(name, parameters, body, FunctionType::Function)?;
            }
            Stmt::Return { value } => {
                if self.current_function == FunctionType::None {
                    return Err(RloxError::InterpreterError);
                }
                if let Some(val) = value {
                    self.resolve_expression(val)?;
                }
            }
        }

        Ok(())
    }

    fn resolve_expression(&mut self, expr: &Expr) -> Result<(), RloxError> {
        match expr {
            Expr::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Expr::Call { callee, arguments } => {
                self.resolve_expression(callee)?;
                for arg in arguments.as_ref() {
                    self.resolve_expression(arg)?;
                }
            }
            Expr::Grouping { expression } => {
                self.resolve_expression(expression)?;
            }
            Expr::Unary { operator: _, right } => {
                self.resolve_expression(right)?;
            }
            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Expr::Variable { id, name } => {
                if let Some(local) = self.scopes.last() {
                    if local.get::<str>(name) == Some(&false) {
                        return Err(RloxError::InterpreterError);
                    }
                    self.resolve_local(*id, name);
                }
            }
            Expr::Assign { id, name, value } => {
                self.resolve_expression(value)?;
                self.resolve_local(*id, name);
            }
            _ => {}
        }
        Ok(())
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str) {
        match self.scopes.last_mut() {
            Some(scope) => {
                scope.insert(name.to_string(), false);
            }
            None => {}
        }
    }

    fn define(&mut self, name: &str) {
        match self.scopes.last_mut() {
            Some(scope) => {
                scope.insert(name.to_string(), true);
            }
            None => {}
        }
    }

    fn resolve_local(&mut self, depth: usize, name: &str) {
        for (index, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.depth_map.insert(depth, index);
            }
        }
    }

    fn resolve_function(
        &mut self,
        name: &str,
        parameters: &Vec<String>,
        body: &Rc<Vec<Stmt>>,
        function_type: FunctionType,
    ) -> Result<(), RloxError> {
        self.declare(name);
        self.define(name);
        let enclosing_function = self.current_function;
        self.current_function = function_type;
        self.begin_scope();
        for token in parameters {
            self.declare(&token);
            self.define(&token);
        }

        self.resolve_statements(&body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }
}
