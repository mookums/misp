#![no_std]
extern crate alloc;

mod builtin;
pub mod config;
pub mod environment;
pub mod instruction;

use alloc::{boxed::Box, rc::Rc, string::String, vec::Vec};
use compact_str::CompactString;
use hashbrown::HashMap;

use core::hash::Hash;

use misp_num::decimal::Decimal;
use misp_parser::SExpr;

use crate::{
    // builtin::{
    //     combinatorics::{builtin_combinations, builtin_factorial, builtin_permutations},
    //     control::builtin_if,
    //     func::{builtin_func, builtin_lambda},
    //     math::{
    //         builtin_abs, builtin_add, builtin_divide, builtin_equal, builtin_gt, builtin_gte,
    //         builtin_lt, builtin_lte, builtin_max, builtin_min, builtin_minus, builtin_multiply,
    //         builtin_not_equal, builtin_pow, builtin_sqrt, builtin_summate,
    //     },
    // },
    builtin::math::{
        builtin_abs, builtin_add, builtin_divide, builtin_equal, builtin_gt, builtin_gte,
        builtin_lt, builtin_lte, builtin_minus, builtin_multiply, builtin_not_equal, builtin_pow,
        builtin_sqrt,
    },
    config::Config,
    environment::Environment,
    instruction::Instruction,
};

#[derive(Debug, Clone, Hash)]
pub struct Lambda {
    pub params: Vec<CompactString>,
    pub body: Box<Value>,
}

type NativeMispFunction = fn(&mut Executor) -> Result<Value, Error>;

#[derive(Debug, Clone, Hash)]
pub struct RuntimeMispFunction {
    pub id: usize,
    pub params: Rc<Vec<CompactString>>,
    pub body: Rc<Value>,
}

#[derive(Debug, Clone, Hash)]
pub enum Function {
    Native(NativeMispFunction),
    Runtime(RuntimeMispFunction),
    Lambda(Lambda),
}

#[derive(Debug, Clone, Hash)]
pub enum Value {
    Atom(CompactString),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoKey {
    pub id: usize,
    pub args_hash: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown Symbol: {0}")]
    UnknownSymbol(CompactString),
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
    #[error("Invalid type")]
    InvalidType,
    #[error("Empty Stack")]
    EmptyStack,
}

#[derive(Debug)]
pub struct CallFrame {
    return_pc: usize,
    stack_base: usize,
}

pub struct Executor {
    pub config: Config,
    pub env: Environment,
    pub function_location: HashMap<usize, usize>,

    pub instructions: Vec<Instruction>,
    pub pc: usize,
    pub stack: Vec<Value>,
    pub frames: Vec<CallFrame>,

    pub memos: HashMap<MemoKey, Value>,
    pub next_function_id: usize,
}

impl Default for Executor {
    fn default() -> Self {
        let config = Config::default();
        let mut env = Environment::default();
        env.load_constants();

        // Functions
        // env.define_native_function("func", builtin_func);
        // env.define_native_function("lambda", builtin_lambda);

        // Control Functions
        // env.define_native_function("if", builtin_if);

        // Math Functions
        env.define_native_function("+", builtin_add);
        env.define_native_function("-", builtin_minus);
        env.define_native_function("*", builtin_multiply);
        env.define_native_function("/", builtin_divide);
        // env.define_native_function("%", builtin_mod);
        env.define_native_function("==", builtin_equal);
        env.define_native_function("!=", builtin_not_equal);
        env.define_native_function("<", builtin_lt);
        env.define_native_function("<=", builtin_lte);
        env.define_native_function(">", builtin_gt);
        env.define_native_function(">=", builtin_gte);
        env.define_native_function("abs", builtin_abs);
        // env.define_native_function("min", builtin_min);
        // env.define_native_function("max", builtin_max);
        env.define_native_function("pow", builtin_pow);
        env.define_native_function("sqrt", builtin_sqrt);
        // env.define_native_function("summate", builtin_summate);

        // Combinatorics
        // env.define_native_function("factorial", builtin_factorial);
        // env.define_native_function("combinations", builtin_combinations);
        // env.define_native_function("permutations", builtin_permutations);

        // Trig Functions
        // env.define_native_function("sin", builtin_sin);
        // env.define_native_function("cos", builtin_cos);
        // env.define_native_function("tan", builtin_tan);
        // env.define_native_function("asin", builtin_asin);
        // env.define_native_function("acos", builtin_acos);
        // env.define_native_function("atan", builtin_atan);

        env.push_scope();
        env.set_prev(Value::Decimal(Decimal::ZERO));

        Self {
            config,
            env,
            function_location: HashMap::default(),
            instructions: Vec::default(),
            pc: 0,
            stack: Vec::default(),
            next_function_id: 0,
            memos: HashMap::default(),
            frames: Vec::default(),
        }
    }
}

impl Executor {
    fn compile_self(&mut self, value: Value) -> Result<usize, Error> {
        self.compile_runtime_functions(&value)?;
        let offset = self.instructions.len();

        match value {
            Value::Atom(atom) => self.instructions.push(Instruction::Load(atom)),
            Value::Decimal(_) | Value::Function(_) => {
                self.instructions.push(Instruction::Push(value));
            }
            Value::List(values) => {
                if let Some(Value::Atom(name)) = values.first() {
                    match name.as_str() {
                        "func" => {
                            if values.len() != 4 {
                                panic!("func arity");
                            }

                            let mut iter = values.into_iter();
                            iter.next();

                            let Value::Atom(name) = iter.next().unwrap() else {
                                panic!("wrong func type")
                            };

                            let params = match iter.next().unwrap() {
                                Value::List(param_list) => param_list
                                    .into_iter()
                                    .map(|p| match p {
                                        Value::Atom(param) => Ok(param.clone()),
                                        _ => Err(Error::InvalidType),
                                    })
                                    .collect::<Result<Vec<_>, _>>()?,
                                _ => return Err(Error::InvalidType),
                            };

                            let body = iter.next().unwrap();

                            let function_id = self.next_function_id;
                            self.next_function_id += 1;

                            let rt_func = RuntimeMispFunction {
                                id: function_id,
                                params: params.into(),
                                body: body.into(),
                            };

                            let function = Value::Function(Function::Runtime(rt_func));
                            self.env.set_global(name, function.clone());

                            self.instructions.push(Instruction::Push(function));
                        }
                        _ => {
                            let arity = (values.len() - 1) as u64;

                            let mut iter = values.into_iter();
                            let Value::Atom(name) = iter.next().unwrap() else {
                                return Err(Error::InvalidType);
                            };

                            for param in iter {
                                self.compile_self(param.clone())?;
                            }

                            let Value::Function(func) =
                                self.env.get(&name).ok_or(Error::UnknownSymbol(name))?
                            else {
                                return Err(Error::InvalidType);
                            };

                            self.instructions
                                .push(Instruction::Push(Value::Decimal(arity.into())));

                            self.instructions.push(Instruction::Call(func.clone()));
                        }
                    }
                }
            }
        }

        Ok(offset)
    }

