use std::collections::HashMap;

use crate::{error::*, interpreter::*, scanner::*};

#[derive(Debug, Clone)]
pub struct Environment {
    pub values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Environment {
        Self {
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
            Err(RloxError::RuntimeError {
                lexeme: name.clone(),
                message: format!("Undefined variable {}.", &name),
            })
        }
    }
}
