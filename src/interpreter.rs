use crate::expressions::{
    Assign, Binary, Call, Expr, Get, Grouping, Literal, Logical, Set, This, Unary, Variable,
    VisitableE, VisitorE,
};
use crate::scanner::{Token, TokenType};
use crate::statements::{
    Block, Class, Func, IfStmt, ReturnStmt, Stmt, Var, VisitableS, VisitorS, WhileStmt,
};
use crate::types::{Function, LoxClass, LoxInstance, NativeFunc, Object};
use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

pub struct RuntimeError {
    message: String,
    token: Token,
}

impl RuntimeError {
    pub fn new(message: String, token: Token) -> Self {
        Self { message, token }
    }
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

#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    pub values: HashMap<String, Object>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

fn ancestor(mut env: Rc<RefCell<Environment>>, distance: usize) -> Rc<RefCell<Environment>> {
    for _ in 0..distance {
        let new_env = env.borrow().enclosing.clone().unwrap();
        env = new_env;
    }
    env
}

pub fn get_at(env: Rc<RefCell<Environment>>, distance: usize, name: String) -> Option<Object> {
    ancestor(env, distance).borrow().values.get(&name).cloned()
}

fn assign_at(env: Rc<RefCell<Environment>>, distance: usize, name: Token, value: Object) {
    ancestor(env, distance)
        .borrow_mut()
        .values
        .insert(name.lexeme.unwrap(), value);
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    //fn get(&self, name: Token) -> Result<Object, RuntimeError> {
    //    match self.values.get(&name.clone().lexeme.unwrap()) {
    //        Some(v) => Ok(v.clone()),
    //        _ => match &self.enclosing {
    //            Some(e) => e.borrow().get(name),
    //            _ => Err(RuntimeError {
    //                message: "Undefined variable.",
    //                token: name,
    //            }),
    //        },
    //    }
    //}

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
                    message: "Undefined variable.".to_string(),
                    token: name,
                }),
            },
        }
    }
}

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    pub env: Rc<RefCell<Environment>>,
    locals: HashMap<usize, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let global = Rc::new(RefCell::new(Environment {
            values: HashMap::from([
                ("input".to_string(), Object::NativeFunc(NativeFunc::INPUT)),
                (
                    "println".to_string(),
                    Object::NativeFunc(NativeFunc::PRINTLN),
                ),
                ("print".to_string(), Object::NativeFunc(NativeFunc::PRINT)),
            ]),
            enclosing: None,
        }));

        Self {
            env: global.clone(),
            globals: global,
            locals: HashMap::new(),
        }
    }

    pub fn evaluate(&mut self, expr: &mut Expr) -> Result<Object, RuntimeError> {
        return expr.accept(self);
    }

    pub fn execute(&mut self, stmt: &mut Stmt) -> Result<Option<Object>, RuntimeError> {
        stmt.accept(self)
    }

    pub fn resolve(&mut self, value: Expr, depth: usize) {
        self.locals.insert(value.get_id(), depth);
    }

    pub fn execute_block(
        &mut self,
        stmts: &mut Vec<Stmt>,
        env: Rc<RefCell<Environment>>,
    ) -> Result<Option<Object>, RuntimeError> {
        let prev = self.env.clone();

        self.env = env.clone();
        for stmt in stmts.iter_mut() {
            if let Some(r) = self.execute(stmt)? {
                self.env = prev;
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

    fn lookup_variable(&mut self, name: Token, expr: Expr) -> Option<Object> {
        if let Some(d) = self.locals.get(&expr.get_id()) {
            return get_at(self.env.clone(), *d, name.lexeme.unwrap());
        } else {
            return self
                .globals
                .borrow()
                .values
                .get(&name.lexeme.clone().unwrap())
                .map(|x| x.clone());
        }
    }
}

impl VisitorS<Result<Option<Object>, RuntimeError>> for Interpreter {
    fn visit_class_stmt(&mut self, stmt: &mut Class) -> Result<Option<Object>, RuntimeError> {
        self.env
            .borrow_mut()
            .set(stmt.name.lexeme.clone().unwrap(), Object::None);

        let mut methods = HashMap::new();
        for method in stmt.methods.clone() {
            let name = method.name.lexeme.clone().unwrap();
            methods.insert(
                name.clone(),
                Function {
                    name: method.name.lexeme.unwrap(),
                    is_init: name.clone() == "init".to_string(),
                    body: method.body,
                    params: method.params,
                    closure: self.env.clone(),
                },
            );
        }

        let klass = LoxClass::new(stmt.name.lexeme.clone().unwrap(), methods);
        self.env
            .borrow_mut()
            .assign(stmt.name.clone(), Object::Class(klass))?;

        Ok(None)
    }

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

    fn visit_expr_stmt(&mut self, expr: &mut Expr) -> Result<Option<Object>, RuntimeError> {
        self.evaluate(&mut expr.clone())?;
        Ok(None)
    }

    fn visit_block_stmt(&mut self, block: &mut Block) -> Result<Option<Object>, RuntimeError> {
        self.execute_block(
            &mut block.stmts,
            Rc::new(RefCell::new(Environment::new(Some(self.env.clone())))),
        )
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
        let name = stmt.name.clone().lexeme.unwrap();

        let function = Function {
            name: stmt.name.clone().lexeme.unwrap(),
            is_init: false,
            body: stmt.body.clone(),
            params: stmt.params.clone(),
            closure: self.env.clone(),
        };

        self.env
            .borrow_mut()
            .set(name.clone(), Object::Func(function));

        Ok(None)
    }
}

impl VisitorE<Result<Object, RuntimeError>> for Interpreter {
    fn visit_this(&mut self, expr: &This) -> Result<Object, RuntimeError> {
        if let Some(o) = self.lookup_variable(expr.keyword.clone(), Expr::This(expr.clone())) {
            Ok(o)
        } else {
            Err(RuntimeError::new(
                "Undeclared variable this".to_string(),
                expr.keyword.clone(),
            ))
        }
    }

    fn visit_set(&mut self, expr: &Set) -> Result<Object, RuntimeError> {
        let mut left = self.evaluate(&mut expr.expr.clone())?;
        if let Object::ClassInstance(i) = &mut left {
            return i.set(
                expr.name.lexeme.clone().unwrap(),
                self.evaluate(&mut expr.value.clone())?,
            );
        } else {
            return Err(RuntimeError::new(
                "Only instances have properties.".to_string(),
                expr.name.clone(),
            ));
        }
    }

    fn visit_get(&mut self, expr: &Get) -> Result<Object, RuntimeError> {
        let left = self.evaluate(&mut expr.expr.clone())?;
        if let Object::ClassInstance(i) = left {
            return i.get(expr.name.clone());
        } else {
            return Err(RuntimeError::new(
                "Only instances have properties.".to_string(),
                expr.name.clone(),
            ));
        }
    }

    fn visit_call(&mut self, expr: &Call) -> Result<Object, RuntimeError> {
        let mut calle = self.evaluate(&mut expr.calle.clone())?;
        match &mut calle {
            Object::Func(f) => {
                let mut args = Vec::new();
                for arg in expr.arguments.clone().iter_mut() {
                    args.push(self.evaluate(arg)?);
                }
                return f.call(self, args);
            }
            Object::NativeFunc(f) => {
                let mut args = Vec::new();
                for arg in expr.arguments.clone().iter_mut() {
                    args.push(self.evaluate(arg)?);
                }

                return f.call(expr.paren.clone(), args);
            }
            Object::Class(c) => {
                let mut args = Vec::new();
                for arg in expr.arguments.clone().iter_mut() {
                    args.push(self.evaluate(arg)?);
                }

                let instance = LoxInstance::new(c.clone());
                if let Some(init_method) = c.find_method("init".to_string()) {
                    init_method.bind(instance.clone()).call(self, args)?;
                }

                return Ok(Object::ClassInstance(instance));
            }
            _ => {
                return Err(RuntimeError {
                    message: "Can only call functions and classes.".to_owned(),
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
        match self.lookup_variable(expr.name.clone(), Expr::Var(expr.clone())) {
            Some(o) => Ok(o),
            _ => Err(RuntimeError {
                message: "Undefined variable.".to_string(),
                token: expr.name.clone(),
            }),
        }
    }

    fn visit_binary(&mut self, expr: &Binary) -> Result<Object, RuntimeError> {
        let left = self.evaluate(&mut expr.left.clone())?;
        let right = self.evaluate(&mut expr.right.clone())?;

        match expr.operator.token_type {
            TokenType::MINUS => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Num(l - r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.".to_string(),
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers.".to_string(),
                    token: expr.operator.clone(),
                }),
            },
            TokenType::PLUS => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Num(l + r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.".to_string(),
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
                        message: "operands must be two strings.".to_string(),
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers or two strings.".to_string(),
                    token: expr.operator.clone(),
                }),
            },
            TokenType::STAR => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Num(l * r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.".to_string(),
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers.".to_string(),
                    token: expr.operator.clone(),
                }),
            },
            TokenType::SLASH => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Num(l / r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.".to_string(),
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers.".to_string(),
                    token: expr.operator.clone(),
                }),
            },
            TokenType::GREATER => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Bool(l > r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.".to_string(),
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers.".to_string(),
                    token: expr.operator.clone(),
                }),
            },
            TokenType::LESS => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Bool(l < r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.".to_string(),
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numberss.".to_string(),
                    token: expr.operator.clone(),
                }),
            },
            TokenType::GREATEREQUAL => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Bool(l >= r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.".to_string(),
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers.".to_string(),
                    token: expr.operator.clone(),
                }),
            },
            TokenType::LESSEQUAL => match left {
                Object::Num(l) => match right {
                    Object::Num(r) => return Ok(Object::Bool(l <= r)),
                    _ => Err(RuntimeError {
                        message: "operands must be two numbers.".to_string(),
                        token: expr.operator.clone(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "operands must be two numbers.".to_string(),
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
                    message: "operands must be numbers.".to_string(),
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

        if let Some(d) = self.locals.get(&expr.id) {
            assign_at(self.env.clone(), *d, expr.name.clone(), value.clone());
            // return the value we just assigned
            Ok(get_at(self.env.clone(), *d, expr.name.lexeme.clone().unwrap()).unwrap())
        } else {
            // assign into globals and return the assigned value
            self.globals
                .borrow_mut()
                .values
                .insert(expr.name.lexeme.clone().unwrap(), value.clone());
            Ok(value)
        }
    }
}
