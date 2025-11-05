use crate::expressions::Expr;
use crate::scanner::Token;

pub trait VisitorS<T> {
    fn visit_expr_stmt(&mut self, stmt: &mut Expr) -> T;
    fn visit_var_stmt(&mut self, stmt: &mut Var) -> T;
    fn visit_block_stmt(&mut self, stmt: &mut Block) -> T;
    fn visit_if_stmt(&mut self, stmt: &mut IfStmt) -> T;
    fn visit_while_stmt(&mut self, stmt: &mut WhileStmt) -> T;
    fn visit_func_stmt(&mut self, stmt: &mut Func) -> T;
    fn visit_return_stmt(&mut self, stmt: &mut ReturnStmt) -> T;
    fn visit_class_stmt(&mut self, stmt: &mut Class) -> T;
}

pub trait VisitableS<T> {
    fn accept(&mut self, visitor: &mut impl VisitorS<T>) -> T;
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    ExprStmt(Expr),
    Var(Var),
    Block(Block),
    If(IfStmt),
    While(WhileStmt),
    Func(Func),
    Class(Class),
    Return(ReturnStmt),
}

impl<T> VisitableS<T> for Stmt {
    fn accept(&mut self, visitor: &mut impl VisitorS<T>) -> T {
        match self {
            Self::ExprStmt(e) => visitor.visit_expr_stmt(e),
            Self::Var(v) => visitor.visit_var_stmt(v),
            Self::Block(b) => visitor.visit_block_stmt(b),
            Self::If(i) => visitor.visit_if_stmt(i),
            Self::While(w) => visitor.visit_while_stmt(w),
            Self::Func(f) => visitor.visit_func_stmt(f),
            Self::Return(r) => visitor.visit_return_stmt(r),
            Self::Class(c) => visitor.visit_class_stmt(c),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    pub name: Token,
    pub methods: Vec<Func>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Func {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Var {
    pub token: Token,
    pub initializer: Option<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_block: Box<Stmt>,
    pub else_block: Option<Box<Stmt>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReturnStmt {
    pub keyword: Token,
    pub value: Option<Expr>,
}
