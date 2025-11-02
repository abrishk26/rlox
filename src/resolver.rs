use crate::expressions::{
    Assign, Binary, Call, Expr, Get, Grouping, Literal, Logical, Set, Unary, Variable, VisitableE,
    VisitorE,
};
use crate::interpreter::Interpreter;
use crate::scanner::Token;
use crate::statements::{
    Block, Class, Func, IfStmt, ReturnStmt, Stmt, Var, VisitableS, VisitorS, WhileStmt,
};
use std::collections::HashMap;

#[derive(Clone)]
enum FunctionType {
    FUNCTION,
    NONE,
}

pub struct Resolver<'a> {
    scopes: Vec<HashMap<String, bool>>,
    interpreter: &'a mut Interpreter,
    current_function: FunctionType,
}

impl<'a> Resolver<'a> {
    pub fn new(scopes: Vec<HashMap<String, bool>>, interpreter: &'a mut Interpreter) -> Self {
        Self {
            scopes,
            interpreter,
            current_function: FunctionType::NONE,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: Token) -> Option<()> {
        if self.scopes.is_empty() {
            return Some(());
        }

        if let Some(_) = self.scopes[self.scopes.len() - 1].get(&name.lexeme.clone().unwrap()) {
            eprintln!(
                "Already a variable with this name in this scope. [Line: {}]",
                name.line
            );
            return None;
        }

        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.lexeme.unwrap(), false);

        Some(())
    }

    fn define(&mut self, name: Token) -> Option<()> {
        if self.scopes.is_empty() {
            return Some(());
        }

        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.lexeme.unwrap(), true);

        Some(())
    }

    pub fn resolve_stmts(&mut self, stmts: &mut Vec<Stmt>) -> Option<()> {
        for stmt in stmts.iter_mut() {
            self.resolve_stmt(stmt)?;
        }

        Some(())
    }

    fn resolve_stmt(&mut self, stmt: &mut Stmt) -> Option<()> {
        stmt.accept(self);
        Some(())
    }

    fn resolve_expr(&mut self, expr: &mut Expr) -> Option<()> {
        expr.accept(self);
        Some(())
    }

    fn resolve_fun(&mut self, func: &mut Func, t: FunctionType) -> Option<()> {
        let enclosing = self.current_function.clone();
        self.current_function = t;
        self.begin_scope();
        for param in &func.params {
            self.declare(param.clone())?;
            self.define(param.clone())?;
        }

        self.resolve_stmts(&mut func.body)?;
        self.end_scope();

        self.current_function = enclosing;

        Some(())
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

impl<'a> VisitorE<Option<()>> for Resolver<'a> {
    fn visit_set(&mut self, expr: &Set) -> Option<()> {
        self.resolve_expr(&mut expr.value.clone());
        self.resolve_expr(&mut expr.expr.clone());
        Some(())
    }

    fn visit_get(&mut self, expr: &Get) -> Option<()> {
        self.resolve_expr(&mut expr.expr.clone());
        Some(())
    }

    fn visit_binary(&mut self, expr: &Binary) -> Option<()> {
        self.resolve_expr(&mut expr.left.clone())?;
        self.resolve_expr(&mut expr.right.clone())
    }

    fn visit_unary(&mut self, expr: &Unary) -> Option<()> {
        self.resolve_expr(&mut expr.right.clone())
    }

    fn visit_grouping(&mut self, expr: &Grouping) -> Option<()> {
        self.resolve_expr(&mut expr.expr.clone())
    }

    fn visit_literal(&mut self, _: &Literal) -> Option<()> {
        Some(())
    }

    fn visit_variable(&mut self, expr: &Variable) -> Option<()> {
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
            return None;
        }
        self.resolve_local(Expr::Var(expr.clone()), expr.name.clone());
        Some(())
    }

    fn visit_assign(&mut self, expr: &Assign) -> Option<()> {
        let mut a = expr.clone();
        self.resolve_expr(&mut a.value)?;
        self.resolve_local(Expr::Assign(a.clone()), a.name.clone());

        Some(())
    }

    fn visit_logical(&mut self, expr: &Logical) -> Option<()> {
        self.resolve_expr(&mut expr.left.clone())?;
        self.resolve_expr(&mut expr.right.clone())
    }

    fn visit_call(&mut self, expr: &Call) -> Option<()> {
        self.resolve_expr(&mut expr.calle.clone())?;

        for arg in expr.arguments.clone().iter_mut() {
            self.resolve_expr(arg)?;
        }

        Some(())
    }
}

impl<'a> VisitorS<Option<()>> for Resolver<'a> {
    fn visit_class_stmt(&mut self, stmt: &mut Class) -> Option<()> {
        self.declare(stmt.name.clone());
        self.define(stmt.name.clone());
        Some(())
    }

    fn visit_block_stmt(&mut self, stmt: &mut Block) -> Option<()> {
        self.begin_scope();
        self.resolve_stmts(&mut stmt.stmts)?;
        self.end_scope();

        Some(())
    }

    fn visit_expr_stmt(&mut self, stmt: &mut Expr) -> Option<()> {
        self.resolve_expr(stmt)
    }

    fn visit_var_stmt(&mut self, stmt: &mut Var) -> Option<()> {
        self.declare(stmt.token.clone())?;
        self.define(stmt.token.clone())
    }

    fn visit_if_stmt(&mut self, stmt: &mut IfStmt) -> Option<()> {
        self.resolve_expr(&mut stmt.condition)?;
        self.resolve_stmt(&mut stmt.then_block)?;

        if let Some(e) = &mut stmt.else_block {
            self.resolve_stmt(e)?;
        }

        Some(())
    }

    fn visit_while_stmt(&mut self, stmt: &mut WhileStmt) -> Option<()> {
        self.resolve_expr(&mut stmt.condition)?;
        self.resolve_stmt(&mut stmt.body)
    }

    fn visit_func_stmt(&mut self, stmt: &mut Func) -> Option<()> {
        self.declare(stmt.name.clone())?;
        self.define(stmt.name.clone())?;

        self.resolve_fun(stmt, FunctionType::FUNCTION)
    }

    fn visit_return_stmt(&mut self, stmt: &mut ReturnStmt) -> Option<()> {
        if let FunctionType::NONE = self.current_function {
            eprintln!(
                "Can't return from top-level code. [Line: {}]",
                stmt.keyword.line
            );
            return None;
        }

        if let Some(e) = &mut stmt.value {
            self.resolve_expr(e)?;
        }

        Some(())
    }
}
