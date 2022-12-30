mod ast_printer;
mod error;
mod expr;
mod parser;
mod scanner;
mod interpreter;

use error::RloxError;
use std::{
    env::args,
    fs::read_to_string,
    io::{stdin, stdout, Error, Write},
    process::exit,
};

use crate::{ast_printer::AstPrinter, parser::*};

fn main() -> std::io::Result<()> {
    let args: Vec<_> = args().collect();
    if args.len() > 2 {
        println!("Usage: rlox [script]");
        exit(64);
    } else if args.len() == 2 {
        run_file(&args[1])
    } else {
        run_prompt()
    }
}

fn run_file(path: &str) -> std::io::Result<()> {
    let file = read_to_string(path)?;
    run(&file);
    Ok(())
}
fn run_prompt() -> std::io::Result<()> {
    Ok(loop {
        print!("> ");
        stdout().flush()?;
        let mut line = String::new();
        stdin().read_line(&mut line)?;
        if line.trim().is_empty() {
            break;
        }
        run(&line);
    })
}

fn run(source: &str) -> Result<(), RloxError> {
    let scanner = scanner::Scanner::default().scan_tokens(source.to_string())?;
    let mut parser = Parser {
        tokens: scanner.to_vec(),
        current: 0,
    };
    let expr = parser.parse()?;

    let interpreter = interpreter::Interpreter{};
    interpreter.interpret(expr);
    Ok(())
}
