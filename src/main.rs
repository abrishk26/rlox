use rlox::Scanner;
use rlox::parser::{Eval, Parser};

fn main() {
    let text = String::from("10 * 10 < 900 != false");
    let mut scanner = Scanner::new(text.chars().peekable());
    let tokens = scanner.scan_tokens();
    println!("{:?}", tokens);
    let mut parser = Parser::new(tokens);
    let expr = parser.expr().unwrap();
    println!("expression ({})\nvalue ({})", expr, expr.eval());
}
