use crate::types::{Object, Token, TokenType};
use std::{collections::HashMap, fmt};

pub struct Interpreter {
    env: HashMap<String, Object>,
    stmts: Vec<Stmt>,
}

impl Interpreter {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Self {
            env: HashMap::new(),
            stmts,
        }
    }

    pub fn parse(&mut self) {
        for stmt in self.stmts.iter() {
            stmt.exec(Some(&mut self.env));
        }
    }
}

pub trait Eval {
    fn eval(&self, _: Option<&mut HashMap<String, Object>>) -> Object;
}

pub trait Exec {
    fn exec(&self, _: Option<&mut HashMap<String, Object>>) {}
}

#[derive(Debug)]
pub enum Stmt {
    Print(Expr),
    VarStmt(VarStmt),
    ExprStmt(Expr),
}

impl Exec for Stmt {
    fn exec(&self, map: Option<&mut HashMap<String, Object>>) {
        match self {
            Stmt::Print(e) => println!("{}", e.eval(map)),
            Stmt::VarStmt(v) => v.exec(map),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Lit(Literal),
    Group(Box<Grouping>),
    Unary(Box<Unary>),
    Binary(Box<Binary>),
    Var(VarStmt),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lit(l) => write!(f, "{}", l),
            Self::Binary(b) => match b.operator {
                TokenType::PLUS => write!(f, "({} + {})", b.left, b.right),
                TokenType::MINUS => write!(f, "({} - {})", b.left, b.right),
                TokenType::SLASH => write!(f, "({} / {})", b.left, b.right),
                TokenType::STAR => write!(f, "({} * {})", b.left, b.right),
                TokenType::LESS => write!(f, "({} < {})", b.left, b.right),
                TokenType::LESSEQUAL => write!(f, "({} <= {})", b.left, b.right),
                TokenType::GREATER => write!(f, "({} > {})", b.left, b.right),
                TokenType::GREATEREQUAL => write!(f, "({} >= {})", b.left, b.right),
                TokenType::EQUAL => write!(f, "({} = {})", b.left, b.right),
                TokenType::EQUALEQUAL => write!(f, "({} == {})", b.left, b.right),
                TokenType::BANGEQUAL => write!(f, "({} != {})", b.left, b.right),
                _ => unreachable!(),
            },
            Self::Unary(u) => match u.operator {
                TokenType::BANG => write!(f, "(!{})", u.right),
                TokenType::MINUS => write!(f, "(-{})", u.right),
                _ => unreachable!(),
            },
            Self::Group(g) => write!(f, "({})", g.expr),
            Self::Var(v) => write!(f, "({})", v.value),
        }
    }
}

