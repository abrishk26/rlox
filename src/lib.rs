use TokenType::*;
use std::collections::HashMap;
use std::fmt;
use std::iter::{Iterator, Peekable};
use std::str::Chars;
use std::str::FromStr;
use std::sync::LazyLock;

pub mod parser;

static KEYWORDS: LazyLock<HashMap<&'static str, TokenType>> = LazyLock::new(|| {
    HashMap::from([
        ("and", AND),
        ("class", CLASS),
        ("else", ELSE),
        ("false", FALSE),
        ("fun", FUN),
        ("for", FOR),
        ("if", IF),
        ("nil", NIL),
        ("or", OR),
        ("print", PRINT),
        ("return", RETURN),
        ("super", SUPER),
        ("this", THIS),
        ("true", TRUE),
        ("var", VAR),
        ("while", WHILE),
    ])
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens.
    LEFTPAREN,
    RIGHTPAREN,
    LEFTBRACE,
    RIGHTBRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // one or two character tokens.
    BANG,
    BANGEQUAL,
    EQUAL,
    EQUALEQUAL,
    GREATER,
    GREATEREQUAL,
    LESS,
    LESSEQUAL,

    // Literals
    IDENTIFIER,
    STRING,
    NUMBER,

    // Keywords
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}

#[derive(Debug, Clone)]
pub enum Object {
    Num(f64),
    Str(String),
    Bool(bool),
    None,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(n) => write!(f, "{}", n),
            Self::Str(s) => write!(f, "{}", s),
            Self::Bool(b) => write!(f, "{}", b),
            Self::None => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    line: u64,
    lexeme: Option<String>,
    literal: Object,
}

impl Token {
    fn new(token_type: TokenType, line: u64, lexeme: Option<String>, literal: Object) -> Token {
        Token {
            token_type,
            line,
            lexeme,
            literal,
        }
    }
}

pub struct Scanner<'a> {
    source: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    line: u64,
}

impl<'a> Scanner<'a> {
    pub fn new(source: Peekable<Chars<'a>>) -> Scanner<'a> {
        Scanner {
            source,
            tokens: Vec::<Token>::new(),
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.scan_token();
        }

        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.source.next().unwrap();
        match c {
            '{' => self
                .tokens
                .push(Token::new(LEFTBRACE, self.line, None, Object::None)),
            '}' => self
                .tokens
                .push(Token::new(RIGHTBRACE, self.line, None, Object::None)),
            '(' => self
                .tokens
                .push(Token::new(LEFTPAREN, self.line, None, Object::None)),
            ')' => self
                .tokens
                .push(Token::new(RIGHTPAREN, self.line, None, Object::None)),
            ',' => self
                .tokens
                .push(Token::new(COMMA, self.line, None, Object::None)),
            '.' => self
                .tokens
                .push(Token::new(DOT, self.line, None, Object::None)),
            ';' => self
                .tokens
                .push(Token::new(SEMICOLON, self.line, None, Object::None)),
            '*' => self
                .tokens
                .push(Token::new(STAR, self.line, None, Object::None)),
            '-' => self
                .tokens
                .push(Token::new(MINUS, self.line, None, Object::None)),
            '+' => self
                .tokens
                .push(Token::new(PLUS, self.line, None, Object::None)),
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
                        self.tokens
                            .push(Token::new(SLASH, self.line, None, Object::None));
                    }
                } else {
                    self.tokens
                        .push(Token::new(SLASH, self.line, None, Object::None));
                }
            }
            '!' => {
                if let Some(c) = self.source.peek() {
                    if *c == '=' {
                        self.tokens
                            .push(Token::new(BANGEQUAL, self.line, None, Object::None));
                        self.source.next();
                    } else {
                        self.tokens
                            .push(Token::new(BANG, self.line, None, Object::None));
                    }
                } else {
                    self.tokens
                        .push(Token::new(BANG, self.line, None, Object::None));
                }
            }

            '=' => {
                if let Some(c) = self.source.peek() {
                    if *c == '=' {
                        self.tokens
                            .push(Token::new(EQUALEQUAL, self.line, None, Object::None));
                        self.source.next();
                    } else {
                        self.tokens
                            .push(Token::new(EQUAL, self.line, None, Object::None));
                    }
                } else {
                    self.tokens
                        .push(Token::new(EQUAL, self.line, None, Object::None));
                }
            }

            '>' => {
                if let Some(c) = self.source.peek() {
                    if *c == '=' {
                        self.tokens
                            .push(Token::new(GREATEREQUAL, self.line, None, Object::None));
                        self.source.next();
                    } else {
                        self.tokens
                            .push(Token::new(GREATER, self.line, None, Object::None));
                    }
                } else {
                    self.tokens
                        .push(Token::new(GREATER, self.line, None, Object::None));
                }
            }
            '<' => {
                if let Some(c) = self.source.peek() {
                    if *c == '=' {
                        self.tokens
                            .push(Token::new(LESSEQUAL, self.line, None, Object::None));
                        self.source.next();
                    } else {
                        self.tokens
                            .push(Token::new(LESS, self.line, None, Object::None));
                    }
                } else {
                    self.tokens
                        .push(Token::new(LESS, self.line, None, Object::None));
                }
            }
            '"' => {
                let mut buf = String::new();
                while self.source.peek() != None && *self.source.peek().unwrap() != '"' {
                    buf.push(*self.source.peek().unwrap());
                    self.source.next();
                }
                self.source.next();
                self.tokens.push(Token::new(
                    STRING,
                    self.line,
                    Some(buf.clone()),
                    Object::Str(buf),
                ));
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
                    NUMBER,
                    self.line,
                    Some(buf.clone()),
                    Object::Num(f64::from_str(&buf).unwrap()),
                ));
            }
            '_' | 'a'..='z' | 'A'..='Z' => {
                let mut buf = String::from(c);
                while self.source.peek() != None
                    && (*self.source.peek().unwrap()).is_ascii_alphanumeric()
                {
                    buf.push(*self.source.peek().unwrap());
                    self.source.next();
                }
                match KEYWORDS.get(buf.as_str()) {
                    Some(k) => {
                        self.tokens
                            .push(Token::new(k.clone(), self.line, None, Object::None))
                    }
                    None => self
                        .tokens
                        .push(Token::new(IDENTIFIER, self.line, None, Object::None)),
                }
            }
            ' ' | '\t' | 'r' => (),
            '\n' => self.line += 1,
            _ => self
                .tokens
                .push(Token::new(TokenType::EOF, self.line, None, Object::None)),
        }
    }
    fn is_at_end(&mut self) -> bool {
        self.source.peek() == None
    }
}
