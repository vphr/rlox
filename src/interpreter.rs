use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::environment::*;
use crate::error::RloxError;
use crate::expr::*;
use crate::scanner::*;
use crate::stmt::*;

#[derive(Debug, Clone)]
pub struct Interpreter {
    globals: RefCell<Environment>,
    environment: RefCell<Environment>,
}
#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Number(f64),
    Bool(bool),
    Func(Callable),
    Nil,
}

#[derive(Clone)]
pub struct Callable {
    pub func: Rc<dyn RloxCallable>,
    pub arity: usize,
}

impl std::fmt::Debug for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<callable>")
    }
}

pub trait RloxCallable {
    fn call(&self, interpreter: &Interpreter, args: &[Value]) -> Result<Value, RloxError>;
    fn arity(&self) -> usize;
}

impl RloxCallable for Callable {
    fn call(&self, interpreter: &Interpreter, args: &[Value]) -> Result<Value, RloxError> {
        self.func.call(interpreter, args)
    }

    fn arity(&self) -> usize {
        print!("{}", self.arity);
        self.arity
    }
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

    fn visit_variable_expr(&self, variable: &VariableExpr) -> Result<Value, RloxError> {
        self.environment.borrow().get(&variable.name)
    }

    fn visit_assign_expr(&self, assign: &AssignExpr) -> Result<Value, RloxError> {
        let value = self.evaluate(*assign.value.clone())?;
        self.environment
            .borrow_mut()
            .assign(&assign.name.clone(), &value.clone())?;
        Ok(value)
    }

    fn visit_logical_expr(&self, visitor: &LogicalExpr) -> Result<Value, RloxError> {
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

    fn visit_call_expr(&self, expr: &CallExpr) -> Result<Value, RloxError> {
        let callee = self.evaluate(*expr.callee.clone())?;

        let mut arguments: Vec<Value> = vec![];

        for args in &expr.arguments {
            arguments.push(self.evaluate(*args.clone())?);
        }

        if let Value::Func(function) = callee {
            if !arguments.len().eq(&function.arity) {
                return Err(RloxError::InterpreterError);
            }
            return function.func.call(self, &arguments);
        } else {
            return Err(RloxError::InterpreterError);
        }
        // let function =
    }
}

impl StmtVisitor<()> for Interpreter {
    fn visit_expression_stmt(&self, stmt: &ExpressionStmt) -> Result<(), RloxError> {
        let e = stmt.expression.as_ref();
        let ee = e.clone();
        self.evaluate(ee)?;
        Ok(())
    }

    fn visit_print_stmt(&self, visitor: &PrintStmt) -> Result<(), RloxError> {
        let e = visitor.expression.as_ref();
        let ee = e.clone();
        let value = self.evaluate(ee)?;
        println!("{}", self.stringify(value));
        Ok(())
    }

    fn visit_var_stmt(&self, stmt: &VarStmt) -> Result<(), RloxError> {
        let value = match &stmt.initializer {
            Some(val) => self.evaluate(*val.clone())?,
            None => Value::Nil,
        };
        self.environment
            .borrow_mut()
            .define(&stmt.name.lexeme, value);
        Ok(())
    }

    fn visit_block_stmt(&self, stmt: &BlockStmt) -> Result<(), RloxError> {
        self.execute_block(
            stmt,
            RefCell::new(Environment::new_with_enclosing(self.environment.clone())),
        )?;
        Ok(())
    }

    fn visit_if_stmt(&self, stmt: &IfStmt) -> Result<(), RloxError> {
        if self.is_truthy(self.evaluate(*stmt.condition.clone())?) {
            self.execute(*stmt.then_branch.clone())?
        } else if let Some(v) = &stmt.else_branch {
            return self.execute(*v.clone());
        }
        Ok(())
    }

    fn visit_while_stmt(&self, stmt: &WhileStmt) -> Result<(), RloxError> {
        // dbg!(&self.environment.as_ref().borrow().enclosing);
        while self.is_truthy(self.evaluate(*stmt.condition.clone())?) {
            self.execute(*stmt.body.clone())?;
        }
        Ok(())
    }
}

pub struct NativeClock;
impl RloxCallable for NativeClock {
    fn call(&self, interpreter: &Interpreter, args: &[Value]) -> Result<Value, RloxError> {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();

        Ok(Value::Number(since_the_epoch.as_millis() as f64))
    }

    fn arity(&self) -> usize {
        0
    }
}
impl Interpreter {
    pub fn new() -> Self {
        let globals = RefCell::new(Environment::new());
        let name = "clock".as_bytes();
        globals.borrow_mut().define(
            &name.to_vec(),
            Value::Func(Callable {
                func: Rc::new(NativeClock {}),
                arity: 0,
            }),
        );

        let environment = globals.clone();
        Self {
            globals,
            environment,
        }
    }
    pub fn interpret(&self, statements: Vec<Stmt>) -> Result<(), RloxError> {
        for statement in statements {
            self.execute(statement)?
        }
        Ok(())
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
            Value::Func(_) => "<func>".to_string(),
        }
    }

    fn execute(&self, statement: Stmt) -> Result<(), RloxError> {
        statement.accept(self)
    }

    fn execute_block(
        &self,
        block: &BlockStmt,
        new_env: RefCell<Environment>,
    ) -> Result<(), RloxError> {
        let mut previous = std::mem::replace(
            &mut *self.environment.borrow_mut(),
            new_env.borrow().clone(),
        );

        let mut result = Ok(());

        for statement in &block.statements {
            if let Err(e) = self.execute(statement.clone()) {
                result = Err(e);
                break;
            };
        }
        if let Some(enclosing) = self.environment.borrow().enclosing.clone() {
            std::mem::swap(&mut previous, &mut enclosing.borrow_mut().clone());
        }
        result
    }
}
