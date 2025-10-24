use crate::expressions::{
    Assign, Binary, Call, Expr, Function, Grouping, Literal, Logical, Unary, Variable, VisitableE,
    VisitorE,
};
use crate::scanner::{Object, Token, TokenType};
use crate::statements::{
    Block, Func, IfStmt, Print, ReturnStmt, Stmt, Var, VisitableS, VisitorS, WhileStmt,
};
use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

pub struct RuntimeError {
    message: &'static str,
    token: Token,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Runtime Error: {} - [Line: {}]",
            self.message, self.token.line
        )
    }
}

#[derive(Clone, Debug)]
pub struct Environment {
    values: HashMap<String, Object>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    fn get(&self, name: Token) -> Result<Object, RuntimeError> {
        match self.values.get(&name.clone().lexeme.unwrap()) {
            Some(v) => Ok(v.clone()),
            _ => match &self.enclosing {
                Some(e) => e.borrow().get(name),
                _ => Err(RuntimeError {
                    message: "Undefined variable.",
                    token: name,
                }),
            },
        }
    }

    fn assign(&mut self, name: Token, value: Object) -> Result<Object, RuntimeError> {
        let key = name.lexeme.clone().unwrap();
        match self.values.get(&key) {
            Some(_) => {
                self.set(key, value.clone());
                return Ok(value);
            }
            _ => match &self.enclosing {
                Some(e) => e.borrow_mut().assign(name, value),
                _ => Err(RuntimeError {
                    message: "Undefined variable.",
                    token: name,
                }),
            },
        }
    }
}

pub struct Interpreter {
    pub env: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Rc::new(RefCell::new(Environment::new(None))),
        }
    }
    pub fn evaluate(&mut self, expr: &mut Expr) -> Result<Object, RuntimeError> {
        return expr.accept(self);
    }

    pub fn execute(&mut self, stmt: &mut Stmt) -> Result<Option<Object>, RuntimeError> {
        stmt.accept(self)
    }

    pub fn execute_block(
        &mut self,
        stmts: &mut Vec<Stmt>,
        env: Rc<RefCell<Environment>>,
    ) -> Result<Option<Object>, RuntimeError> {
        let prev = env.clone();
        let new_env = Rc::new(RefCell::new(Environment::new(Some(prev.clone()))));

        self.env = new_env;

        for stmt in stmts.iter_mut() {
            if let Some(r) = self.execute(stmt)? {
                return Ok(Some(r));
            }
        }

        self.env = prev;

        Ok(None)
    }

    pub fn interpret(&mut self, mut stmts: Vec<Stmt>) {
        for stmt in stmts.iter_mut() {
            match self.execute(stmt) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}", e);
                    break;
                }
            }
        }
    }
}

impl VisitorS<Result<Option<Object>, RuntimeError>> for Interpreter {
    fn visit_return_stmt(&mut self, stmt: &mut ReturnStmt) -> Result<Option<Object>, RuntimeError> {
        let mut value = Object::None;
        if let Some(e) = &mut stmt.value {
            value = self.evaluate(e)?;
        }

        Ok(Some(value))
    }
    fn visit_while_stmt(&mut self, stmt: &mut WhileStmt) -> Result<Option<Object>, RuntimeError> {
        while self.evaluate(&mut stmt.condition)?.is_truthy() {
            if let Some(r) = self.execute(&mut stmt.body)? {
                return Ok(Some(r));
            }
        }

        Ok(None)
    }

    fn visit_var_stmt(&mut self, stmt: &mut Var) -> Result<Option<Object>, RuntimeError> {
        let value = match stmt.initializer.clone() {
            Some(ref mut e) => self.evaluate(e)?,
            _ => Object::None,
        };
        self.env
            .borrow_mut()
            .set(stmt.token.clone().lexeme.unwrap(), value);
        Ok(None)
    }

    fn visit_print(&mut self, stmt: &mut Print) -> Result<Option<Object>, RuntimeError> {
        let value = self.evaluate(&mut stmt.expr.clone())?;
        println!("{}", value);
        Ok(None)
    }

    fn visit_expr_stmt(&mut self, expr: &mut Expr) -> Result<Option<Object>, RuntimeError> {
        self.evaluate(&mut expr.clone())?;
        Ok(None)
    }

    fn visit_block_stmt(&mut self, block: &mut Block) -> Result<Option<Object>, RuntimeError> {
        self.execute_block(&mut block.stmts, self.env.clone())
    }

    fn visit_if_stmt(&mut self, stmt: &mut IfStmt) -> Result<Option<Object>, RuntimeError> {
        if self.evaluate(&mut stmt.condition)?.is_truthy() {
            return self.execute(&mut stmt.then_block);
        } else {
            if let Some(s) = &mut stmt.else_block {
                return self.execute(s);
            }
        }

        Ok(None)
    }

