use crate::expressions::{Assign, Binary, Call, Expr, Grouping, Literal, Logical, Unary, Variable};
use crate::scanner::{Object, Token, TokenType};
use crate::statements::{Block, Func, IfStmt, ReturnStmt, Stmt, Var, WhileStmt};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    new_id: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            new_id: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ()> {
        let mut stmts = Vec::new();

        while !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        Ok(stmts)
    }

    fn declaration(&mut self) -> Result<Stmt, ()> {
        if self.matchh(vec![TokenType::VAR]) {
            return self.var_decl();
        }
        if self.matchh(vec![TokenType::FUN]) {
            return self.function();
        }

        self.statement()
    }

    fn var_decl(&mut self) -> Result<Stmt, ()> {
        let name = self.consume(&TokenType::IDENTIFIER, "Expect variable name.")?;

        let mut initializer = None;
        if self.matchh(vec![TokenType::EQUAL]) {
            initializer = Some(self.expression()?);
        }

        self.consume(&TokenType::SEMICOLON, "Expect ';' after value.")?;
        Ok(Stmt::Var(Var {
            token: name,
            initializer,
        }))
    }

    fn statement(&mut self) -> Result<Stmt, ()> {
        if self.matchh(vec![TokenType::RETURN]) {
            return self.return_stmt();
        }

        if self.matchh(vec![TokenType::LEFTBRACE]) {
            return Ok(Stmt::Block(Block {
                stmts: self.block()?,
            }));
        }

        if self.matchh(vec![TokenType::IF]) {
            return self.if_stmt();
        }

        if self.matchh(vec![TokenType::WHILE]) {
            return self.while_stmt();
        }

        if self.matchh(vec![TokenType::FOR]) {
            return self.for_stmt();
        }

        self.expr_stmt()
    }

    fn return_stmt(&mut self) -> Result<Stmt, ()> {
        let keyword = self.previous();
        let mut value = None;
        if !self.check(&TokenType::SEMICOLON) {
            value = Some(self.expression()?);
        }

        self.consume(&TokenType::SEMICOLON, "Expect ';' after return statement.")?;

        Ok(Stmt::Return(ReturnStmt { keyword, value }))
    }

    fn function(&mut self) -> Result<Stmt, ()> {
        let name = self.consume(&TokenType::IDENTIFIER, "Expect name.")?;
        self.consume(&TokenType::LEFTPAREN, "Expect '(' after function name.")?;

        let mut params = Vec::new();
        if !self.check(&TokenType::RIGHTPAREN) {
            params.push(self.consume(&TokenType::IDENTIFIER, "Expect parameter name.")?);

            while self.matchh(vec![TokenType::COMMA]) {
                params.push(self.consume(&TokenType::IDENTIFIER, "Expect parameter name.")?);
            }
        }

        self.consume(&TokenType::RIGHTPAREN, "Expect ')' after parameters.")?;

        self.consume(&TokenType::LEFTBRACE, "Expect '{' before function body.")?;

        let body = self.block()?;

        Ok(Stmt::Func(Func { name, body, params }))
    }

    fn for_stmt(&mut self) -> Result<Stmt, ()> {
        self.consume(&TokenType::LEFTPAREN, "Expect '(' after 'for'.")?;

        let initializer = if self.matchh(vec![TokenType::SEMICOLON]) {
            None
        } else if self.matchh(vec![TokenType::VAR]) {
            Some(self.var_decl()?)
        } else {
            Some(self.expr_stmt()?)
        };

        let condition = if !self.check(&TokenType::SEMICOLON) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(&TokenType::SEMICOLON, "Expect ';' after loop condition.")?;

        let increment = if !self.check(&TokenType::RIGHTPAREN) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(&TokenType::RIGHTPAREN, "Expect ')' after loop clauses.")?;

        let mut body = self.statement()?;

        if let Some(i) = increment {
            body = Stmt::Block(Block {
                stmts: vec![body, Stmt::ExprStmt(i)],
            });
        }

        if let Some(c) = condition {
            body = Stmt::While(WhileStmt {
                condition: c,
                body: Box::new(body),
            });
        } else {
            body = Stmt::While(WhileStmt {
                condition: Expr::Literal(Literal {
                    id: self.get_new_id(),
                    value: Object::Bool(true),
                }),
                body: Box::new(body),
            });
        }

        if let Some(i) = initializer {
            body = Stmt::Block(Block {
                stmts: vec![i, body],
            });
        }

        Ok(body)
    }

    fn while_stmt(&mut self) -> Result<Stmt, ()> {
        self.consume(&TokenType::LEFTPAREN, "Expect '(' after 'if'.")?;

        let condition = self.expression()?;

        self.consume(&TokenType::RIGHTPAREN, "Expect ')' after condition.")?;

        let body = self.statement()?;

        Ok(Stmt::While(WhileStmt {
            condition,
            body: Box::new(body),
        }))
    }

    fn if_stmt(&mut self) -> Result<Stmt, ()> {
        self.consume(&TokenType::LEFTPAREN, "Expect '(' after if statement.")?;

        let condition = self.expression()?;
        self.consume(&TokenType::RIGHTPAREN, "Expect ')' after condition.")?;

        let then_block = self.statement()?;
        let mut else_block = None;

        if self.matchh(vec![TokenType::ELSE]) {
            else_block = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If(IfStmt {
            condition,
            then_block: Box::new(then_block),
            else_block,
        }))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ()> {
        let mut stmts = Vec::new();

        while !self.is_at_end() && !self.check(&TokenType::RIGHTBRACE) {
            stmts.push(self.declaration()?);
        }

        self.consume(&TokenType::RIGHTBRACE, "Expect '}' after block.")?;

        Ok(stmts)
    }

    fn expr_stmt(&mut self) -> Result<Stmt, ()> {
        let expr = self.expression()?;

        self.consume(&TokenType::SEMICOLON, "Expect ';' after value.")?;

        Ok(Stmt::ExprStmt(expr))
    }

    fn expression(&mut self) -> Result<Expr, ()> {
        self.assignment()
    }

    fn primary(&mut self) -> Result<Expr, ()> {
        if self.matchh(vec![TokenType::FALSE]) {
            return Ok(Expr::Literal(Literal {
                id: self.get_new_id(),
                value: Object::Bool(false),
            }));
        }
        if self.matchh(vec![TokenType::TRUE]) {
            return Ok(Expr::Literal(Literal {
                id: self.get_new_id(),
                value: Object::Bool(true),
            }));
        }
        if self.matchh(vec![TokenType::NIL]) {
            return Ok(Expr::Literal(Literal {
                id: self.get_new_id(),
                value: Object::None,
            }));
        }

        if self.matchh(vec![TokenType::NUMBER, TokenType::STRING]) {
            return Ok(Expr::Literal(Literal {
                id: self.get_new_id(),
                value: self.previous().literal,
            }));
        }

        if self.matchh(vec![TokenType::LEFTPAREN]) {
            let expr = self.expression()?;
            self.consume(&TokenType::RIGHTPAREN, "Expect ')' after expression")?;
            return Ok(Expr::Grouping(Grouping {
                id: self.get_new_id(),
                expr: Box::new(expr),
            }));
        }

        if self.matchh(vec![TokenType::IDENTIFIER]) {
            let name = self.previous();

            return Ok(Expr::Var(Variable {
                id: self.get_new_id(),
                name,
            }));
        }

        self.error(self.peek(), "Expect expression");

        Err(())
    }

    fn finish_call(&mut self, calle: Expr) -> Result<Expr, ()> {
        let mut arguments = Vec::new();
        if !self.check(&TokenType::RIGHTPAREN) {
            arguments.push(self.expression()?);

            while self.matchh(vec![TokenType::COMMA]) {
                arguments.push(self.expression()?);
            }
        }

        let paren = self.consume(&TokenType::RIGHTPAREN, "Expect ')' after arguments.")?;
        Ok(Expr::Call(Call {
            id: self.get_new_id(),
            calle: Box::new(calle),
            paren,
            arguments,
        }))
    }

    fn call(&mut self) -> Result<Expr, ()> {
        let mut expr = self.primary()?;

        loop {
            if self.matchh(vec![TokenType::LEFTPAREN]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        return Ok(expr);
    }

    fn unary(&mut self) -> Result<Expr, ()> {
        if self.matchh(vec![TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous();
            let expr = self.unary()?;

            return Ok(Expr::Unary(Unary {
                id: self.get_new_id(),
                operator: operator,
                right: Box::new(expr),
            }));
        }

        return self.call();
    }

    fn factor(&mut self) -> Result<Expr, ()> {
        let mut expr = self.unary()?;

        while self.matchh(vec![TokenType::SLASH, TokenType::STAR]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Binary {
                id: self.get_new_id(),
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ()> {
        let mut expr = self.factor()?;

        while self.matchh(vec![TokenType::PLUS, TokenType::MINUS]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Binary {
                id: self.get_new_id(),
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn comparision(&mut self) -> Result<Expr, ()> {
        let mut expr = self.term()?;

        while self.matchh(vec![
            TokenType::GREATER,
            TokenType::GREATEREQUAL,
            TokenType::EQUALEQUAL,
            TokenType::LESS,
            TokenType::LESSEQUAL,
        ]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Binary {
                id: self.get_new_id(),
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ()> {
        let mut left = self.comparision()?;

        while self.matchh(vec![TokenType::AND]) {
            let operator = self.previous();
            let right = self.comparision()?;
            left = Expr::Logical(Logical {
                id: self.get_new_id(),
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });
        }

        return Ok(left);
    }

    fn or(&mut self) -> Result<Expr, ()> {
        let mut left = self.and()?;

        while self.matchh(vec![TokenType::OR]) {
            let operator = self.previous();
            let right = self.and()?;
            left = Expr::Logical(Logical {
                id: self.get_new_id(),
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });
        }

        return Ok(left);
    }

    fn assignment(&mut self) -> Result<Expr, ()> {
        let expr = self.or()?;

        if self.matchh(vec![TokenType::EQUAL]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Var(v) = expr {
                return Ok(Expr::Assign(Assign {
                    id: self.get_new_id(),
                    name: v.name,
                    value: Box::new(value),
                }));
            }

            self.error(equals, "Invalid assignment target.");
            return Err(());
        }

        return Ok(expr);
    }

    fn error(&self, token: Token, message: &str) {
        if token.token_type == TokenType::EOF {
            eprintln!("[Line: {}] at {} '{}'", token.line, "end", message);
        } else {
            eprintln!("[Line: {}] at {:?} '{}'", token.line, token.lexeme, message);
        };
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn check(&self, token_type: &TokenType) -> bool {
        self.peek().token_type == *token_type
    }

    fn matchh(&mut self, types: Vec<TokenType>) -> bool {
        for t in &types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }

        return false;
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<Token, ()> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        self.error(self.peek(), message);

        Err(())
    }

    fn get_new_id(&mut self) -> usize {
        let id = self.new_id;
        self.new_id += 1;
        id
    }
}
