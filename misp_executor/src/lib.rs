mod builtin;
pub mod config;
pub mod environment;

use bigdecimal::BigDecimal;
use misp_parser::SExpr;
use num::{BigInt, BigRational};

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
        control::builtin_let,
        func::{builtin_lambda, builtin_let_func},
        math::{builtin_mod, builtin_not_equal},
        trig::{builtin_acos, builtin_asin, builtin_atan, builtin_cos, builtin_sin, builtin_tan},
    },
    config::{Config, DecimalFormat},
    environment::{Environment, Scope},
};

#[derive(Debug, Clone)]
pub struct Lambda {
    params: Vec<String>,
    body: Box<Value>,
    scope: Scope,
}

type NativeMispFunction = fn(&mut Executor, &[Value]) -> Result<Value, Error>;

#[derive(Debug, Clone)]
pub struct RuntimeMispFunction {
    pub params: Vec<String>,
    pub body: Box<Value>,
}

#[derive(Debug, Clone)]
pub enum Function {
    Native(NativeMispFunction),
    Runtime(RuntimeMispFunction),
}

#[derive(Debug, Clone)]
pub enum Value {
    Atom(String),
    List(Vec<Value>),
    Integer(BigInt),
    Decimal(BigDecimal),
    Rational(BigRational),
    Lambda(Lambda),
    Function(Function),
}

impl From<SExpr> for Value {
    fn from(value: SExpr) -> Self {
        match value {
            SExpr::Atom(str) => Value::Atom(str),
            SExpr::List(sexprs) => Value::List(sexprs.into_iter().map(|e| e.into()).collect()),
            SExpr::Integer(n) => Value::Integer(n),
            SExpr::Decimal(d) => Value::Decimal(d),
            SExpr::Rational(r) => Value::Rational(r),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown Symbol: {0}")]
    UnknownSymbol(String),
    #[error("Invalid Function Call")]
    FunctionCall,
    #[error("Wrong arity for '{name}': expected {expected}, got {actual}")]
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
        env.set_prev(Value::Integer(BigInt::ZERO));

        env.define_native_function("func", builtin_func);
        env.define_native_function("letFunc", builtin_let_func);
        env.define_native_function("lambda", builtin_lambda);

        // Control Flow Functions
        env.define_native_function("if", builtin_if);
        env.define_native_function("let", builtin_let);

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
    fn eval(&mut self, value: &Value) -> Result<Value, Error> {
        match value {
            Value::Atom(name) => self
                .env
                .get(name)
                .cloned()
                .ok_or_else(|| Error::UnknownSymbol(name.clone())),

            Value::List(exprs) => {
                if exprs.is_empty() {
                    return Err(Error::FunctionCall);
                }

                let caller = &self.eval(&exprs[0])?;
                let args = &exprs[1..];

                match caller {
                    Value::Function(func) => match func {
                        Function::Native(f) => f(self, args),
                        Function::Runtime(f) => {
                            if args.len() != f.params.len() {
                                return Err(Error::FunctionArity {
                                    name: self.print(caller),
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
                                self.env.set(param, value.clone());
                            }

                            let result = self.eval(&f.body);

                            self.env.pop_scope();

                            result
                        }
                    },
                    Value::Lambda(lambda) => {
                        if args.len() != lambda.params.len() {
                            return Err(Error::FunctionArity {
                                name: self.print(caller),
                                expected: lambda.params.len(),
                                actual: args.len(),
                            });
                        }

                        let values = args
                            .iter()
                            .map(|a| self.eval(a))
                            .collect::<Result<Vec<_>, _>>()?;

                        self.env.push_given_scope(lambda.scope.clone());

                        for (param, value) in lambda.params.iter().zip(values.iter()) {
                            self.env.set(param, value.clone());
                        }

                        let result = self.eval(&lambda.body);

                        self.env.pop_scope();

                        result
                    }
                    _ => Err(Error::FunctionCall),
                }
            }
            Value::Integer(_)
            | Value::Decimal(_)
            | Value::Rational(_)
            | Value::Lambda(_)
            | Value::Function(_) => Ok(value.clone()),
        }
    }

    pub fn execute(&mut self, expr: &SExpr) -> Result<Value, Error> {
        let prev = self.eval(&expr.clone().into())?;
        self.env.set_prev(prev.clone());
        Ok(prev)
    }

    pub fn print(&self, value: &Value) -> String {
        match value {
            Value::Atom(s) => s.to_string(),
            Value::List(exprs) => {
                let items: Vec<String> = exprs.iter().map(|e| self.print(e)).collect();
                format!("({})", items.join(" "))
            }
            Value::Integer(n) => format!("{n}"),
            Value::Decimal(d) => match self.config.decimal_format {
                DecimalFormat::Standard => d.with_prec(self.config.decimal_precision).to_string(),
                DecimalFormat::Scientific => d
                    .with_prec(self.config.decimal_precision)
                    .to_scientific_notation(),
            },
            Value::Rational(r) => format!("{r}"),
            Value::Lambda(lambda) => format!("lambda ({})", lambda.params.join(", ")),
            Value::Function(func) => format!(
                "func ({})",
                match func {
                    Function::Native(_) => "native".to_string(),
                    Function::Runtime(rt) => rt.params.join(", "),
                }
            ),
        }
    }
}
