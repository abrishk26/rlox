use crate::types::{Object, Token, TokenType};
use std::{cell::RefCell, rc::Rc};
use std::{collections::HashMap, fmt};

#[derive(Clone)]
pub struct Environment {
    map: HashMap<String, Object>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            map: HashMap::new(),
            enclosing,
        }
    }

    fn get(&self, key: &String) -> Option<Object> {
        if let Some(o) = self.map.get(key) {
            Some(o.clone())
        } else if let Some(ref e) = self.enclosing {
            e.borrow().get(key)
        } else {
            None
        }
    }

    fn assign(&mut self, key: String, value: Object) {
        if let Some(_) = self.map.get(&key) {
            self.map.insert(key, value);
        } else if let Some(ref e) = self.enclosing {
            e.borrow_mut().assign(key, value)
        }
    }

    fn set(&mut self, key: String, value: Object) {
        self.map.insert(key, value);
    }
}

pub struct Interpreter {
    env: Rc<RefCell<Environment>>,
    stmts: Vec<Stmt>,
}

impl Interpreter {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Self {
            env: Rc::new(RefCell::new(Environment::new(None))),
            stmts,
        }
    }

    pub fn parse(&mut self) {
        for stmt in self.stmts.iter() {
            stmt.exec(self.env.clone());
        }
    }
}

pub trait Eval {
    fn eval(&self, _: Rc<RefCell<Environment>>) -> Object;
}

pub trait Exec {
    fn exec(&self, _: Rc<RefCell<Environment>>) {}
}

#[derive(Debug)]
pub enum Stmt {
    Print(Expr),
    VarStmt(VarStmt),
    ExprStmt(Expr),
    If(Box<IfStmt>),
    While(Box<WhileStmt>),
    BlockStmt(BlockStmt),
}

