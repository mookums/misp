#![no_std]
extern crate alloc;

pub mod cas;
pub mod config;
pub mod environment;
pub mod instruction;
pub mod value;

use alloc::{string::String, vec::Vec};
use compact_str::CompactString;
use hashbrown::{HashMap, HashSet};

use core::hash::Hash;

use misp_num::decimal::Decimal;

use crate::{
    cas::CasOperation,
    config::Config,
    environment::Environment,
    instruction::Instruction,
    value::{Function, RuntimeMispFunction, Value},
};

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
    fn compile_value(&mut self, value: Value, is_tail: bool) -> Result<(), Error> {
        match value {
            Value::Atom(atom) => self.instructions.push(Instruction::Load(atom)),
            Value::Decimal(_) | Value::Function(_) | Value::Symbol(_) => {
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
                        "if" => {
                            if values.len() != 4 {
                                panic!("if arity");
                            }

                            let mut iter = values.into_iter();
                            iter.next();

                            let condition = iter.next().unwrap();
                            let then_expr = iter.next().unwrap();
                            let else_expr = iter.next().unwrap();

                            self.compile_value(condition, false)?;

                            let jz_to_else = self.instructions.len();
                            self.instructions.push(Instruction::Placeholder);

                            self.compile_value(then_expr, is_tail)?;
                            let jmp_past_else = self.instructions.len();
                            self.instructions.push(Instruction::Placeholder);

                            let else_start = self.instructions.len();
                            self.compile_value(else_expr, is_tail)?;

                            let end = self.instructions.len();
                            self.instructions[jz_to_else] = Instruction::Jz(else_start);
                            self.instructions[jmp_past_else] = Instruction::Jmp(end);
                        }
                        "simplify" => {
                            let arg = values[1].clone();

                            self.instructions.push(Instruction::Push(arg));
                            self.instructions
                                .push(Instruction::Cas(CasOperation::Simplify));
                        }
                        "+" => variadic_instruction!(self, values, Add),
                        "-" => variadic_instruction!(self, values, Sub),
                        "*" => variadic_instruction!(self, values, Mult),
                        "/" => variadic_instruction!(self, values, Div),
                        "==" => variadic_instruction!(self, values, Eq),
                        "!=" => variadic_instruction!(self, values, Neq),
                        ">" => variadic_instruction!(self, values, Gt),
                        ">=" => variadic_instruction!(self, values, Gte),
                        "<" => variadic_instruction!(self, values, Lt),
                        "<=" => variadic_instruction!(self, values, Lte),
                        _ => {
                            let arity = (values.len() - 1) as u64;

                            let mut iter = values.into_iter();
                            let Value::Atom(name) = iter.next().unwrap() else {
                                return Err(Error::InvalidType);
                            };

                            for param in iter {
                                self.compile_value(param.clone(), false)?;
                            }

                            let Value::Function(func) = self.env.get(&name) else {
                                return Err(Error::InvalidType);
                            };

                            self.instructions
                                .push(Instruction::Push(Value::Decimal(arity.into())));

                            if is_tail {
                                self.instructions.push(Instruction::TailCall(func.clone()));
                            } else {
                                self.instructions.push(Instruction::Call(func.clone()));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn compile_self(&mut self, value: Value, is_tail: bool) -> Result<usize, Error> {
        self.discover_and_compile_rt_funcs(&value)?;
        let offset = self.instructions.len();
        self.compile_value(value, is_tail)?;
        Ok(offset)
    }

    pub fn compile(&mut self, value: Value) -> Result<(usize, Vec<Instruction>), Error> {
        self.pc = 0;
        self.instructions.clear();
        self.frames.clear();
        self.function_location.clear();

        let pc = self.compile_self(value, false)?;
        Ok((pc, self.instructions.clone()))
    }

    fn discover_rt_funcs(
        &mut self,
        value: &Value,
        discovered: &mut HashSet<RuntimeMispFunction>,
    ) -> Result<(), Error> {
        match value {
            Value::Atom(name) => {
                let Value::Function(Function::Runtime(rt)) = self.env.get(name) else {
                    return Ok(());
                };

                if discovered.insert(rt.clone()) {
                    self.discover_rt_funcs(&rt.body, discovered)?;
                }
            }
            Value::List(values) => {
                for subvalue in values {
                    self.discover_rt_funcs(subvalue, discovered)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn discover_and_compile_rt_funcs(&mut self, value: &Value) -> Result<(), Error> {
        let mut discovered = HashSet::new();
        self.discover_rt_funcs(value, &mut discovered)?;

        for rt in discovered {
            if self.function_location.get(&rt.id).is_some() {
                return Ok(());
            }

            let start_location = self.instructions.len();

            for param in rt.params.iter().rev() {
                self.instructions.push(Instruction::Store(param.clone()));
            }

            self.function_location.insert(rt.id, start_location);

            self.compile_value((*rt.body).clone(), true)?;

            self.instructions.push(Instruction::Return);
        }

        Ok(())
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
        //     eprintln!("Locations: {:?}", self.function_location);
        //     eprintln!();
        //     std::thread::sleep(std::time::Duration::from_secs(1));
        // }

        match instruction {
            Instruction::Placeholder => unreachable!(),
            Instruction::Push(value) => {
                self.stack.push(value);
            }
            Instruction::Store(name) => {
                self.env.set(name, self.stack.pop().unwrap());
            }
            Instruction::Load(name) => {
                let value = self.env.get(&name);
                self.stack.push(value.clone());
            }
            Instruction::Call(func) => match func {
                Function::Runtime(rt) => {
                    let location = self
                        .function_location
                        .get(&rt.id)
                        .expect("Function must have a location");

                    arity_check!(self, "<func>", rt.params.len());

                    self.frames.push(CallFrame {
                        return_pc: self.pc,
                        stack_base: self.stack.len(),
                    });

                    self.env.push_scope();
                    self.pc = *location;
                }
                Function::Lambda(_) => todo!(),
            },
            Instruction::TailCall(func) => match func {
                Function::Runtime(rt) => {
                    let location = self
                        .function_location
                        .get(&rt.id)
                        .expect("Function must have a location");

                    let frame = self.frames.last().expect("Must have a last frame");

                    let Value::Decimal(arity) = self.stack.pop().unwrap() else {
                        return Err(Error::InvalidType);
                    };

                    let new_args: Vec<Value> = self
                        .stack
                        .split_off(self.stack.len() - arity.to_u128() as usize);

                    self.stack.truncate(frame.stack_base);

                    self.stack.extend(new_args);
                    self.pc = *location;
                }
                Function::Lambda(_) => todo!(),
            },
            Instruction::Return => {
                let frame = self.frames.pop().expect("Can't return without frame");
                self.pc = frame.return_pc;
                self.stack.truncate(frame.stack_base);
                self.env.pop_scope();
            }
            Instruction::Jmp(pc) => {
                self.pc = pc;
            }
            Instruction::Jz(pc) => {
                let Value::Decimal(n) = self.stack.pop().ok_or(Error::EmptyStack)? else {
                    return Err(Error::InvalidType);
                };

                if n == Decimal::ZERO {
                    self.pc = pc;
                }
            }
            Instruction::Cas(op) => {
                let result = match op {
                    CasOperation::Simplify => cas::simplify::builtin_simplify(self)?,
                };

                self.stack.push(result);
            }
            Instruction::Add => variadic_op!(self, +),
            Instruction::Sub => variadic_op!(self, -),
            Instruction::Mult => variadic_op!(self, *),
            Instruction::Div => variadic_op!(self, /),
            Instruction::Eq => binary_comparison!(self, ==),
            Instruction::Neq => binary_comparison!(self, !=),
            Instruction::Gt => binary_comparison!(self, >),
            Instruction::Gte => binary_comparison!(self, >=),
            Instruction::Lt => binary_comparison!(self, <),
            Instruction::Lte => binary_comparison!(self, <=),
        }

        Ok(())
    }

    pub fn execute(&mut self, value: Value) -> Result<Value, Error> {
        self.pc = 0;
        self.instructions.clear();
        self.frames.clear();
        self.function_location.clear();

        match value {
            Value::Atom(name) => {
                let val = self.env.get(&name);
                let result = val.clone();
                self.env.set_prev(result.clone());
                Ok(result)
            }
            Value::Decimal(_) => {
                self.env.set_prev(value.clone());
                Ok(value)
            }
            _ => {
                let pc = self.compile_self(value, false)?;
                self.pc = pc;

                while self.pc < self.instructions.len() {
                    let current_pc = self.pc;
                    self.pc += 1;
                    let instr = self.instructions[current_pc].clone();
                    self.execute_instruction(instr)?;
                }

                let value = self.stack.pop().expect("Execute must return a value");
                self.env.set_prev(value.clone());
                Ok(value)
            }
        }
    }
}

#[macro_export]
macro_rules! arity_check {
    ($e:ident, $name:expr, $expected:expr) => {
        let Value::Decimal(arity) = $e.stack.pop().unwrap() else {
            return Err(Error::InvalidType);
        };

        let arity_int = arity.to_u128() as usize;
        if arity_int != $expected {
            use alloc::string::ToString;

            return Err(Error::FunctionArity {
                name: $name.to_string(),
                expected: $expected,
                actual: arity_int,
            });
        }
    };
}
