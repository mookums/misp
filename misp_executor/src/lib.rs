mod builtin;
pub mod config;
pub mod environment;

use std::{collections::VecDeque, thread::sleep, time::Duration};

use misp_num::decimal::Decimal;
use misp_parser::SExpr;

use crate::{
    builtin::math::builtin_add,
    config::Config,
    environment::{Environment, Scope},
};

#[derive(Debug, Clone)]
pub struct Lambda {
    pub params: Vec<String>,
    pub body: Box<Value>,
    pub scope: Scope,
}

type NativeMispFunction = fn(&mut Executor) -> Result<(), Error>;

#[derive(Debug, Clone)]
pub struct RuntimeMispFunction {
    pub params: Vec<String>,
    pub body: Box<Value>,
}

#[derive(Debug, Clone)]
pub enum Function {
    Native(NativeMispFunction),
    Runtime(RuntimeMispFunction),
    Lambda(Lambda),
}

#[derive(Debug, Clone)]
pub enum Value {
    Atom(String),
    List(Vec<Value>),
    Decimal(Decimal),
    Function(Function),
}

impl From<SExpr> for Value {
    fn from(value: SExpr) -> Self {
        match value {
            SExpr::Atom(str) => Value::Atom(str),
            SExpr::List(sexprs) => Value::List(sexprs.into_iter().map(|e| e.into()).collect()),
            SExpr::Decimal(d) => Value::Decimal(d),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Push(Value),
    Store(String),
    Load(String),
    Call,
    PushDefinedScope(Scope),
    PopScope,
    // Arithmetic Instructions
    Add,
    Sub,
    Mult,
    Div,
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    Sqrt,
    Pow,
    If,
}

macro_rules! binary_op {
    ($s: ident, $op:tt) => {{
        let (Value::Decimal(second), Value::Decimal(first)) =
            ($s.stack.pop().unwrap(), $s.stack.pop().unwrap())
        else {
            panic!()
        };

        $s.stack.push(Value::Decimal(Decimal::from(first $op second)));
    }};
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
    #[error("Function not found")]
    FunctionNotFound,
    #[error("Recursion limit reached")]
    Recursion,
}

pub struct Injector<'a> {
    instructions: &'a mut VecDeque<Instruction>,
    index: usize,
}

impl<'a> Injector<'a> {
    pub fn inject(&mut self, instruction: Instruction) {
        self.instructions.insert(self.index, instruction);
        self.index += 1;
    }
}

#[derive(Debug, Clone)]
pub struct Executor {
    pub config: Config,
    pub env: Environment,
    pub instructions: VecDeque<Instruction>,
    pub stack: Vec<Value>,
}

impl Default for Executor {
    fn default() -> Self {
        let config = Config::default();
        let mut env = Environment::default();

        env.push_scope();
        env.set_prev(Value::Decimal(Decimal::ZERO));
        env.set("pi", Value::Decimal(Decimal::PI));
        env.set("e", Value::Decimal(Decimal::E));

        // env.define_native_function("func", builtin_func);
        // env.define_native_function("letFunc", builtin_let_func);
        // env.define_native_function("lambda", builtin_lambda);

        // Control Flow Functions
        // env.define_native_function("if", builtin_if);
        // env.define_native_function("let", builtin_let);

        // Math Functions
        env.define_native_function("+", builtin_add);
        // env.define_native_function("-", builtin_minus);
        // env.define_native_function("*", builtin_multiply);
        // env.define_native_function("/", builtin_divide);
        // env.define_native_function("%", builtin_mod);
        // env.define_native_function("==", builtin_equal);
        // env.define_native_function("!=", builtin_not_equal);
        // env.define_native_function("<", builtin_lt);
        // env.define_native_function("<=", builtin_lte);
        // env.define_native_function(">", builtin_gt);
        // env.define_native_function(">=", builtin_gte);
        // env.define_native_function("pow", builtin_pow);
        // env.define_native_function("sqrt", builtin_sqrt);
        // env.define_native_function("summate", builtin_summate);
        // env.define_native_function("factorial", builtin_factorial);

        // Trig Functions
        // env.define_native_function("sin", builtin_sin);
        // env.define_native_function("cos", builtin_cos);
        // env.define_native_function("tan", builtin_tan);
        // env.define_native_function("asin", builtin_asin);
        // env.define_native_function("acos", builtin_acos);
        // env.define_native_function("atan", builtin_atan);

        Self {
            config,
            env,
            instructions: VecDeque::new(),
            stack: Vec::new(),
        }
    }
}

impl Executor {
    pub fn inject_compiled(value: Value, injector: &mut Injector) -> Result<(), Error> {
        match value {
            Value::Atom(atom) => injector.inject(Instruction::Load(atom)),
            Value::Decimal(_) | Value::Function(_) => {
                injector.inject(Instruction::Push(value));
            }
            Value::List(mut values) => {
                let mut drain = values.drain(0..values.len());
                let Value::Atom(name) = drain.next().unwrap() else {
                    panic!();
                };

                for param in drain {
                    injector.inject(Instruction::Push(param));
                }

                injector.inject(Instruction::Load(name));
                injector.inject(Instruction::Call);
            }
        }

        Ok(())
    }

