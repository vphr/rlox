use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::environment::*;
use crate::error::*;
use crate::interpreter::*;
use crate::stmt::*;

#[derive(Debug, Clone)]
pub struct RloxFunction {
    parameters: Rc<Vec<String>>,
    body: Rc<Vec<Stmt>>,
    closure: Rc<RefCell<Environment>>,
}

pub trait RloxCallable {
    fn call(&self, interpreter: &mut Interpreter, args: &[Rc<Value>]) -> Result<Rc<Value>, RloxError>;
    fn arity(&self) -> usize;
}

impl std::fmt::Debug for dyn RloxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<lox fn>")
    }
}

impl RloxFunction {
    pub fn new(
        parameters: Rc<Vec<String>>,
        body: Rc<Vec<Stmt>>,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        Self {
            parameters,
            body,
            closure,
        }
    }
}

impl RloxCallable for RloxFunction {
    fn call(&self, interpreter: &mut Interpreter, args: &[Rc<Value>]) -> Result<Rc<Value>, RloxError> {
        let environment = Environment::new(Rc::clone(&self.closure));

        for (token, val) in self.parameters.iter().zip(args.iter()) {
            environment.define(&token.to_string(), val.clone())
        }
        if let Err(err) = interpreter.execute_block(&self.body, Rc::new(RefCell::new(environment))) {
            match err {
                RloxError::Return(val) => Ok(Rc::new(val)),
                e => Err(e),
            }
        } else {
            Ok(Rc::new(Value::Nil))
        }
    }

    fn arity(&self) -> usize {
        self.parameters.len()
    }
}

#[derive(Debug, Clone)]
pub struct Clock {}

impl RloxCallable for Clock {
    fn call(&self, _interpreter: &mut Interpreter, _args: &[Rc<Value>]) -> Result<Rc<Value>, RloxError> {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();

        Ok(Rc::new(Value::Number(since_the_epoch.as_millis() as f64)))
    }

    fn arity(&self) -> usize {
        0
    }
}

#[derive(Debug, Clone)]
pub struct RloxClass {
    pub name: String,
}