    fn execute_instruction(self: &mut Executor, instruction: Instruction) -> Result<(), Error> {
        // {
        //     extern crate std;
        //     use std::eprintln;
        //     eprintln!("Current Instruction: {instruction:?}");
        //     eprintln!("Current Program Counter: {}", self.pc - 1);
        //     eprintln!("Memos: {:?}", self.memos.keys());
        //     eprintln!("Stack: {:?}", self.stack);
        //     eprintln!("Frames: {:?}", self.frames);
        //     eprintln!("Scopes: {:?}", self.env.scopes.len());
        //     eprintln!();
        //     std::thread::sleep(std::time::Duration::from_secs(1));
        // }

        match instruction {
            Instruction::Push(value) => {
                self.stack.push(value);
            }
            Instruction::Store(name) => {
                self.env.set(name, self.stack.pop().unwrap());
            }
            Instruction::Load(name) => {
                let value = self.env.get(&name).ok_or(Error::UnknownSymbol(name))?;
                self.stack.push(value.clone());
            }
            Instruction::Call(func) => match func {
                Function::Native(f) => {
                    self.env.push_scope();
                    let value = f(self)?;
                    self.env.pop_scope();

                    self.stack.push(value);
                }
                Function::Runtime(rt) => {
                    let location = self
                        .function_location
                        .get(&rt.id)
                        .expect("Function must have a location");

                    let Value::Decimal(arity) = self.stack.pop().unwrap() else {
                        return Err(Error::InvalidType);
                    };

                    if arity.to_u128() as usize != rt.params.len() {
                        panic!("wrong arity in rt func");
                    }

                    self.frames.push(CallFrame {
                        return_pc: self.pc,
                        stack_base: self.stack.len(),
                    });

                    self.pc = *location;
                }
                Function::Lambda(lambda) => todo!(),
            },

            Instruction::Return => {
                let frame = self.frames.pop().expect("Can't return without frame");
                self.pc = frame.return_pc;
                self.stack.truncate(frame.stack_base);
            }
            Instruction::PushScope => {
                self.env.push_scope();
            }
            Instruction::PushDefinedScope(scope) => {
                self.env.push_given_scope(scope);
            }
            Instruction::PopScope => {
                self.env.pop_scope();
            }
        }

        Ok(())
    }

    fn compile_runtime_function(&mut self, rt: &RuntimeMispFunction) -> Result<(), Error> {
        if self.function_location.get(&rt.id).is_some() {
            return Ok(());
        }

        let start_location = self.instructions.len();

        self.instructions.push(Instruction::PushScope);

        for param in rt.params.iter().rev() {
            self.instructions.push(Instruction::Store(param.clone()));
        }

        self.compile_self((*rt.body).clone())?;

        self.instructions.push(Instruction::PopScope);
        self.instructions.push(Instruction::Return);

        self.function_location.insert(rt.id, start_location);

        Ok(())
    }

    fn compile_runtime_functions(&mut self, value: &Value) -> Result<(), Error> {
        match value {
            Value::Atom(name) => {
                let Some(Value::Function(Function::Runtime(rt))) = self.env.get(name).cloned()
                else {
                    return Ok(());
                };

                self.compile_runtime_function(&rt)?;
            }
            Value::List(values) => {
                for value in values {
                    self.compile_runtime_functions(value)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn compile(&mut self, value: Value) -> Result<(usize, Vec<Instruction>), Error> {
        let pc = self.compile_self(value)?;
        Ok((pc, self.instructions.clone()))
    }

    pub fn execute(&mut self, value: Value) -> Result<Value, Error> {
        self.pc = 0;
        self.instructions.clear();
        self.frames.clear();
        self.function_location.clear();

        match value {
            Value::Atom(name) => {
                if let Some(val) = self.env.get(&name) {
                    let result = val.clone();
                    self.env.set_prev(result.clone());
                    Ok(result)
                } else {
                    Err(Error::UnknownSymbol(name))
                }
            }
            Value::Decimal(_) => {
                self.env.set_prev(value.clone());
                Ok(value)
            }
            _ => {
                self.pc = self.compile_self(value)?;

                while self.pc < self.instructions.len() {
                    let current_pc = self.pc;
                    self.pc += 1;
                    let instr = self.instructions[current_pc].clone();
                    self.execute_instruction(instr)?;
                }

                let value = self.stack.pop().expect("Execute must return a value");
                Ok(value)
            }
        }
    }
}