impl Exec for Stmt {
    fn exec(&self, env: Rc<RefCell<Environment>>) {
        match self {
            Stmt::Print(e) => println!("{}", e.eval(env)),
            Stmt::VarStmt(v) => v.exec(env),
            Stmt::ExprStmt(a) => a.exec(env),
            Stmt::If(i) => i.exec(env),
            Stmt::While(w) => w.exec(env),
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
                TokenType::OR => write!(f, "({} or {})", b.left, b.right),
                TokenType::AND => write!(f, "({} and {})", b.left, b.right),
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
    fn eval(&self, env: Rc<RefCell<Environment>>) -> Object {
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
    fn exec(&self, env: Rc<RefCell<Environment>>) {
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
pub struct Literal {
    value: Object,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Eval for Literal {
    fn eval(&self, _: Rc<RefCell<Environment>>) -> Object {
        self.value.clone()
    }
}

#[derive(Debug)]
pub struct Grouping {
    expr: Expr,
}

#[derive(Debug)]
pub struct WhileStmt {
    condition: Expr,
    block: Stmt,
}

fn is_truthy(expr: Object) -> bool {
    match expr {
        Object::Bool(b) => b,
        Object::None => false,
        _ => true,
    }
}

impl Exec for WhileStmt {
    fn exec(&self, env: Rc<RefCell<Environment>>) {
        while is_truthy(self.condition.eval(env.clone())) {
            self.block.exec(env.clone());
        }
    }
}

#[derive(Debug)]
pub struct IfStmt {
    condition: Expr,
    then_block: Stmt,
    else_block: Option<Stmt>,
}

impl Exec for IfStmt {
    fn exec(&self, env: Rc<RefCell<Environment>>) {
        match self.condition.eval(env.clone()) {
            Object::Bool(b) => {
                if b {
                    self.then_block.exec(env.clone());
                } else {
                    match self.else_block {
                        Some(ref s) => s.exec(env.clone()),
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
}

#[derive(Debug)]
pub struct Unary {
    operator: TokenType,
    right: Expr,
}

#[derive(Debug)]
pub struct Binary {
    left: Expr,
    operator: TokenType,
    right: Expr,
}

#[derive(Debug)]
pub struct Assign {
    name: String,
    value: Expr,
}

impl Eval for Assign {
    fn eval(&self, env: Rc<RefCell<Environment>>) -> Object {
        self.exec(env.clone());
        self.value.eval(env.clone())
    }
}

impl Exec for Assign {
    fn exec(&self, env: Rc<RefCell<Environment>>) {
        let value = self.value.eval(env.clone());
        env.borrow_mut().assign(self.name.clone(), value);
    }
}

#[derive(Debug)]
pub struct VarExpr {
    name: String,
}

impl Eval for VarExpr {
    fn eval(&self, env: Rc<RefCell<Environment>>) -> Object {
        env.borrow_mut()
            .get(&self.name)
            .unwrap_or(Object::None)
            .clone()
    }
}

#[derive(Debug)]
pub struct VarStmt {
    name: String,
    value: Option<Expr>,
}

impl Exec for VarStmt {
    fn exec(&self, env: Rc<RefCell<Environment>>) {
        let value = match self.value {
            Some(ref e) => e.eval(env.clone()),
            _ => Object::None,
        };
        env.borrow_mut().set(self.name.clone(), value.clone());
    }
}

#[derive(Debug)]
pub struct BlockStmt {
    stmts: Vec<Stmt>,
}

impl Exec for BlockStmt {
    fn exec(&self, env: Rc<RefCell<Environment>>) {
        for stmt in self.stmts.iter() {
            stmt.exec(env.clone());
        }
    }
}

impl Eval for Binary {
    fn eval(&self, env: Rc<RefCell<Environment>>) -> Object {
        match self.operator {
            TokenType::PLUS => {
                let left = self.left.eval(env.clone());
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
                let left = self.left.eval(env.clone());
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
                let left = self.left.eval(env.clone());
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
                let left = self.left.eval(env.clone());
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
                let left = self.left.eval(env.clone());
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
                let left = self.left.eval(env.clone());
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
                let left = self.left.eval(env.clone());
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
                let left = self.left.eval(env.clone());
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
                let left = self.left.eval(env.clone());
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
                let left = self.left.eval(env.clone());
                let right = self.right.eval(env);
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
            TokenType::OR => {
                let left = self.left.eval(env.clone());
                let right = self.right.eval(env);
                match left {
                    Object::Bool(l) => {
                        if l {
                            return left;
                        }
                        match right {
                            Object::Bool(_) => {
                                return right;
                            }
                            _ => Object::None,
                        }
                    }
                    _ => Object::None,
                }
            }
            TokenType::AND => {
                let left = self.left.eval(env.clone());
                let right = self.right.eval(env);
                match left {
                    Object::Bool(l) => {
                        if !l {
                            return left;
                        }
                        match right {
                            Object::Bool(_) => {
                                return right;
                            }
                            _ => Object::None,
                        }
                    }
                    _ => Object::None,
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
                TokenType::IF => {
                    self.current += 1;
                    return self.if_statement();
                }
                TokenType::FOR => {
                    self.current += 1;
                    return self.for_statement();
                }
                TokenType::WHILE => {
                    self.current += 1;
                    return self.while_statement();
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
                    return self.expression_statement();
                }
            }
        }

        None
    }

    fn for_statement(&mut self) -> Option<Stmt> {
        if self.tokens[self.current].token_type != TokenType::LEFTPAREN {
            return None;
        }

        self.current += 1;

        let initializer;
        if self.tokens[self.current].token_type == TokenType::SEMICOLON {
            initializer = None;
        } else if self.tokens[self.current].token_type == TokenType::VAR {
            self.current += 1;
            initializer = Some(self.var_statement()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition = None;
        if self.tokens[self.current].token_type != TokenType::SEMICOLON {
            condition = Some(self.expr()?);
        }

        if self.tokens[self.current].token_type != TokenType::SEMICOLON {
            return None;
        }

        self.current += 1;

        let mut increment = None;
        if self.tokens[self.current].token_type != TokenType::RIGHTPAREN {
            increment = Some(self.expr()?);
        }

        if self.tokens[self.current].token_type != TokenType::RIGHTPAREN {
            return None;
        }

        self.current += 1;

        let mut body = self.statement()?;

        if let Some(i) = increment {
            body = Stmt::BlockStmt(BlockStmt {
                stmts: vec![body, Stmt::ExprStmt(i)],
            });
        }

        match condition {
            Some(_) => (),
            _ => {
                condition = Some(Expr::Lit(Literal {
                    value: Object::Bool(true),
                }));
            }
        }

        body = Stmt::While(Box::new(WhileStmt {
            condition: condition.unwrap(),
            block: body,
        }));

        if let Some(i) = initializer {
            body = Stmt::BlockStmt(BlockStmt {
                stmts: vec![i, body],
            });
        }

        Some(body)
    }
    fn while_statement(&mut self) -> Option<Stmt> {
        if self.tokens[self.current].token_type != TokenType::LEFTPAREN {
            return None;
        }

        self.current += 1;
        let condition = self.expr()?;

        if self.tokens[self.current].token_type != TokenType::RIGHTPAREN {
            return None;
        }

        self.current += 1;

        let block = self.statement()?;

        Some(Stmt::While(Box::new(WhileStmt { condition, block })))
    }

    fn if_statement(&mut self) -> Option<Stmt> {
        if self.tokens[self.current].token_type != TokenType::LEFTPAREN {
            return None;
        }

        self.current += 1;
        let condition = self.expr()?;

        if self.tokens[self.current].token_type != TokenType::RIGHTPAREN {
            return None;
        }

        self.current += 1;

        let then_block = self.statement()?;
        let mut else_block = None;
        if self.current < self.tokens.len()
            && self.tokens[self.current].token_type == TokenType::ELSE
        {
            self.current += 1;
            else_block = Some(self.statement()?);
        }

        Some(Stmt::If(Box::new(IfStmt {
            condition,
            then_block,
            else_block,
        })))
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
                    return Some(Expr::Lit(Literal {
                        value: token.literal.clone(),
                    }));
                }
                TokenType::IDENTIFIER => {
                    self.current += 1;
                    return Some(Expr::Var(VarExpr {
                        name: token.lexeme.clone().unwrap(),
                    }));
                }
                TokenType::TRUE => {
                    self.current += 1;
                    return Some(Expr::Lit(Literal {
                        value: Object::Bool(true),
                    }));
                }

                TokenType::FALSE => {
                    self.current += 1;
                    return Some(Expr::Lit(Literal {
                        value: Object::Bool(false),
                    }));
                }

                TokenType::NIL => {
                    self.current += 1;
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
                    return None;
                }
            }
        } else {
            None
        }
    }

    fn unary(&mut self) -> Option<Expr> {
        let token = self.tokens[self.current].clone();
        if !self.is_at_end()
            && (token.token_type == TokenType::BANG || token.token_type == TokenType::MINUS)
        {
            self.current += 1;
            let right = self.unary()?;
            return Some(Expr::Unary(Box::new(Unary {
                operator: token.token_type.clone(),
                right,
            })));
        }

        self.primary()
    }

    fn factor(&mut self) -> Option<Expr> {
        let mut left = self.unary()?;
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
        let mut left = self.factor()?;
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
        let mut left = self.term()?;
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
        let mut left = self.comparision()?;
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

    fn and(&mut self) -> Option<Expr> {
        let mut left = self.equality()?;

        while let Some(token) = self.peek() {
            match token.token_type {
                TokenType::AND => {
                    let operator = self.tokens[self.current].token_type.clone();
                    self.current += 1;
                    let right = self.equality()?;
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

    fn or(&mut self) -> Option<Expr> {
        let mut left = self.and()?;

        while let Some(token) = self.peek() {
            match token.token_type {
                TokenType::OR => {
                    let operator = self.tokens[self.current].token_type.clone();
                    self.current += 1;
                    let right = self.and()?;
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
        let expr = self.or()?;

        if self.tokens[self.current].token_type == TokenType::EQUAL {
            self.current += 1;
            let value = self.or()?;

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
