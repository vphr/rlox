use crate::scanner::*;

#[derive(Debug)]
pub enum RloxError {
    ScanError { character: char, message: String },
    UnterminatedStringError { token: String, message: String },
    ParseError { current: usize, token: Token, message: String},
    InterpreterError,
}

impl RloxError {
    pub fn report(&self){
        match &self {
            RloxError::ScanError { character, message } => {
               eprintln!("[line {}] Error {}", character, message)
            }
            RloxError::ParseError {current, token, message} => {
                eprintln!("[position {}] Error {} lexeme {:?}",current,message, token.lexeme )
            }
            RloxError::InterpreterError => eprintln!("todo: implement interpreter error messages"),
            RloxError::UnterminatedStringError { token, message } => {
               eprintln!("[line {}] Error {}", token, message)
            }
        }
    }
}
