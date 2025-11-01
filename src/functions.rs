use crate::interpreter::{Interpreter, RuntimeError, Environment};
use crate::scanner::{Object, Token};
use crate::statements::Stmt;
use std::io::{self, Write};
use std::{rc::Rc, cell::RefCell};


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