impl Eval for Expr {
    fn eval(&self, map: Option<&mut HashMap<String, Object>>) -> Object {
        match self {
            Self::Lit(l) => l.value.clone(),
            Self::Group(g) => g.expr.eval(None),
            Self::Unary(u) => match u.operator {
                TokenType::MINUS => match u.right.eval(None) {
                    Object::Num(n) => Object::Num(-n),
                    _ => Object::None,
                },
                _ => Object::None,
            },
            Self::Binary(b) => b.eval(None),
            Self::Var(v) => v.eval(map),
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug)]
struct Literal {
    value: Object,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Eval for Literal {
    fn eval(&self, _: Option<&mut HashMap<String, Object>>) -> Object {
        self.value.clone()
    }
}

#[derive(Debug)]
struct Grouping {
    expr: Expr,
}

#[derive(Debug)]
struct Unary {
    operator: TokenType,
    right: Expr,
}

#[derive(Debug)]
struct Binary {
    left: Expr,
    operator: TokenType,
    right: Expr,
}

#[derive(Debug)]
struct VarStmt {
    name: String,
    value: Object,
}

impl Exec for VarStmt {
    fn exec(&self, map: Option<&mut HashMap<String, Object>>) {
        map.unwrap().insert(self.name.clone(), self.value.clone());
    }
}

impl Eval for VarStmt {
    fn eval(&self, map: Option<&mut HashMap<String, Object>>) -> Object {
        map.unwrap()
            .get(&self.name)
            .unwrap_or(&Object::None)
            .clone()
    }
}

impl Eval for Binary {
    fn eval(&self, _: Option<&mut HashMap<String, Object>>) -> Object {
        match self.operator {
            TokenType::PLUS => {
                let left = self.left.eval(None);
                let right = self.right.eval(None);
                match left {
                    Object::Num(l) => match right {
                        Object::Num(r) => {
                            return Object::Num(l + r);
                        }
                        _ => Object::None,
                    },
                    Object::Str(s1) => match right {
                        Object::Str(s2) => {
                            let mut res = s1.clone();
                            res.extend(s2.chars());
                            return Object::Str(res);
                        }
                        _ => Object::None,
                    },
                    _ => Object::None,
                }
            }
            TokenType::MINUS => {
                let left = self.left.eval(None);
                let right = self.right.eval(None);
                match left {
                    Object::Num(l) => match right {
                        Object::Num(r) => {
                            return Object::Num(l - r);
                        }
                        _ => Object::None,
                    },
                    _ => Object::None,
                }
            }
            TokenType::STAR => {
                let left = self.left.eval(None);
                let right = self.right.eval(None);
                match left {
                    Object::Num(l) => match right {
                        Object::Num(r) => {
                            return Object::Num(l * r);
                        }
                        _ => Object::None,
                    },
                    _ => Object::None,
                }
            }
            TokenType::SLASH => {
                let left = self.left.eval(None);
                let right = self.right.eval(None);
                match left {
                    Object::Num(l) => match right {
                        Object::Num(r) => {
                            return Object::Num(l / r);
                        }
                        _ => Object::None,
                    },
                    _ => Object::None,
                }
            }
            TokenType::GREATER => {
                let left = self.left.eval(None);
                let right = self.right.eval(None);
                match left {
                    Object::Num(l) => match right {
                        Object::Num(r) => {
                            return Object::Bool(l > r);
                        }
                        _ => Object::None,
                    },
                    _ => Object::None,
                }
            }
            TokenType::GREATEREQUAL => {
                let left = self.left.eval(None);
                let right = self.right.eval(None);
                match left {
                    Object::Num(l) => match right {
                        Object::Num(r) => {
                            return Object::Bool(l >= r);
                        }
                        _ => Object::None,
                    },
                    _ => Object::None,
                }
            }
            TokenType::LESS => {
                let left = self.left.eval(None);
                let right = self.right.eval(None);
                match left {
                    Object::Num(l) => match right {
                        Object::Num(r) => {
                            return Object::Bool(l < r);
                        }
                        _ => Object::None,
                    },
                    _ => Object::None,
                }
            }
            TokenType::LESSEQUAL => {
                let left = self.left.eval(None);
                let right = self.right.eval(None);
                match left {
                    Object::Num(l) => match right {
                        Object::Num(r) => {
                            return Object::Bool(l <= r);
                        }
                        _ => Object::None,
                    },
                    _ => Object::None,
                }
            }
            TokenType::EQUALEQUAL => {
                let left = self.left.eval(None);
                let right = self.right.eval(None);
                println!("left and right on eval: ({}), ({})", self.left, self.right);
                match left {
                    Object::Num(l) => match right {
                        Object::Num(r) => {
                            return Object::Bool(l == r);
                        }
                        _ => Object::None,
                    },
                    Object::Str(l) => match right {
                        Object::Str(r) => {
                            return Object::Bool(l.eq(&r));
                        }
                        _ => Object::None,
                    },
                    Object::Bool(l) => match right {
                        Object::Bool(r) => {
                            return Object::Bool(l.eq(&r));
                        }
                        _ => Object::None,
                    },
                    Object::None => match right {
                        Object::None => return Object::Bool(true),
                        _ => return Object::Bool(false),
                    },
                }
            }
            TokenType::BANGEQUAL => {
                let left = self.left.eval(None);
                let right = self.right.eval(None);
                println!("left and right on eval: ({}), ({})", self.left, self.right);
                match left {
                    Object::Num(l) => match right {
                        Object::Num(r) => {
                            return Object::Bool(l != r);
                        }
                        _ => Object::None,
                    },
                    Object::Str(l) => match right {
                        Object::Str(r) => {
                            return Object::Bool(!l.eq(&r));
                        }
                        _ => Object::None,
                    },
                    Object::Bool(l) => match right {
                        Object::Bool(r) => {
                            return Object::Bool(!l.eq(&r));
                        }
                        _ => Object::None,
                    },
                    Object::None => match right {
                        Object::None => return Object::Bool(false),
                        _ => return Object::Bool(true),
                    },
                }
            }

            _ => Object::None,
        }
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Vec<Stmt>> {
        let mut stmts = Vec::new();

        while self.current < self.tokens.len()
            && self.tokens[self.current].token_type != TokenType::EOF
        {
            let stmt = self.statement()?;
            stmts.push(stmt);
        }

        Some(stmts)
    }

    fn statement(&mut self) -> Option<Stmt> {
        if let Some(token) = self.peek() {
            match token.token_type {
                TokenType::PRINT => {
                    self.current += 1;
                    return self.print_statement();
                }
                TokenType::VAR => {
                    self.current += 1;
                    return self.var_statement();
                }
                _ => {
                    self.current += 1;
                    return self.expression_statement();
                }
            }
        }

        None
    }

    fn var_statement(&mut self) -> Option<Stmt> {
        if self.tokens[self.current].token_type != TokenType::IDENTIFIER {
            return None;
        }

        let name = self.tokens[self.current].clone();
        self.current += 1;

        let mut value = Object::None;
        if self.tokens[self.current].token_type == TokenType::EQUAL {
            self.current += 1;
            value = self.expr()?.eval(None);
        }

        if self.tokens[self.current].token_type != TokenType::SEMICOLON {
            return None;
        }

        self.current += 1;

        Some(Stmt::VarStmt(VarStmt {
            name: name.lexeme.unwrap(),
            value,
        }))
    }

    fn print_statement(&mut self) -> Option<Stmt> {
        let expr = self.expr()?;

        if self.tokens[self.current].token_type != TokenType::SEMICOLON {
            return None;
        }

        self.current += 1;

        Some(Stmt::Print(expr))
    }

    fn expression_statement(&mut self) -> Option<Stmt> {
        let expr = self.expr()?;

        if self.tokens[self.current].token_type != TokenType::SEMICOLON {
            return None;
        }

        self.current += 1;

        Some(Stmt::ExprStmt(expr))
    }

    fn expr(&mut self) -> Option<Expr> {
        self.equality()
    }

    fn primary(&mut self) -> Option<Expr> {
        if !self.is_at_end() {
            let token = &self.tokens[self.current];
            match token.token_type {
                TokenType::NUMBER | TokenType::STRING => {
                    self.current += 1;
                    println!(
                        "returning from primary - token type ({:?})",
                        token.token_type
                    );
                    return Some(Expr::Lit(Literal {
                        value: token.literal.clone(),
                    }));
                }
                TokenType::IDENTIFIER => {
                    self.current += 1;
                    println!(
                        "returning from primary - token type ({:?})",
                        token.token_type
                    );
                    return Some(Expr::Var(VarStmt {
                        name: token.lexeme.clone().unwrap(),
                        value: token.literal.clone(),
                    }));
                }
                TokenType::TRUE => {
                    self.current += 1;
                    println!(
                        "returning from primary - token type ({:?})",
                        token.token_type
                    );
                    return Some(Expr::Lit(Literal {
                        value: Object::Bool(true),
                    }));
                }

                TokenType::FALSE => {
                    self.current += 1;
                    println!(
                        "returning from primary - token type ({:?})",
                        token.token_type
                    );
                    return Some(Expr::Lit(Literal {
                        value: Object::Bool(false),
                    }));
                }

                TokenType::NIL => {
                    self.current += 1;
                    println!(
                        "returning from primary - token type ({:?})",
                        token.token_type
                    );
                    return Some(Expr::Lit(Literal {
                        value: Object::None,
                    }));
                }

                TokenType::LEFTPAREN => {
                    self.current += 1;
                    let expr = self.expr()?;
                    match self.tokens[self.current].token_type {
                        TokenType::RIGHTPAREN => {
                            self.current += 1;
                            return Some(Expr::Group(Box::new(Grouping { expr })));
                        }
                        _ => return None,
                    }
                }
                _ => {
                    println!(
                        "failing on the first call, token type ({:?})",
                        token.token_type
                    );
                    return None;
                }
            }
        } else {
            None
        }
    }

    fn unary(&mut self) -> Option<Expr> {
        let token = self.tokens[self.current].clone();
        println!("token type on unary ({:?})", token.token_type);
        if !self.is_at_end()
            && (token.token_type == TokenType::BANG || token.token_type == TokenType::MINUS)
        {
            self.current += 1;
            let right = self.unary()?;
            println!("unary right ({})", right);
            return Some(Expr::Unary(Box::new(Unary {
                operator: token.token_type.clone(),
                right,
            })));
        }

        println!("returning from unary");
        self.primary()
    }

    fn factor(&mut self) -> Option<Expr> {
        println!("factor is called");
        let mut left = self.unary()?;
        println!("left on factor ( {} )", left);
        while let Some(token) = self.peek() {
            match token.token_type {
                TokenType::STAR | TokenType::SLASH => {
                    let operator = self.tokens[self.current].token_type.clone();
                    self.current += 1;
                    let right = self.unary()?;
                    left = Expr::Binary(Box::new(Binary {
                        left,
                        operator,
                        right,
                    }));
                }
                _ => break,
            }
        }

        Some(left)
    }

    fn term(&mut self) -> Option<Expr> {
        println!("term is called");
        let mut left = self.factor()?;
        println!("left on term ( {} )", left);
        while let Some(token) = self.peek() {
            match token.token_type {
                TokenType::PLUS | TokenType::MINUS => {
                    let operator = self.tokens[self.current].token_type.clone();
                    self.current += 1;
                    let right = self.factor()?;
                    left = Expr::Binary(Box::new(Binary {
                        left,
                        operator,
                        right,
                    }));
                }
                _ => break,
            }
        }

        Some(left)
    }

    fn comparision(&mut self) -> Option<Expr> {
        println!("comparision is called");
        let mut left = self.term()?;
        println!("left on comparision ( {} )", left);
        while let Some(token) = self.peek() {
            match token.token_type {
                TokenType::GREATER
                | TokenType::LESS
                | TokenType::GREATEREQUAL
                | TokenType::LESSEQUAL => {
                    let operator = self.tokens[self.current].token_type.clone();
                    self.current += 1;
                    let right = self.term()?;
                    left = Expr::Binary(Box::new(Binary {
                        left,
                        operator,
                        right,
                    }));
                }
                _ => break,
            }
        }

        Some(left)
    }

    fn equality(&mut self) -> Option<Expr> {
        println!("equality is called");
        let mut left = self.comparision()?;
        println!("left on equality ( {} )", left);
        while let Some(token) = self.peek() {
            match token.token_type {
                TokenType::EQUALEQUAL | TokenType::BANGEQUAL => {
                    let operator = self.tokens[self.current].token_type.clone();
                    self.current += 1;
                    let right = self.comparision()?;
                    left = Expr::Binary(Box::new(Binary {
                        left,
                        operator,
                        right,
                    }));
                }
                _ => break,
            }
        }

        Some(left)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.tokens[self.current].token_type == TokenType::EOF
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }
}
