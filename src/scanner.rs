use crate::types::{KEYWORDS, Object, Token, TokenType};
use std::iter::{Iterator, Peekable};
use std::str::Chars;
use std::str::FromStr;

pub struct Scanner<'a> {
    source: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    line: u64,
    had_error: bool,
}

impl<'a> Scanner<'a> {
    pub fn new(source: Peekable<Chars<'a>>) -> Scanner<'a> {
        Scanner {
            source,
            tokens: Vec::<Token>::new(),
            line: 1,
            had_error: false,
        }
    }

    fn is_alpha(c: char) -> bool {
        match c {
            '_' | 'a'..='z' | 'A'..='Z' => true,
            _ => false,
        }
    }

    pub fn error(&mut self, line: u64, message: &str) {
        self.had_error = true;
        eprintln!("[line: {}] Error: {}", line, message);
    }

    pub fn scan_tokens(&mut self) -> Option<Vec<Token>> {
        while !self.is_at_end() {
            self.scan_token();
        }

        if self.had_error {
            return None;
        }

        self.tokens
            .push(Token::new(TokenType::EOF, self.line, None, Object::None));
        Some(self.tokens.clone())
    }

    fn scan_token(&mut self) {
        let c = self.source.next().unwrap();
        match c {
            '{' => self.tokens.push(Token::new(
                TokenType::LEFTBRACE,
                self.line,
                None,
                Object::None,
            )),
            '}' => self.tokens.push(Token::new(
                TokenType::RIGHTBRACE,
                self.line,
                None,
                Object::None,
            )),
            '(' => self.tokens.push(Token::new(
                TokenType::LEFTPAREN,
                self.line,
                None,
                Object::None,
            )),
            ')' => self.tokens.push(Token::new(
                TokenType::RIGHTPAREN,
                self.line,
                None,
                Object::None,
            )),
            ',' => self
                .tokens
                .push(Token::new(TokenType::COMMA, self.line, None, Object::None)),
            '.' => self
                .tokens
                .push(Token::new(TokenType::DOT, self.line, None, Object::None)),
            ';' => self.tokens.push(Token::new(
                TokenType::SEMICOLON,
                self.line,
                None,
                Object::None,
            )),
            '*' => self
                .tokens
                .push(Token::new(TokenType::STAR, self.line, None, Object::None)),
            '-' => self
                .tokens
                .push(Token::new(TokenType::MINUS, self.line, None, Object::None)),
            '+' => self
                .tokens
                .push(Token::new(TokenType::PLUS, self.line, None, Object::None)),
            '/' => {
                if let Some(c) = self.source.peek() {
                    if *c == '/' {
                        while self.source.peek() != None && *self.source.peek().unwrap() != '\n' {
                            self.source.next();
                        }
                        if self.source.peek() != None {
                            self.line += 1;
                            self.source.next();
                        }
                    } else {
                        self.tokens.push(Token::new(
                            TokenType::SLASH,
                            self.line,
                            None,
                            Object::None,
                        ));
                    }
                } else {
                    self.tokens
                        .push(Token::new(TokenType::SLASH, self.line, None, Object::None));
                }
            }
            '!' => {
                if let Some(c) = self.source.peek() {
                    if *c == '=' {
                        self.tokens.push(Token::new(
                            TokenType::BANGEQUAL,
                            self.line,
                            None,
                            Object::None,
                        ));
                        self.source.next();
                    } else {
                        self.tokens.push(Token::new(
                            TokenType::BANG,
                            self.line,
                            None,
                            Object::None,
                        ));
                    }
                } else {
                    self.tokens
                        .push(Token::new(TokenType::BANG, self.line, None, Object::None));
                }
            }

            '=' => {
                if let Some(c) = self.source.peek() {
                    if *c == '=' {
                        self.tokens.push(Token::new(
                            TokenType::EQUALEQUAL,
                            self.line,
                            None,
                            Object::None,
                        ));
                        self.source.next();
                    } else {
                        self.tokens.push(Token::new(
                            TokenType::EQUAL,
                            self.line,
                            None,
                            Object::None,
                        ));
                    }
                } else {
                    self.tokens
                        .push(Token::new(TokenType::EQUAL, self.line, None, Object::None));
                }
            }

            '>' => {
                if let Some(c) = self.source.peek() {
                    if *c == '=' {
                        self.tokens.push(Token::new(
                            TokenType::GREATEREQUAL,
                            self.line,
                            None,
                            Object::None,
                        ));
                        self.source.next();
                    } else {
                        self.tokens.push(Token::new(
                            TokenType::GREATER,
                            self.line,
                            None,
                            Object::None,
                        ));
                    }
                } else {
                    self.tokens.push(Token::new(
                        TokenType::GREATER,
                        self.line,
                        None,
                        Object::None,
                    ));
                }
            }
            '<' => {
                if let Some(c) = self.source.peek() {
                    if *c == '=' {
                        self.tokens.push(Token::new(
                            TokenType::LESSEQUAL,
                            self.line,
                            None,
                            Object::None,
                        ));
                        self.source.next();
                    } else {
                        self.tokens.push(Token::new(
                            TokenType::LESS,
                            self.line,
                            None,
                            Object::None,
                        ));
                    }
                } else {
                    self.tokens
                        .push(Token::new(TokenType::LESS, self.line, None, Object::None));
                }
            }
            '"' => {
                let mut buf = String::new();
                while !self.is_at_end() && *self.source.peek().unwrap() != '"' {
                    if *self.source.peek().unwrap() == '\n' {
                        self.line += 1;
                    } else {
                        buf.push(*self.source.peek().unwrap());
                    }

                    self.source.next();
                }

                if self.is_at_end() {
                    self.error(self.line, "Unterminated string.");
                } else {
                    self.source.next();

                    self.tokens.push(Token::new(
                        TokenType::STRING,
                        self.line,
                        Some(buf.clone()),
                        Object::Str(buf),
                    ));
                }
            }
            '0'..='9' => {
                let mut buf = String::from(c);
                while self.source.peek() != None
                    && ((*self.source.peek().unwrap()).is_digit(10)
                        || *self.source.peek().unwrap() == '.')
                {
                    buf.push(*self.source.peek().unwrap());
                    self.source.next();
                }
                self.tokens.push(Token::new(
                    TokenType::NUMBER,
                    self.line,
                    Some(buf.clone()),
                    Object::Num(f64::from_str(&buf).unwrap()),
                ));
            }
            '_' | 'a'..='z' | 'A'..='Z' => {
                let mut buf = String::from(c);
                while self.source.peek() != None && Scanner::is_alpha(*self.source.peek().unwrap())
                {
                    buf.push(*self.source.peek().unwrap());
                    self.source.next();
                }
                match KEYWORDS.get(buf.as_str()) {
                    Some(k) => {
                        self.tokens
                            .push(Token::new(k.clone(), self.line, None, Object::None))
                    }
                    None => self.tokens.push(Token::new(
                        TokenType::IDENTIFIER,
                        self.line,
                        Some(buf),
                        Object::None,
                    )),
                }
            }
            ' ' | '\t' | '\r' => (),
            '\n' => self.line += 1,
            _ => self.error(self.line, "Unexpected character."),
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.source.peek() == None
    }
}
