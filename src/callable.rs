use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::interpreter::*;
use crate::environment::*;
use crate::stmt::*;
use crate::error::*;

#[derive(Debug, Clone)]
pub struct RloxFunction {
    declaration: FunctionStmt,
}
pub trait RloxCallable {
    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, RloxError>;
    fn arity(&self) -> usize;
}

impl RloxFunction {
    pub fn new(declaration: FunctionStmt) -> Self {
        Self { declaration }
    }
}

impl RloxCallable for RloxFunction {
    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, RloxError> {
        let mut environment = Environment::new(Some(Rc::clone(&interpreter.globals)));

        for (token, val) in self.declaration.params.iter().zip(args.iter()) {
            environment.define(&token.lexeme, val.clone())
        }
        if let Err(err) = interpreter.execute_block(&self.declaration.body, environment) {
            match err{
                RloxError::Return(val) => Ok(val),
                e => Err(e),
            }
        } else {
            Ok(Value::Nil)
        }
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }
}

#[derive(Debug, Clone)]
pub struct RloxNative {}



impl RloxCallable for RloxNative {
    fn call(&self, _interpreter: &mut Interpreter, _args: &[Value]) -> Result<Value, RloxError> {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();

        Ok(Value::Number(since_the_epoch.as_millis() as f64))
    }

    fn arity(&self) -> usize {
        0
    }
}
