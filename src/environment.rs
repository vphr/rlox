use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::*, interpreter::*, scanner::*};

#[derive(Debug, Clone)]
pub struct Environment {
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    pub values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Environment {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }
    pub fn new_with_enclosing(enclosing: RefCell<Self>) -> Environment {
        Self {
            enclosing: Some(Rc::new(enclosing)),
            values: HashMap::new(),
        }
    }
    pub fn define(&mut self, name: &Vec<u8>, value: Value) {
        let name = String::from_utf8(name.to_vec()).expect("valid string");
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, token: &Token) -> Result<Value, RloxError> {
        let cloned_lexeme_vec = token.lexeme.to_vec();
        let name = String::from_utf8(cloned_lexeme_vec).expect("valid string");

        if let Some(value) = self.values.get(&name) {
            Ok(value.clone())
        } else {
            if let Some(enclosing) = &self.enclosing {
                return enclosing.borrow_mut().get(token);
            };
            Err(RloxError::RuntimeError {
                lexeme: name.clone(),
                message: format!("Undefined variable {}.", &name),
            })
        }
    }
    pub fn assign(&mut self, token: &Token, value: Value) -> Result<(), RloxError> {
        let cloned_lexeme_vec = token.lexeme.to_vec();
        let name = String::from_utf8(cloned_lexeme_vec).expect("valid string");

        match self.values.insert(name.clone(), value.clone()) {
            Some(_) => Ok(()),
            None => {
                if let Some(enclosing) = &self.enclosing {
                    return enclosing.borrow_mut().assign(token, value);
                };
                return Err(RloxError::RuntimeError {
                    lexeme: name.clone(),
                    message: format!("Undefined variable '{}'.", name),
                });
            }
        }
    }
}