    pub fn inject_function(&mut self, function: Function) -> Result<(), Error> {
        let mut injector = Injector {
            instructions: &mut self.instructions,
            index: 0,
        };

        match function {
            Function::Native(f) => {
                f(self)?;
            }
            Function::Runtime(f) => {
                for param in f.params.into_iter() {
                    injector.inject(Instruction::Push(self.stack.pop().unwrap()));
                    injector.inject(Instruction::Store(param));
                }

                Self::inject_compiled(*f.body, &mut injector)?;
            }
            Function::Lambda(l) => {
                injector.inject(Instruction::PushDefinedScope(l.scope));
                for param in l.params.into_iter() {
                    injector.inject(Instruction::Push(self.stack.pop().unwrap()));
                    injector.inject(Instruction::Store(param));
                }

                Self::inject_compiled(*l.body, &mut injector)?;
                injector.inject(Instruction::PopScope);
            }
        }

        Ok(())
    }

    pub fn execute(&mut self, value: Value) -> Result<Value, Error> {
        self.instructions.clear();
        Self::inject_compiled(
            value,
            &mut Injector {
                instructions: &mut self.instructions,
                index: 0,
            },
        )?;
        self.stack.clear();

        while let Some(instruction) = self.instructions.pop_front() {
            // eprintln!("Current Instruction: {instruction:?}");
            // eprintln!("Instructions: {:?}", self.instructions);
            // eprintln!("Stack: {:?}", self.stack);
            // sleep(Duration::from_secs(2));

            match instruction {
                Instruction::Push(value) => {
                    // Pseudo-pipelining
                    if let Some(instr) = self.instructions.front() {
                        match instr {
                            Instruction::Store(_) => {
                                let Instruction::Store(name) =
                                    self.instructions.pop_front().unwrap()
                                else {
                                    unreachable!()
                                };

                                self.env.set(name, value);
                                continue;
                            }
                            Instruction::Call => todo!(),
                            _ => {}
                        }
                    }

                    self.stack.push(value);
                }
                Instruction::Store(name) => {
                    self.env.set(name, self.stack.pop().unwrap());
                }
                Instruction::Load(name) => {
                    let value = self.env.get(&name).ok_or(Error::UnknownSymbol(name))?;
                    self.stack.push(value.clone());
                }
                Instruction::Call => {
                    let func = self.stack.pop().unwrap();

                    match func {
                        Value::Function(f) => {
                            self.inject_function(f)?;
                        }
                        _ => return Err(Error::FunctionNotFound),
                    }
                }
                Instruction::PushDefinedScope(scope) => {
                    self.env.push_given_scope(scope);
                }
                Instruction::PopScope => {
                    self.env.pop_scope();
                }
                Instruction::Add => binary_op!(self, +),
                Instruction::Sub => binary_op!(self, -),
                Instruction::Mult => binary_op!(self, *),
                Instruction::Div => binary_op!(self, /),
                Instruction::Eq => binary_op!(self, ==),
                Instruction::NotEq => binary_op!(self, !=),
                Instruction::Lt => binary_op!(self, <),
                Instruction::Lte => binary_op!(self, <=),
                Instruction::Gt => binary_op!(self, >),
                Instruction::Gte => binary_op!(self, >=),
                Instruction::Sqrt => todo!(),
                Instruction::Pow => todo!(),
                Instruction::If => todo!(),
            }
        }

        Ok(self.stack.pop().unwrap())
    }
}
