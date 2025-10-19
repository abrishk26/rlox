use rlox::parser::{Interpreter, Parser};
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
    Interpreter::new(
        Parser::new(Scanner::new(source.chars().peekable()).scan_tokens())
            .parse()
            .unwrap(),
    )
    .parse();
}

fn run_file(path: &str) {
    let source = std::fs::read_to_string(path).unwrap();
    run(source);
}