    fn visit_func_stmt(&mut self, stmt: &mut Func) -> Result<Option<Object>, RuntimeError> {
        let function = Function {
            name: stmt.name.clone().lexeme.unwrap(),
            body: stmt.body.clone(),
            params: stmt.params.clone(),
        };

        self.env
            .borrow_mut()
            .set(stmt.name.clone().lexeme.unwrap(), Object::Func(function));

        Ok(None)
    }
}

impl VisitorE<Result<Object, RuntimeError>> for Interpreter {
    fn visit_call(&mut self, expr: &Call) -> Result<Object, RuntimeError> {
        let mut calle = self.evaluate(&mut expr.calle.clone())?;

        match &mut calle {
            Object::Func(f) => {
                return f.call(self, expr.arguments.clone());
            }
            _ => {
                return Err(RuntimeError {
                    message: "Can only call functions and classes.",
                    token: expr.paren.clone(),
                });
            }
        }
    }

    fn visit_logical(&mut self, expr: &Logical) -> Result<Object, RuntimeError> {
        let left = self.evaluate(&mut expr.left.clone())?;
        let right = self.evaluate(&mut expr.right.clone())?;

        match expr.operator.token_type {
            TokenType::OR => {
                if left.is_truthy() {
                    return Ok(left);
                }
                Ok(right)
            }
            TokenType::AND => {
                if !left.is_truthy() {
                    return Ok(left);
                }
                Ok(right)
            }
            _ => unreachable!(),
        }
    }

    fn visit_variable(&mut self, expr: &Variable) -> Result<Object, RuntimeError> {
        self.env.borrow().get(expr.name.clone())
    }
    fn visit_binary(&mut self, expr: &Binary) -> Result<Object, RuntimeError> {
        let left = self.evaluate(&mut expr.left.clone())?;
        let right = self.evaluate(&mut expr.right.clone())?;

        match expr.operator.token_type {
            TokenType::MINUS => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Num(l - r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.",
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers.",
                    token: expr.operator.clone(),
                }),
            },
            TokenType::PLUS => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Num(l + r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.",
                        token: expr.operator.clone(),
                    }),
                },
                Object::Str(l) => match right {
                    Object::Str(r) => {
                        let mut s = l.clone();
                        s.push_str(&r);
                        return Ok(Object::Str(s));
                    }
                    _ => Err(RuntimeError {
                        message: "operands must be two strings.",
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers or two strings.",
                    token: expr.operator.clone(),
                }),
            },
            TokenType::STAR => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Num(l * r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.",
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers.",
                    token: expr.operator.clone(),
                }),
            },
            TokenType::SLASH => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Num(l / r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.",
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers.",
                    token: expr.operator.clone(),
                }),
            },
            TokenType::GREATER => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Bool(l > r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.",
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers.",
                    token: expr.operator.clone(),
                }),
            },
            TokenType::LESS => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Bool(l < r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.",
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numberss.",
                    token: expr.operator.clone(),
                }),
            },
            TokenType::GREATEREQUAL => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Bool(l >= r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.",
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers.",
                    token: expr.operator.clone(),
                }),
            },
            TokenType::LESSEQUAL => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Bool(l <= r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.",
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers.",
                    token: expr.operator.clone(),
                }),
            },
            TokenType::EQUALEQUAL => return Ok(Object::Bool(left == right)),
            TokenType::BANGEQUAL => return Ok(Object::Bool(!(left == right))),
            _ => unreachable!(),
        }
    }

    fn visit_unary(&mut self, expr: &Unary) -> Result<Object, RuntimeError> {
        let right = self.evaluate(&mut expr.right.clone())?;

        match expr.operator.token_type {
            TokenType::MINUS => match right {
                Object::Num(n) => return Ok(Object::Num(-1. * n)),
                _ => Err(RuntimeError {
                    message: "operands must be numbers.",
                    token: expr.operator.clone(),
                }),
            },
            TokenType::BANG => Ok(Object::Bool(!right.is_truthy())),
            _ => unreachable!(),
        }
    }

    fn visit_grouping(&mut self, expr: &Grouping) -> Result<Object, RuntimeError> {
        return self.evaluate(&mut expr.expr.clone());
    }

    fn visit_literal(&mut self, expr: &Literal) -> Result<Object, RuntimeError> {
        Ok(expr.value.clone())
    }

    fn visit_assign(&mut self, expr: &Assign) -> Result<Object, RuntimeError> {
        let value = self.evaluate(&mut expr.value.clone())?;

        self.env.borrow_mut().assign(expr.name.clone(), value)
    }
}
