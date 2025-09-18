mod builtin;
pub mod config;
pub mod environment;

use misp_parser::SExpr;
use num::BigInt;

use builtin::{
    control::builtin_if,
    func::builtin_func,
    math::{
        builtin_add, builtin_divide, builtin_equal, builtin_gt, builtin_gte, builtin_lt,
        builtin_lte, builtin_minus, builtin_multiply, builtin_pow, builtin_sqrt,
    },
};

use crate::{
    builtin::{
        func::builtin_lambda,
        math::{builtin_mod, builtin_not_equal},
        trig::{builtin_acos, builtin_asin, builtin_atan, builtin_cos, builtin_sin, builtin_tan},
    },
    config::{Config, DecimalFormat},
    environment::{Environment, Function},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown Symbol: {0}")]
    UnknownSymbol(String),
    #[error("Invalid Function Call")]
    FunctionCall,
    #[error("Wrong arity for func '{name}': expected {expected}, got {actual}")]
    FunctionArity {
        name: String,
        expected: usize,
        actual: usize,
    },
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
}

pub struct Executor {
    config: Config,
    env: Environment,
}

impl Default for Executor {
    fn default() -> Self {
        let config = Config::default();
        let mut env = Environment::default();

        env.push_scope();
        env.set_prev(SExpr::Integer(BigInt::ZERO));

        env.define_native_function("func", builtin_func);
        env.define_native_function("lambda", builtin_lambda);

        // Control Flow Functions
        env.define_native_function("if", builtin_if);

        // Math Functions
        env.define_native_function("+", builtin_add);
        env.define_native_function("-", builtin_minus);
        env.define_native_function("*", builtin_multiply);
        env.define_native_function("/", builtin_divide);
        env.define_native_function("%", builtin_mod);
        env.define_native_function("==", builtin_equal);
        env.define_native_function("!=", builtin_not_equal);
        env.define_native_function("<", builtin_lt);
        env.define_native_function("<=", builtin_lte);
        env.define_native_function(">", builtin_gt);
        env.define_native_function(">=", builtin_gte);
        env.define_native_function("pow", builtin_pow);
        env.define_native_function("sqrt", builtin_sqrt);

        // Trig Functions
        env.define_native_function("sin", builtin_sin);
        env.define_native_function("cos", builtin_cos);
        env.define_native_function("tan", builtin_tan);
        env.define_native_function("asin", builtin_asin);
        env.define_native_function("acos", builtin_acos);
        env.define_native_function("atan", builtin_atan);

        Self { config, env }
    }
}

impl Executor {
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
                    .get_function(func_name)
                    .cloned()
                    .ok_or_else(|| Error::FunctionNotFound(func_name.to_string()))?;

                let args = &exprs[1..];

                match func {
                    Function::Native(f) => f(self, args),
                    Function::UserDefined(f) => {
                        if args.len() != f.params.len() {
                            return Err(Error::FunctionArity {
                                name: func_name.to_string(),
                                expected: f.params.len(),
                                actual: args.len(),
                            });
                        }

                        let values = args
                            .iter()
                            .map(|a| self.eval(a))
                            .collect::<Result<Vec<_>, _>>()?;

                        self.env.push_scope();

                        for (param, value) in f.params.iter().zip(values.iter()) {
                            self.env.set_variable(param, value.clone());
                        }

                        let result = self.eval(&f.body);

                        self.env.pop_scope();

                        result
                    }
                }
            }
            SExpr::Integer(_) | SExpr::Decimal(_) | SExpr::Rational(_) => Ok(expr.clone()),
        }
    }

    pub fn execute(&mut self, expr: &SExpr) -> Result<SExpr, Error> {
        let prev = self.eval(expr)?;
        self.env.set_prev(prev.clone());
        Ok(prev)
    }

    pub fn print(&self, expr: &SExpr) -> String {
        match expr {
            SExpr::Atom(s) => s.to_string(),
            SExpr::List(exprs) => {
                let items: Vec<String> = exprs.iter().map(|e| self.print(e)).collect();
                format!("({})", items.join(" "))
            }
            SExpr::Integer(n) => format!("{n}"),
            SExpr::Decimal(d) => match self.config.decimal_format {
                DecimalFormat::Standard => d.with_prec(self.config.decimal_precision).to_string(),
                DecimalFormat::Scientific => d
                    .with_prec(self.config.decimal_precision)
                    .to_scientific_notation(),
            },
            SExpr::Rational(r) => format!("{r}"),
        }
    }
}
