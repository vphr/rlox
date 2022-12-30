use crate::scanner::*;

#[derive(Debug)]
pub enum RloxError {
    ScanError { character: char, message: String },
    ParseError { current: usize, token: Token, message: String},
}

impl RloxError {
    pub fn report(&self) -> String {
        match &self {
            RloxError::ScanError { character, message } => {
               format!("[line {}] Error {}", character, message)
            }
            RloxError::ParseError {current, token, message} => {
                format!("[position {}] Error {} lexeme {:?}",current,message, token.lexeme )
            }
        }
    }
}
