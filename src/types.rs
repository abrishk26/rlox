use crate::parser::{Function, NativeFunc};
use TokenType::*;
use std::{collections::HashMap, fmt, sync::LazyLock};

pub static KEYWORDS: LazyLock<HashMap<&'static str, TokenType>> = LazyLock::new(|| {
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
    Func(Function),
    NativeFunc(NativeFunc),
    None,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(n) => write!(f, "{}", n),
            Self::Str(s) => write!(f, "{}", s),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Func(_) => write!(f, "function call"),
            Self::NativeFunc(_) => write!(f, "native fn"),
            Self::None => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: u64,
    pub lexeme: Option<String>,
    pub literal: Object,
}

impl Token {
    pub fn new(token_type: TokenType, line: u64, lexeme: Option<String>, literal: Object) -> Token {
        Token {
            token_type,
            line,
            lexeme,
            literal,
        }
    }
}
