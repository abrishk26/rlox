use rlox::parser::Parser;
use rlox::interpreter::Interpreter;
use rlox::scanner::Scanner;
use std::env;

fn main() {
    if env::args().len() > 2 {
        println!("Usage: rlox [script]");
        return;
    }

    let file_name = env::args().nth(1).unwrap();
    run_file(&file_name);
}

fn run(source: String) {
    let tokens = Scanner::new(source.chars().peekable()).scan_tokens();
    match tokens {
        Some(t) => match Parser::new(t).parse() {
            Ok(e) => {
                (&mut Interpreter::new()).interpret(e);
            }
            _ => std::process::exit(67),
        },
        None => std::process::exit(67),
    }
}

fn run_file(path: &str) {
    let source = std::fs::read_to_string(path).unwrap();
    run(source);
}
