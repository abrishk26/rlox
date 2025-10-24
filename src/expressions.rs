use crate::interpreter::{Interpreter, RuntimeError};
use crate::scanner::{Object, Token};
use crate::statements::Stmt;

pub trait VisitorE<T> {
    fn visit_binary(&mut self, expr: &Binary) -> T;
    fn visit_unary(&mut self, expr: &Unary) -> T;
    fn visit_grouping(&mut self, expr: &Grouping) -> T;
    fn visit_literal(&mut self, expr: &Literal) -> T;
    fn visit_variable(&mut self, expr: &Variable) -> T;
    fn visit_assign(&mut self, expr: &Assign) -> T;
    fn visit_logical(&mut self, expr: &Logical) -> T;
    fn visit_call(&mut self, expr: &Call) -> T;
}

pub trait VisitableE<T> {
    fn accept(&mut self, visitor: &mut impl VisitorE<T>) -> T;
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Binary(Binary),
    Unary(Unary),
    Grouping(Grouping),
    Var(Variable),
    Assign(Assign),
    Logical(Logical),
    Call(Call),
}

impl<T> VisitableE<T> for Expr {
    fn accept(&mut self, visitor: &mut impl VisitorE<T>) -> T {
        match self {
            Self::Literal(l) => visitor.visit_literal(l),
            Self::Binary(b) => visitor.visit_binary(b),
            Self::Unary(u) => visitor.visit_unary(u),
            Self::Grouping(g) => visitor.visit_grouping(g),
            Self::Var(v) => visitor.visit_variable(v),
            Self::Assign(a) => visitor.visit_assign(a),
            Self::Logical(l) => visitor.visit_logical(l),
            Self::Call(c) => visitor.visit_call(c),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
    pub params: Vec<Token>,
}

impl Function {
    pub fn call(
        &mut self,
        interpreter: &mut Interpreter,
        mut arguments: Vec<Expr>,
    ) -> Result<Object, RuntimeError> {
        let env = interpreter.env.clone();
        for i in 0..self.params.len() {
            env.borrow_mut().set(
                self.params[i].clone().lexeme.unwrap(),
                interpreter.evaluate(&mut arguments[i])?,
            );
        }

        match interpreter.execute_block(&mut self.body, env)? {
            Some(v) => Ok(v),
            _ => Ok(Object::None),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Literal {
    pub value: Object,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Logical {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Grouping {
    pub expr: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    pub name: Token,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assign {
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub calle: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}
