use crate::{scanner::*, interpreter::{Value, Interpreter}};

#[derive(Debug)]
pub enum RloxError {
    ScanError { character: char, message: String },
    UnterminatedStringError { token: String, message: String },
    ParseError { current: usize, token: Token, message: String},
    RuntimeError { lexeme: String, message: String},
    Return(Value),
    InterpreterError,
}

impl RloxError {
    pub fn report(&self){
        match &self {
            RloxError::ScanError { character, message } => {
               eprintln!("[line {}] Error {}", character, message)
            }
            RloxError::ParseError {current, token, message} => {
                eprintln!("[position {}] Error {} lexeme {:?}",current,message, String::from_utf8(token.lexeme.clone()).unwrap() )
            }
            RloxError::InterpreterError => eprintln!("todo: implement interpreter error messages"),
            RloxError::UnterminatedStringError { token, message } => {
               eprintln!("[line {}] Error {}", token, message)
            }
            RloxError::RuntimeError { lexeme, message } =>
               eprintln!("[token {}] Error {}", lexeme, message),
            RloxError::Return(a) => eprintln!("{}", Interpreter::stringify(a)),

        }
    }
}
