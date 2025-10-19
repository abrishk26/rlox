use crate::types::{Object, Token, TokenType};
use std::{cell::RefCell, rc::Rc};
use std::{collections::HashMap, fmt};

#[derive(Clone)]
pub struct Environment {
    map: HashMap<String, Object>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    fn new(enclosing: Option<Box<Environment>>) -> Self {
        Self {
            map: HashMap::new(),
            enclosing,
        }
    }

    fn get(&self, key: &String) -> Option<Object> {
        match self.enclosing {
            Some(ref e) => match self.map.get(key) {
                Some(o) => Some(o.clone()),
                _ => e.get(key),
            },
            _ => match self.map.get(key) {
                Some(o) => Some(o.clone()),
                _ => None,
            },
        }
    }

    fn set(&mut self, key: String, value: Object) {
        self.map.insert(key, value);
    }
}

pub struct Interpreter {
    env: Environment,
    stmts: Vec<Stmt>,
}

impl Interpreter {
    pub fn new(env: Option<Box<Environment>>, stmts: Vec<Stmt>) -> Self {
        Self {
            env: Environment::new(None),
            stmts,
        }
    }

    pub fn parse(&mut self) {
        for stmt in self.stmts.iter() {
            stmt.exec(&mut self.env);
        }
    }
}

pub trait Eval {
    fn eval(&self, _: &mut Environment) -> Object;
}

pub trait Exec {
    fn exec(&self, _: &mut Environment) {}
}

#[derive(Debug)]
pub enum Stmt {
    Print(Expr),
    VarStmt(VarStmt),
    ExprStmt(Expr),
    BlockStmt(BlockStmt),
}

impl Exec for Stmt {
    fn exec(&self, env: &mut Environment) {
        match self {
            Stmt::Print(e) => println!("{}", e.eval(env)),
            Stmt::VarStmt(v) => v.exec(env),
            Stmt::ExprStmt(a) => a.exec(env),
            Stmt::BlockStmt(b) => b.exec(env),
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Lit(Literal),
    Group(Box<Grouping>),
    Unary(Box<Unary>),
    Binary(Box<Binary>),
    Var(VarExpr),
    Assign(Box<Assign>),
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
            Self::Var(v) => write!(f, "({})", v.name),
            Self::Assign(a) => write!(f, "({} = {})", a.name, a.value),
        }
    }
}

impl Eval for Expr {
    fn eval(&self, env: &mut Environment) -> Object {
        match self {
            Self::Lit(l) => l.value.clone(),
            Self::Group(g) => g.expr.eval(env),
            Self::Unary(u) => match u.operator {
                TokenType::MINUS => match u.right.eval(env) {
                    Object::Num(n) => Object::Num(-n),
                    _ => Object::None,
                },
                _ => Object::None,
            },
            Self::Binary(b) => b.eval(env),
            Self::Var(v) => v.eval(env),
            Self::Assign(a) => a.eval(env),
        }
    }
}

impl Exec for Expr {
    fn exec(&self, env: &mut Environment) {
        match self {
            Expr::Assign(a) => a.exec(env),
            _ => unreachable!(),
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
    fn eval(&self, _: &mut Environment) -> Object {
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
struct Assign {
    name: String,
    value: Expr,
}

impl Eval for Assign {
    fn eval(&self, env: &mut Environment) -> Object {
        self.exec(env);
        self.value.eval(env)
    }
}

impl Exec for Assign {
    fn exec(&self, env: &mut Environment) {
        match env.get(&self.name) {
            Some(_) => {
                let value = self.value.eval(env);
                env.set(self.name.clone(), value);
            }
            _ => (),
        }
    }
}

#[derive(Debug)]
struct VarExpr {
    name: String,
}

impl Eval for VarExpr {
    fn eval(&self, env: &mut Environment) -> Object {
        env.get(&self.name).unwrap_or(Object::None).clone()
    }
}

#[derive(Debug)]
struct VarStmt {
    name: String,
    value: Option<Expr>,
}

impl Exec for VarStmt {
    fn exec(&self, env: &mut Environment) {
        let value = match self.value {
            Some(ref e) => e.eval(env),
            _ => Object::None,
        };
        env.set(self.name.clone(), value.clone());
    }
}

#[derive(Debug)]
struct BlockStmt {
    stmts: Vec<Stmt>,
}

impl Exec for BlockStmt {
    fn exec(&self, env: &mut Environment) {
        let mut e = Environment::new(Some(Box::new(env.clone())));
        for stmt in self.stmts.iter() {
            stmt.exec(&mut e);
        }
    }
}

impl Eval for Binary {
    fn eval(&self, env: &mut Environment) -> Object {
        match self.operator {
            TokenType::PLUS => {
                let left = self.left.eval(env);
                let right = self.right.eval(env);
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
                let left = self.left.eval(env);
                let right = self.right.eval(env);
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
                let left = self.left.eval(env);
                let right = self.right.eval(env);
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
                let left = self.left.eval(env);
                let right = self.right.eval(env);
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
                let left = self.left.eval(env);
                let right = self.right.eval(env);
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
                let left = self.left.eval(env);
                let right = self.right.eval(env);
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
                let left = self.left.eval(env);
                let right = self.right.eval(env);
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
                let left = self.left.eval(env);
                let right = self.right.eval(env);
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
                let left = self.left.eval(env);
                let right = self.right.eval(env);
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
                let left = self.left.eval(env);
                let right = self.right.eval(env);
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
                TokenType::LEFTBRACE => {
                    self.current += 1;
                    return self.block_statement();
                }
                TokenType::VAR => {
                    self.current += 1;
                    return self.var_statement();
                }
                _ => {
                    println!("other statemetn being callled");
                    return self.expression_statement();
                }
            }
        }

        None
    }

    fn block_statement(&mut self) -> Option<Stmt> {
        let mut stmts = Vec::new();

        while self.current < self.tokens.len()
            && self.tokens[self.current].token_type != TokenType::RIGHTBRACE
        {
            let stmt = self.statement()?;
            stmts.push(stmt);
        }

        self.current += 1;

        Some(Stmt::BlockStmt(BlockStmt { stmts }))
    }

    fn var_statement(&mut self) -> Option<Stmt> {
        if self.tokens[self.current].token_type != TokenType::IDENTIFIER {
            return None;
        }

        let name = self.tokens[self.current].clone();
        self.current += 1;

        let mut value: Option<Expr> = None;
        if self.tokens[self.current].token_type == TokenType::EQUAL {
            self.current += 1;
            value = Some(self.expr()?);
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
        println!("expression on other statemetn ({})", expr);

        if self.tokens[self.current].token_type != TokenType::SEMICOLON {
            return None;
        }

        self.current += 1;

        Some(Stmt::ExprStmt(expr))
    }

    fn expr(&mut self) -> Option<Expr> {
        self.assignment()
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
                    return Some(Expr::Var(VarExpr {
                        name: token.lexeme.clone().unwrap(),
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

    fn assignment(&mut self) -> Option<Expr> {
        let expr = self.equality()?;
        println!("expression in assignment ({})", expr);

        if self.tokens[self.current].token_type == TokenType::EQUAL {
            self.current += 1;
            let value = self.assignment()?;

            match expr {
                Expr::Var(v) => {
                    return Some(Expr::Assign(Box::new(Assign {
                        name: v.name.clone(),
                        value: value,
                    })));
                }
                _ => return None,
            }
        }

        Some(expr)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.tokens[self.current].token_type == TokenType::EOF
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }
}
