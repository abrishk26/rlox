use crate::scanner::{Object, Token};


type ExprID = usize;

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

impl Expr {
    pub fn get_id(&self) -> usize {
        match self {
            Self::Literal(l) => l.id,
            Self::Binary(b) => b.id,
            Self::Unary(u) => u.id,
            Self::Grouping(g) => g.id,
            Self::Var(v) => v.id,
            Self::Assign(a) => a.id,
            Self::Logical(l) => l.id,
            Self::Call(c) => c.id,
        }
    }
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
pub struct Literal {
    pub id: ExprID,
    pub value: Object,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Binary {
    pub id: ExprID,
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Logical {
    pub id: ExprID,
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Unary {
    pub id: ExprID,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Grouping {
    pub id: ExprID,
    pub expr: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    pub id: ExprID,
    pub name: Token,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assign {
    pub id: ExprID,
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub id: ExprID,
    pub calle: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}
