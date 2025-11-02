use crate::interpreter::{Environment, Interpreter, RuntimeError};
use crate::scanner::Token;
use crate::statements::Stmt;
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
    pub params: Vec<Token>,
    pub closure: Rc<RefCell<Environment>>,
}

impl Function {
    pub fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError> {
        let env = Rc::new(RefCell::new(Environment::new(Some(self.closure.clone()))));
        for i in 0..self.params.len() {
            env.borrow_mut()
                .values
                .insert(self.params[i].clone().lexeme.unwrap(), arguments[i].clone());
        }

        match interpreter.execute_block(&mut self.body, env)? {
            Some(v) => Ok(v),
            _ => Ok(Object::None),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NativeFunc {
    INPUT,
    PRINTLN,
    PRINT,
}

impl NativeFunc {
    pub fn call(&mut self, name: Token, params: Vec<Object>) -> Result<Object, RuntimeError> {
        match self {
            Self::INPUT => {
                // Optionally accept a single string prompt: input("prompt")
                if params.len() > 1 {
                    return Err(RuntimeError::new(
                        format!("expect 0 or 1 arguments, got {}", params.len()),
                        name,
                    ));
                }

                // If user provided a prompt to input(), print it without newline and flush.
                if params.len() == 1 {
                    // We only accept string prompts here; adjust if your Object type differs.
                    print!("{}", params[0].to_string());
                    if let Err(e) = io::stdout().flush() {
                        return Err(RuntimeError::new(e.to_string(), name));
                    }
                }

                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => Ok(Object::Str(input.trim_end().to_string())), // safe across platforms
                    Err(e) => Err(RuntimeError::new(e.to_string(), name)),
                }
            }

            Self::PRINTLN => {
                // join params by single space (correct spacing logic)
                let output = params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("{}", output);
                // no need to flush after println, but fine to do
                Ok(Object::None)
            }

            Self::PRINT => {
                // join params by single space (correct spacing logic)
                let output = params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                print!("{}", output);
                if let Err(e) = io::stdout().flush() {
                    return Err(RuntimeError::new(e.to_string(), name));
                }
                Ok(Object::None)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    klass: LoxClass,
    fields: Rc<RefCell<HashMap<String, Object>>>,
}

impl LoxInstance {
    pub fn new(klass: LoxClass) -> Self {
        Self {
            klass,
            fields: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: Token) -> Result<Object, RuntimeError> {
        match self.fields.borrow().get(&key.lexeme.clone().unwrap()) {
            Some(v) => Ok(v.clone()),
            _ => Err(RuntimeError::new(
                format!("Undefined property {}", key.lexeme.clone().unwrap()),
                key,
            )),
        }
    }

    pub fn set(&mut self, key: String, value: Object) -> Result<Object, RuntimeError> {
        self.fields.borrow_mut().insert(key, value.clone());
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    Num(f64),
    Str(String),
    Bool(bool),
    Func(Function),
    NativeFunc(NativeFunc),
    Class(LoxClass),
    ClassInstance(LoxInstance),
    None,
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::None => false,
            _ => true,
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Num(l) => match other {
                Self::Num(r) => return l == r,
                _ => return false,
            },
            Self::Str(l) => match other {
                Self::Str(r) => return l == r,
                _ => return false,
            },
            Self::Bool(l) => match other {
                Self::Bool(r) => return l == r,
                _ => return false,
            },
            Self::Func(l) => match other {
                Self::Func(r) => return l == r,
                _ => return false,
            },
            Self::None => match other {
                Self::None => return true,
                _ => return false,
            },
            Self::NativeFunc(l) => match other {
                Self::NativeFunc(r) => return l == r,
                _ => return false,
            },
            Self::Class(l) => match other {
                Self::Class(r) => return l == r,
                _ => return false,
            },
            Self::ClassInstance(l) => match other {
                Self::ClassInstance(r) => return l == r,
                _ => return false,
            },
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(n) => write!(f, "{}", n),
            Self::Str(s) => write!(f, "{}", s),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Func(_) => write!(f, "<user defined> fn"),
            Self::NativeFunc(_) => write!(f, "native fn"),
            Self::Class(c) => write!(f, "{}", c.name),
            Self::ClassInstance(i) => write!(f, "{} instance", i.klass.name),
            Self::None => write!(f, "nil"),
        }
    }
}
