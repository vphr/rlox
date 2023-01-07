// mod ast_printer;
mod environment;
mod error;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod callable;
mod resolver;

use error::RloxError;
use std::{
    env::args,
    fs::read_to_string,
    io::{stdin, stdout, Write},
    process::exit,
};

// use crate::ast_printer::AstPrinter;
use crate::interpreter::*;
use crate::parser::*;
use crate::resolver::*;

struct Rlox {
    interpreter: Interpreter,
}

impl Rlox {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }
    pub fn run_file(&mut self, path: &str) -> std::io::Result<()> {
        let file = read_to_string(path)?;
        match self.run(&file) {
            Ok(_) => {}
            Err(e) => {
                e.report();
            }
        }
        Ok(())
    }

    pub fn run_prompt(&mut self) -> std::io::Result<()> {
        Ok(loop {
            print!("> ");
            stdout().flush()?;
            let mut line = String::new();
            stdin().read_line(&mut line)?;
            if line.trim().is_empty() {
                break;
            }
            match self.run(&line) {
                Ok(_) => {}
                Err(e) => {
                    e.report();
                }
            }
        })
    }
    pub fn run(&mut self, source: &str) -> Result<(), RloxError> {
        let scanner = scanner::Scanner::default().scan_tokens(source.to_string())?;
        let mut parser = Parser {
            tokens: scanner.to_vec(),
            current: 0,
        };
        // println!("{:#?}", scanner);
        let statements = parser.parse()?;
        // println!("{:#?}", statements);

        let mut resolver = Resolver::new(self.interpreter.clone());
        resolver.resolve_statements(statements.clone())?;
        resolver.interpreter.interpret(statements)
    }
}

fn main() -> std::io::Result<()> {
    let mut rlox = Rlox::new();
    let args: Vec<_> = args().collect();
    if args.len() > 2 {
        println!("Usage: rlox [script]");
        exit(64);
    } else if args.len() == 2 {
        rlox.run_file(&args[1])
    } else {
        rlox.run_prompt()
    }
}
