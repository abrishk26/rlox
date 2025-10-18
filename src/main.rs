use rlox::Scanner;
use rlox::parser::{Exec, Parser};

fn main() {
    let text = String::from("print 2 + 1;");
    let mut scanner = Scanner::new(text.chars().peekable());
    let tokens = scanner.scan_tokens();
    println!("{:?}", tokens);
    let mut parser = Parser::new(tokens);
    for stmt in parser.parse().unwrap() {
        stmt.exec();
    }
}
