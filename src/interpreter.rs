use crate::callable::*;
use crate::environment::*;
use crate::error::RloxError;
use crate::expr::Expr;
use crate::scanner::*;
use crate::stmt::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<usize, usize>,
}
#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Number(f64),
    Bool(bool),
    Func(Rc<dyn RloxCallable>),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Value::Nil => write!(f, "nil"),
            Value::Number(num) => {
                if num.fract() == 0.0 {
                    write!(f, "{:.0}", num)
                } else {
                    write!(f, "{}", num)
                }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Func(func) => write!(f, "{:?}", func),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            _ => false,
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::default()));
        globals
            .borrow_mut()
            .define("clock", Rc::new(Value::Func(Rc::new(Clock {}))));
        
        Self {
            globals: globals.clone(),
            environment: globals,
            locals: HashMap::new(),
        }
    }
    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), RloxError> {
        for statement in statements {
            self.execute(&statement)?
        }
        Ok(())
    }
    fn evaluate(&mut self, expr: &Expr) -> Result<Rc<Value>, RloxError> {
        match expr {
            Expr::Nil => Ok(Rc::new(Value::Nil)),
            Expr::Number(n) => Ok(Rc::new(Value::Number(*n))),
            Expr::String(s) => Ok(Rc::new(Value::Str(s.to_string()))),
            Expr::Boolean(b) => Ok(Rc::new(Value::Bool(*b))),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.binary_expr(left, &operator.token_type, right),
            Expr::Unary { operator, right } => self.unary_expr(&operator.token_type, right),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.logical_expr(left, &operator.token_type, right),
            Expr::Variable { id, name } => {
                let depth = self.locals.get(id).copied();
                if let Some(depth) = depth {
                    self.environment.borrow().get_at(depth, name)
                } else {
                    self.globals.borrow().get_at(0, name)
                }
            }
            Expr::Assign { id, name, value } => {
                let value = self.evaluate(value)?;
                let depth = self.locals.get(id).copied();

                if let Some(depth) = depth {
                    self.environment
                        .borrow_mut()
                        .assign_at(&depth, name, value.clone())?;
                } else {
                    self.globals
                        .borrow_mut()
                        .assign_at(&0, name, value.clone())?;
                }
                Ok(value)
            }
            Expr::Call { callee, arguments } => self.call_expr(callee, arguments),
        }
    }

    fn logical_expr(
        &mut self,
        left: &Expr,
        operator: &TokenType,
        right: &Expr,
    ) -> Result<Rc<Value>, RloxError> {
        let left = self.evaluate(left)?;

        if *operator == TokenType::Or {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        } else {
            if !self.is_truthy(&left) {
                return Ok(left);
            }
        }
        self.evaluate(right)
    }
    fn binary_expr(
        &mut self,
        left: &Expr,
        token_type: &TokenType,
        right: &Expr,
    ) -> Result<Rc<Value>, RloxError> {
        let left = &*self.evaluate(left)?;
        let right = &*self.evaluate(right)?;

        match (left, token_type, right) {
            (Value::Number(l), TokenType::Star, Value::Number(r)) => {
                Ok(Rc::new(Value::Number(l * r)))
            }
            (Value::Number(l), TokenType::Minus, Value::Number(r)) => {
                Ok(Rc::new(Value::Number(l - r)))
            }
            (Value::Number(l), TokenType::Plus, Value::Number(r)) => {
                Ok(Rc::new(Value::Number(l + r)))
            }
            (Value::Number(l), TokenType::Greater, Value::Number(r)) => {
                Ok(Rc::new(Value::Bool(l.gt(&r))))
            }
            (Value::Number(l), TokenType::GreaterEqual, Value::Number(r)) => {
                Ok(Rc::new(Value::Bool(l.ge(&r))))
            }
            (Value::Number(l), TokenType::Less, Value::Number(r)) => {
                Ok(Rc::new(Value::Bool(l.lt(&r))))
            }
            (Value::Number(l), TokenType::LessEqual, Value::Number(r)) => {
                Ok(Rc::new(Value::Bool(l.le(&r))))
            }
            (Value::Str(l), TokenType::Plus, Value::Str(r)) => {
                Ok(Rc::new(Value::Str(l.clone() + &r)))
            }
            (Value::Number(l), TokenType::EqualEqual, Value::Number(r)) => {
                Ok(Rc::new(Value::Bool(l.eq(&r))))
            }
            (Value::Number(l), TokenType::BangEqual, Value::Number(r)) => {
                Ok(Rc::new(Value::Bool(!l.eq(&r))))
            }
            (Value::Bool(l), TokenType::EqualEqual, Value::Bool(r)) => {
                Ok(Rc::new(Value::Bool(l.eq(&r))))
            }
            (Value::Bool(l), TokenType::BangEqual, Value::Bool(r)) => {
                Ok(Rc::new(Value::Bool(!l.eq(&r))))
            }
            _ => Err(RloxError::InterpreterError),
        }
    }
    fn unary_expr(&mut self, token_type: &TokenType, expr: &Expr) -> Result<Rc<Value>, RloxError> {
        let right = self.evaluate(expr)?;
        match token_type {
            TokenType::Minus => match *right {
                Value::Number(n) => Ok(Rc::new(Value::Number(-n))),
                _ => Err(RloxError::InterpreterError),
            },
            TokenType::Bang => Ok(Rc::new(Value::Bool(!self.is_truthy(&right)))),
            _ => Err(RloxError::InterpreterError),
        }
    }
    // anything except null and false is true
    fn is_truthy(&self, right: &Value) -> bool {
        match *right {
            Value::Bool(false) | Value::Nil => false,
            _ => true,
        }
    }

    pub fn stringify(value: &Value) -> String {
        match &value {
            Value::Str(s) => s.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
            Value::Func(_) => "<func>".to_string(),
        }
    }
    pub fn add_scopes(&mut self, scopes: HashMap<usize, usize>) {
        scopes.iter().for_each(|(&k, &v)| {
            self.locals.insert(k, v);
        });
    }

    fn execute(&mut self, statement: &Stmt) -> Result<(), RloxError> {
        match statement {
            Stmt::Print { expression } => {
                println!("{:?}", self.evaluate(expression)?);
                Ok(())
            }
            Stmt::Expression { expression } => {
                self.evaluate(expression)?;
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(expression) = initializer {
                    self.evaluate(&expression)?
                } else {
                    Rc::new(Value::Nil)
                };

                self.environment.borrow_mut().define(&name, value);
                Ok(())
            }
            Stmt::Block { statements } => self.execute_block(
                statements,
                Rc::new(RefCell::new(Environment::new(self.environment.clone()))),
            ),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.evaluate(condition)?;
                if self.is_truthy(&condition) {
                    self.execute(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)
                } else {
                    Ok(())
                }
            }
            Stmt::While { condition, body } => {
                let mut evaluated_condition = self.evaluate(&condition)?;
                while self.is_truthy(&evaluated_condition) {
                    self.execute(body)?;
                    evaluated_condition = self.evaluate(&condition)?;
                }
                Ok(())
            }
            Stmt::Function {
                name,
                parameters,
                body,
            } => {
                let function = Rc::new(Value::Func(Rc::new(RloxFunction::new(
                    parameters.clone(),
                    body.clone(),
                    self.environment.clone(),
                ))));
                self.environment.borrow_mut().define(&name, function);
                Ok(())
            }
            Stmt::Return { value } => {
                let value = if let Some(value) = value {
                    self.evaluate(value)?
                } else {
                    Rc::new(Value::Nil)
                };
                Err(RloxError::Return(value.as_ref().clone()))
            }
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        new_env: Rc<RefCell<Environment>>,
    ) -> Result<(), RloxError> {
        let previous = self.environment.clone();
        self.environment = new_env;

        for statement in statements {
            self.execute(statement).map_err(|err| {
                self.environment = previous.clone();
                err
            })?;
        }
        self.environment = previous;
        Ok(())
    }

    fn call_expr(&mut self, callee: &Expr, arguments: &Vec<Expr>) -> Result<Rc<Value>, RloxError> {
        let callee = self.evaluate(callee)?;

        let mut args: Vec<Rc<Value>> = vec![];

        for arg in arguments {
            args.push(self.evaluate(arg)?);
        }

        if let Value::Func(function) = callee.as_ref() {
            if !arguments.len().eq(&function.arity()) {
                return Err(RloxError::InterpreterError);
            }
            return function.call(self, &args);
        } else {
            return Err(RloxError::InterpreterError);
        }
    }
}
