use crate::callable::*;
use crate::environment::*;
use crate::error::RloxError;
use crate::expr::*;
use crate::scanner::*;
use crate::stmt::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    pub environment: Rc<RefCell<Environment>>,
}
#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Number(f64),
    Bool(bool),
    Func(RloxFunction),
    Native(RloxNative),
    Nil,
}

impl ExprVisitor<Value> for Interpreter {
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Result<Value, RloxError> {
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
                TokenType::EqualEqual => Ok(Value::Bool(l.eq(&r))),
                TokenType::BangEqual => Ok(Value::Bool(l.eq(&r))),
                _ => Err(RloxError::InterpreterError),
            },
            (Value::Str(l), Value::Str(r)) => match expr.operator.token_type {
                TokenType::Plus => Ok(Value::Str(l + &r)),
                _ => Err(RloxError::InterpreterError),
            },
            (left, right) => match expr.operator.token_type {
                TokenType::EqualEqual => self.is_equal(left, right),
                TokenType::BangEqual => self.is_equal(left, right),
                _ => Err(RloxError::InterpreterError),
            },
        }
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Result<Value, RloxError> {
        self.evaluate(*expr.expression.clone())
    }

    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> Result<Value, RloxError> {
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

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Result<Value, RloxError> {
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

    fn visit_variable_expr(&mut self, variable: &VariableExpr) -> Result<Value, RloxError> {
        self.environment.borrow().get(&variable.name)
    }

    fn visit_assign_expr(&mut self, assign: &AssignExpr) -> Result<Value, RloxError> {
        let value = self.evaluate(*assign.value.clone())?;
        self.environment
            .borrow_mut()
            .assign(&assign.name.clone(), &value.clone())?;
        Ok(value)
    }

    fn visit_logical_expr(&mut self, visitor: &LogicalExpr) -> Result<Value, RloxError> {
        let left = self.evaluate(*visitor.left.clone())?;

        if visitor.operator.token_type == TokenType::Or {
            if self.is_truthy(left.clone()) {
                return Ok(left);
            }
        } else {
            if !self.is_truthy(left.clone()) {
                return Ok(left);
            }
        }
        self.evaluate(*visitor.right.clone())
    }

    fn visit_call_expr(&mut self, expr: &CallExpr) -> Result<Value, RloxError> {
        let callee = self.evaluate(*expr.callee.clone())?;

        let mut arguments: Vec<Value> = vec![];

        for args in &expr.arguments {
            arguments.push(self.evaluate(*args.clone())?);
        }

        if let Value::Func(function) = callee {
            if !arguments.len().eq(&function.arity()) {
                return Err(RloxError::InterpreterError);
            }
            return function.call(self, &arguments);
        } else {
            return Err(RloxError::InterpreterError);
        }
    }
}

impl StmtVisitor<()> for Interpreter {
    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt) -> Result<(), RloxError> {
        let e = stmt.expression.as_ref();
        let ee = e.clone();
        self.evaluate(ee)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, visitor: &PrintStmt) -> Result<(), RloxError> {
        let e = visitor.expression.as_ref();
        let ee = e.clone();
        let value = self.evaluate(ee)?;
        println!("{}", Interpreter::stringify(&value));
        Ok(())
    }

    fn visit_var_stmt(&mut self, stmt: &VarStmt) -> Result<(), RloxError> {
        let value = match &stmt.initializer {
            Some(val) => self.evaluate(*val.clone())?,
            None => Value::Nil,
        };
        self.environment
            .borrow_mut()
            .define(&stmt.name.lexeme, value);
        Ok(())
    }

    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> Result<(), RloxError> {
        self.execute_block(
            &stmt.statements,
            Environment::new(Some(Rc::clone(&self.environment))),
        )?;
        Ok(())
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> Result<(), RloxError> {
        let evaluated = self.evaluate(*stmt.condition.clone())?;
        if self.is_truthy(evaluated) {
            self.execute(*stmt.then_branch.clone())?
        }
        match &stmt.else_branch {
            Some(eb) => self.execute(*eb.clone())?,
            None => {}
        }
        Ok(())
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> Result<(), RloxError> {
        loop {
            let evaluated_condition = self.evaluate(*stmt.condition.clone())?;
            if self.is_truthy(evaluated_condition) {
                self.execute(*stmt.body.clone())?
            } else {
                break;
            }
        }
        Ok(())
    }

    fn visit_function_stmt(&mut self, stmt: &FunctionStmt) -> Result<(), RloxError> {
        let function = RloxFunction::new(stmt.clone());
        self.environment
            .borrow_mut()
            .define(&stmt.name.lexeme, Value::Func(function));
        Ok(())
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> Result<(), RloxError> {
        let value = match &stmt.value {
            Some(val) => self.evaluate(*val.clone())?,
            None => Value::Nil,
        };

        Err(RloxError::Return(value))
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::default()));
        let name = "clock".as_bytes();
        globals
            .borrow_mut()
            .define(&name.to_vec(), Value::Native(RloxNative {}));

        // let environment = globals.clone();
        Self {
            globals: Rc::clone(&globals),
            environment: globals,
        }
    }
    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), RloxError> {
        for statement in statements {
            self.execute(statement)?
        }
        Ok(())
    }
    fn evaluate(&mut self, expr: Expr) -> Result<Value, RloxError> {
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

    pub fn stringify(value: &Value) -> String {
        match &value {
            Value::Str(s) => s.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
            Value::Func(_) => "<func>".to_string(),
            Value::Native(_) => "<native>".to_string(),
        }
    }

    fn execute(&mut self, statement: Stmt) -> Result<(), RloxError> {
        statement.accept(self)
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        new_env: Environment,
    ) -> Result<(), RloxError> {
        // let prev = self.environment.clone();
        //
        // let mut a = Rc::new(RefCell::new(new_env));

        let previous = self.environment.clone();
        self.environment = Rc::new(RefCell::new(new_env));
        // let mut previous = std::mem::swap(
        //     self.environment.borrow_mut(),
        //     &mut a,
        // );
        //
        let mut result = Ok(());

        for statement in statements {
            if let Err(e) = self.execute(statement.clone()) {
                result = Err(e);
                break;
            };
        }
        // if let Some(enclosing) = self.environment.borrow().enclosing.clone() {
        //     std::mem::swap(&mut prev, &mut enclosing);
        // }
        self.environment = previous;
        result
    }
}
