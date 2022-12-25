use crate::scanner::*;

#[derive(Debug)]
pub enum RloxError {
    ParseError { character: char, message: String },
}

impl RloxError {
    fn report(&self) {
        match &self {
            RloxError::ParseError { character, message } => {
                eprintln!("[line {}] Error {}", character, message)
            }
        }
    }
}
