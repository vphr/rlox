mod scanner;

use std::{
    env::args,
    fs::read_to_string,
    io::{stdin, stdout, Result, Write},
    process::exit,
};

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
    // let scanner = scanner::scan_tokens(source.to_string());
    print!("{:#?}", scanner);
}

