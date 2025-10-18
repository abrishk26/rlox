use rlox::parser::{Interpreter, Parser};
use rlox::scanner::Scanner;

fn main() {
    let text = String::from("var test = \"Abreham Kassa\"; print test;");
    let mut scanner = Scanner::new(text.chars().peekable());
    let tokens = scanner.scan_tokens();
    println!("{:?}", tokens);
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().unwrap();
    let mut interpreter = Interpreter::new(stmts);
    interpreter.parse();
}
