use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use crate::{error::*, interpreter::*};

#[derive(Debug, Clone)]
pub struct Environment {
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    pub values: RefCell<HashMap<String, Rc<Value>>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            enclosing: None,
            values: RefCell::new(HashMap::new()),
        }
    }
}

impl Environment {
    pub fn new(enclosing: Rc<RefCell<Environment>>) -> Environment {
        Self {
            enclosing: Some(enclosing),
            values: RefCell::new(HashMap::new()),
        }
    }
    pub fn define(&self, name: &str, value: Rc<Value>) {
        self.values
            .borrow_mut()
            .insert(name.to_string(), value);
    }
    pub fn get_at(&self, distance: usize, token: &str) -> Result<Rc<Value>, RloxError> {
        if 0 ==distance {
            {
                self.get(token)
            }
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow()
                .get_at(distance - 1, token)
        }
    }

    pub fn get(&self, token: &str) -> Result<Rc<Value>, RloxError> {

        match self.values.borrow().get(token) {
            Some(val) => Ok(val.clone()),
            None => match &self.enclosing {
                Some(enclosing) => enclosing.borrow().get(token),
                None => Err(RloxError::RuntimeError {
                    lexeme: token.to_string(),
                    message: format!("Trying to get undefined variable {}.", &token),
                }),
            },
        }
    }
    pub fn assign_at(
        &mut self,
        distance: &usize,
        token: &str,
        value: Rc<Value>,
    ) -> Result<(), RloxError> {

        if 0.eq(distance) {
            self.values.borrow_mut().insert(token.to_string(), value.clone());
            Ok(())
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow_mut()
                .assign_at(&(distance - 1), token, value)
        }
    }

}
