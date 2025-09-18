mod math;

use misp_parser::SExpr;
use num::BigInt;
use std::collections::HashMap;

use crate::math::{builtin_add, builtin_divide, builtin_minus, builtin_multiply, builtin_sqrt};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown Symbol: {0}")]
    UnknownSymbol(String),
    #[error("Invalid Function Call")]
    FunctionCall,
    #[error("Wrong arity for Function {name}: expected {expected}, got {actual}")]
    FunctionArity {
        name: String,
        expected: usize,
        actual: usize,
    },
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
}

type NativeMispFunction = fn(&mut Executor, &[SExpr]) -> Result<SExpr, Error>;

pub enum Function {
    Native(NativeMispFunction),
    UserDefined {
        params: Vec<String>,
        body: Box<SExpr>,
    },
}

#[derive(Default)]
pub struct Environment {
    variables: HashMap<String, SExpr>,
    functions: HashMap<String, Function>,
}

impl Environment {
    pub fn define_native_function(&mut self, name: impl ToString, f: NativeMispFunction) {
        self.functions.insert(name.to_string(), Function::Native(f));
    }

    pub fn set_variable(&mut self, name: impl ToString, value: SExpr) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&SExpr> {
        self.variables.get(name)
    }
}
pub struct Executor {
    env: Environment,
}

impl Default for Executor {
    fn default() -> Self {
        let mut env = Environment::default();

        env.set_variable("ans", SExpr::Integer(BigInt::ZERO));

        env.define_native_function("+", builtin_add);
        env.define_native_function("-", builtin_minus);
        env.define_native_function("*", builtin_multiply);
        env.define_native_function("/", builtin_divide);
        env.define_native_function("sqrt", builtin_sqrt);

        Self { env }
    }
}

impl Executor {
    pub fn new() -> Self {
        Self {
            env: Environment::default(),
        }
    }

    fn eval(&mut self, expr: &SExpr) -> Result<SExpr, Error> {
        match expr {
            SExpr::Atom(name) => self
                .env
                .get_variable(name)
                .cloned()
                .ok_or_else(|| Error::UnknownSymbol(name.clone())),
            SExpr::List(exprs) => {
                if exprs.is_empty() {
                    return Err(Error::FunctionCall);
                }

                let func_name = match &exprs[0] {
                    SExpr::Atom(name) => name,
                    _ => return Err(Error::FunctionCall),
                };

                let func = self
                    .env
                    .functions
                    .get(func_name)
                    .ok_or_else(|| Error::FunctionNotFound(func_name.to_string()))?;
                let args = &exprs[1..];

                match func {
                    Function::Native(f) => f(self, args),
                    _ => todo!(),
                }
            }
            SExpr::Integer(_) | SExpr::Decimal(_) | SExpr::Rational(_) => Ok(expr.clone()),
        }
    }

    pub fn execute(&mut self, expr: &SExpr) -> Result<SExpr, Error> {
        let ans = self.eval(expr)?;
        self.env.set_variable("ans", ans.clone());
        Ok(ans)
    }
}
