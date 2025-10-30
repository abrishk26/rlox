use crate::expressions::{
    Assign, Binary, Call, Expr, Grouping, Literal, Logical, Unary, Variable, VisitableE, VisitorE,
};
use crate::interpreter::Interpreter;
use crate::scanner::Token;
use crate::statements::{
    Block, Func, IfStmt, Print, ReturnStmt, Stmt, Var, VisitableS, VisitorS, WhileStmt,
};
use std::collections::HashMap;

pub struct Resolver<'a> {
    scopes: Vec<HashMap<String, bool>>,
    interpreter: &'a mut Interpreter,
}

impl<'a> Resolver<'a> {
    pub fn new(scopes: Vec<HashMap<String, bool>>, interpreter: &'a mut Interpreter) -> Self {
        Self {
            scopes,
            interpreter,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }

        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.lexeme.unwrap(), false);
    }

    fn define(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }

        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.lexeme.unwrap(), true);
    }

    pub fn resolve_stmts(&mut self, stmts: &mut Vec<Stmt>) {
        for stmt in stmts.iter_mut() {
            self.resolve_stmt(stmt)
        }
    }

    fn resolve_stmt(&mut self, stmt: &mut Stmt) {
        stmt.accept(self);
    }

    fn resolve_expr(&mut self, expr: &mut Expr) {
        expr.accept(self);
    }

    fn resolve_fun(&mut self, func: &mut Func) {
        self.begin_scope();
        for param in &func.params {
            self.declare(param.clone());
            self.define(param.clone());
        }

        self.resolve_stmts(&mut func.body);
        self.end_scope();
    }

    fn resolve_local(&mut self, expr: Expr, name: Token) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&name.lexeme.clone().unwrap()) {
                self.interpreter.resolve(expr, self.scopes.len() - 1 + i);
                return;
            }
        }
    }
}

impl<'a> VisitorE<()> for Resolver<'a> {
    fn visit_binary(&mut self, expr: &Binary) {
        self.resolve_expr(&mut expr.left.clone());
        self.resolve_expr(&mut expr.right.clone());
    }

    fn visit_unary(&mut self, expr: &Unary) {
        self.resolve_expr(&mut expr.right.clone());
    }

    fn visit_grouping(&mut self, expr: &Grouping) {
        self.resolve_expr(&mut expr.expr.clone());
    }

    fn visit_literal(&mut self, _: &Literal) {}

    fn visit_variable(&mut self, expr: &Variable) {
        if !self.scopes.is_empty()
            && self.scopes[self.scopes.len() - 1].get(&expr.name.lexeme.clone().unwrap()) != None
            && *self.scopes[self.scopes.len() - 1]
                .get(&expr.name.lexeme.clone().unwrap())
                .unwrap()
                == false
        {
            eprintln!(
                "Can't read local variable in its own initializer. [Line: {}]",
                expr.name.line
            );
            return;
        }

        self.resolve_local(Expr::Var(expr.clone()), expr.name.clone());
    }

    fn visit_assign(&mut self, expr: &Assign) {
        let mut a = expr.clone();
        self.resolve_expr(&mut a.value);
        self.resolve_local(Expr::Assign(a.clone()), a.name.clone());
    }

    fn visit_logical(&mut self, expr: &Logical) {
        self.resolve_expr(&mut expr.left.clone());
        self.resolve_expr(&mut expr.right.clone());
    }

    fn visit_call(&mut self, expr: &Call) {
        self.resolve_expr(&mut expr.calle.clone());

        for arg in expr.arguments.clone().iter_mut() {
            self.resolve_expr(arg);
        }
    }
}

impl<'a> VisitorS<()> for Resolver<'a> {
    fn visit_block_stmt(&mut self, stmt: &mut Block) {
        self.begin_scope();
        self.resolve_stmts(&mut stmt.stmts);
        self.end_scope()
    }

    fn visit_print(&mut self, stmt: &mut Print) {
        self.resolve_expr(&mut stmt.expr);
    }

    fn visit_expr_stmt(&mut self, stmt: &mut Expr) {
        self.resolve_expr(stmt);
    }

    fn visit_var_stmt(&mut self, stmt: &mut Var) {
        self.declare(stmt.token.clone());
        self.define(stmt.token.clone());
    }

    fn visit_if_stmt(&mut self, stmt: &mut IfStmt) {
        self.resolve_expr(&mut stmt.condition);
        self.resolve_stmt(&mut stmt.then_block);

        if let Some(e) = &mut stmt.else_block {
            self.resolve_stmt(e);
        }
    }

    fn visit_while_stmt(&mut self, stmt: &mut WhileStmt) {
        self.resolve_expr(&mut stmt.condition);
        self.resolve_stmt(&mut stmt.body);
    }

    fn visit_func_stmt(&mut self, stmt: &mut Func) {
        self.declare(stmt.name.clone());
        self.define(stmt.name.clone());

        self.resolve_fun(stmt);
    }

    fn visit_return_stmt(&mut self, stmt: &mut ReturnStmt) {
        if let Some(e) = &mut stmt.value {
            self.resolve_expr(e);
        }
    }
}
