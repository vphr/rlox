mod ast_printer;
mod expr;
mod scanner;
mod parser;

use std::{
    env::args,
    fs::read_to_string,
    io::{stdin, stdout, Result, Write},
    process::exit,
};
use crate::{parser::*, ast_printer::AstPrinter};

fn main() -> Result<()> {
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

fn run_file(path: &str) -> Result<()> {
    let file = read_to_string(path)?;
    println!("{:#?}", file);
    run(&file);
    Ok(())
}

fn run_prompt() -> Result<()> {
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

fn run(source: &str) {
    let scanner = scanner::Scanner::default().scan_tokens(source.to_string());
    let mut parser = Parser{ tokens: scanner.to_vec(), current: 0 };
    // println!("{:#?}", scanner);
    let expr = parser.parse();
    // println!("{:#?}", expr);

    println!("{:#?}", AstPrinter{}.print(&expr));

    // let expression = Expr::Binary(BinaryExpr {
    //     left: Box::new(Expr::Unary(UnaryExpr {
    //         operator: Token {
    //             token_type: scanner::TokenType::Minus,
    //             lexeme: "-".as_bytes().to_vec(),
    //             literal: None,
    //             line: 1,
    //         },
    //         right: Box::new(Expr::Literal(LiteralExpr {
    //             value: Some(scanner::Literal::Number(123.0)),
    //         })),
    //     })),
    //     operator: Token {
    //         token_type: scanner::TokenType::Star,
    //         lexeme: "*".as_bytes().to_vec(),
    //         literal: None,
    //         line: 1,
    //     },
    //     right: Box::new(Expr::Grouping(GroupingExpr {
    //         expression: Box::new(Expr::Literal(LiteralExpr {
    //             value: Some(scanner::Literal::Number(45.67)),
    //         })),
    //     })),
    // });
    // println!("{:#?}", AstPrinter{}.print(&expression));
}
