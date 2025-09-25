// #![no_std]
// extern crate alloc;

mod builtin;
pub mod config;
pub mod environment;
pub mod future;

use std::{
    collections::{BTreeMap, VecDeque},
    hash::{DefaultHasher, Hash, Hasher},
    pin::{Pin, pin},
    rc::Rc,
    task::{Context, Poll, Waker},
};

use misp_common::intern::StringId;
use misp_interner::Interner;
use misp_num::decimal::Decimal;

use crate::{
    builtin::{
        combinatorics::{builtin_combinations, builtin_factorial, builtin_permutations},
        control::builtin_if,
        func::{builtin_func, builtin_lambda},
        math::{
            builtin_abs, builtin_add, builtin_divide, builtin_equal, builtin_gt, builtin_gte,
            builtin_lt, builtin_lte, builtin_max, builtin_min, builtin_minus, builtin_multiply,
            builtin_not_equal, builtin_pow, builtin_sqrt, builtin_summate,
        },
    },
    config::Config,
    environment::{Environment, Scope},
    future::{EvalFuture, EvalFutureContext},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown Symbol")]
    UnknownSymbol,
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

pub type NativeMispFuture = Pin<Box<dyn Future<Output = Result<Value, Error>> + 'static>>;
pub type RawNativeMispFunction = fn(*mut Executor) -> NativeMispFuture;

// This just wraps the raw function with a linked unique usized id.
pub type NativeMispFunction = (usize, RawNativeMispFunction);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeMispFunction {
    pub id: usize,
    pub params: Vec<StringId>,
    pub body: Rc<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lambda {
    pub params: Vec<StringId>,
    pub body: Rc<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Function {
    Native(NativeMispFunction),
    Runtime(RuntimeMispFunction),
    Lambda(Lambda),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Atom(StringId),
    List(Vec<Value>),
    Decimal(Decimal),
    Function(Function),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoKey {
    pub id: usize,
    pub args_hash: u64,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Push(Value),
    Store(StringId),
    Load(StringId),
    Call(usize),
    PushScope,
    PushDefinedScope(Scope),
    PopScope,
    Resume(usize),
    Await(usize),
    Marker(usize),
    MemoCheck { id: usize, params: Vec<StringId> },
    MemoStore { id: usize, params: Vec<StringId> },
}

pub struct Injector<'a> {
    instructions: &'a mut VecDeque<Instruction>,
    index: usize,
}

impl<'a> Injector<'a> {
    pub fn new(instructions: &'a mut VecDeque<Instruction>) -> Self {
        Self {
            instructions,
            index: 0,
        }
    }

    pub fn inject(&mut self, instruction: Instruction) {
        self.instructions.insert(self.index, instruction);
        self.index += 1;
    }
}

pub struct Executor {
    pub config: Config,
    pub env: Environment,
    pub instructions: VecDeque<Instruction>,
    pub stack: Vec<Value>,

    pub memos: BTreeMap<MemoKey, Value>,
    pub next_function_id: usize,

    pub waker: &'static Waker,
    pub futures: BTreeMap<usize, EvalFutureContext>,
    pub native_futures: BTreeMap<usize, NativeMispFuture>,
    pub next_future_id: usize,
}

impl Default for Executor {
    fn default() -> Self {
        Self {
            config: Config::default(),
            env: Environment::default(),
            instructions: VecDeque::default(),
            stack: Vec::default(),
            next_function_id: 0,
            memos: BTreeMap::default(),
            waker: Waker::noop(),
            futures: BTreeMap::default(),
            native_futures: BTreeMap::default(),
            next_future_id: 0,
        }
    }
}

impl Executor {
    pub fn initalize_env(&mut self, interner: &mut Interner) {
        let env = &mut self.env;

        env.set_prev(Value::Decimal(Decimal::ZERO), interner);

        env.load_constants(interner);

        // Functions
        env.define_native_function("func", builtin_func, interner);
        env.define_native_function("lambda", builtin_lambda, interner);

        // Control Functions
        env.define_native_function("if", builtin_if, interner);

        // Math Functions
        env.define_native_function("+", builtin_add, interner);
        env.define_native_function("-", builtin_minus, interner);
        env.define_native_function("*", builtin_multiply, interner);
        env.define_native_function("/", builtin_divide, interner);
        // env.define_native_function("%", builtin_mod, interner);
        env.define_native_function("==", builtin_equal, interner);
        env.define_native_function("!=", builtin_not_equal, interner);
        env.define_native_function("<", builtin_lt, interner);
        env.define_native_function("<=", builtin_lte, interner);
        env.define_native_function(">", builtin_gt, interner);
        env.define_native_function(">=", builtin_gte, interner);
        env.define_native_function("abs", builtin_abs, interner);
        env.define_native_function("min", builtin_min, interner);
        env.define_native_function("max", builtin_max, interner);
        env.define_native_function("pow", builtin_pow, interner);
        env.define_native_function("sqrt", builtin_sqrt, interner);
        env.define_native_function("summate", builtin_summate, interner);

        // Combinatorics
        env.define_native_function("factorial", builtin_factorial, interner);
        env.define_native_function("combinations", builtin_combinations, interner);
        env.define_native_function("permutations", builtin_permutations, interner);

        // Trig Functions
        // env.define_native_function("sin", builtin_sin);
        // env.define_native_function("cos", builtin_cos);
        // env.define_native_function("tan", builtin_tan);
        // env.define_native_function("asin", builtin_asin);
        // env.define_native_function("acos", builtin_acos);
        // env.define_native_function("atan", builtin_atan);
    }

    pub fn compile(value: Value, injector: &mut Injector) -> Result<(), Error> {
        match value {
            Value::Atom(atom) => injector.inject(Instruction::Load(atom)),
            Value::Decimal(_) | Value::Function(_) => {
                injector.inject(Instruction::Push(value));
            }
            Value::List(values) => {
                let arity = values.len() - 1;
                let mut iter = values.into_iter();

                let Value::Atom(name) = iter.next().unwrap() else {
                    return Err(Error::InvalidType);
                };

                for param in iter {
                    injector.inject(Instruction::Push(param));
                }

                injector.inject(Instruction::Load(name));
                injector.inject(Instruction::Call(arity));
            }
        }

        Ok(())
    }

    pub fn compile_function(&mut self, function: &Function) -> Result<(), Error> {
        match function {
            Function::Native((_, f)) => {
                let native_future = f(self);

                let future_id = self.next_future_id;
                self.next_future_id += 1;

                self.native_futures.insert(future_id, native_future);

                let mut injector = Injector::new(&mut self.instructions);
                injector.inject(Instruction::Await(future_id));
            }
            Function::Runtime(f) => {
                // arity_check!(self, "<func>", f.params.len());

                let mut injector = Injector {
                    instructions: &mut self.instructions,
                    index: 0,
                };

                injector.inject(Instruction::PushScope);

                // Reverse order ensures the correct value->name binding here.
                for param in f.params.iter().copied().rev() {
                    Self::compile(self.stack.pop().unwrap(), &mut injector)?;
                    injector.inject(Instruction::Store(param));
                }

                injector.inject(Instruction::MemoCheck {
                    id: f.id,
                    params: f.params.clone(),
                });

                let body = (*f.body).clone();
                Self::compile(body, &mut injector)?;

                injector.inject(Instruction::MemoStore {
                    id: f.id,
                    params: f.params.clone(),
                });

                injector.inject(Instruction::Marker(f.id));
                injector.inject(Instruction::PopScope);
            }
            Function::Lambda(l) => {
                // arity_check!(self, "<lambda>", l.params.len());

                let mut injector = Injector {
                    instructions: &mut self.instructions,
                    index: 0,
                };

                injector.inject(Instruction::PushScope);

                // Reverse order ensures the correct value->name binding here.
                for param in l.params.iter().copied().rev() {
                    Self::compile(self.stack.pop().unwrap(), &mut injector)?;
                    injector.inject(Instruction::Store(param));
                }

                let body = (*l.body).clone();
                Self::compile(body, &mut injector)?;
                injector.inject(Instruction::PopScope);
            }
        }

        Ok(())
    }

    fn execute_instruction(self: &mut Executor, instruction: Instruction) -> Result<(), Error> {
        // eprintln!("Current Instruction: {instruction:?}");
        // eprintln!("Instructions: {:?}", self.instructions);
        // eprintln!("Memos: {:?}", self.memos);
        // eprintln!("Stack: {:?}", self.stack);
        // eprintln!("Futures: {:?}", self.futures);
        // eprintln!("Native Futures: {:?}", self.native_futures.keys());
        // eprintln!();
        // std::thread::sleep(std::time::Duration::from_secs(1));

        match instruction {
            Instruction::Push(value) => {
                self.stack.push(value);
            }
            Instruction::Store(name) => {
                self.env.set(name, self.stack.pop().unwrap());
            }
            Instruction::Load(name) => {
                let value = self.env.get(name).ok_or(Error::UnknownSymbol)?;
                self.stack.push(value.clone());
            }
            Instruction::Call(arity) => {
                let arity_decimal = Decimal::from(arity as u64);

                let value = self.stack.pop().unwrap();

                let Value::Function(f) = value else {
                    return Err(Error::FunctionNotFound);
                };

                self.stack.push(Value::Decimal(arity_decimal));
                self.compile_function(&f)?;
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
            Instruction::Marker(_) => {}
            Instruction::MemoCheck { id, params } => {
                let mut hasher = DefaultHasher::default();
                for param in params.into_iter() {
                    let value = self.env.get(param).unwrap();
                    value.hash(&mut hasher);
                }

                let key = MemoKey {
                    id,
                    args_hash: hasher.finish(),
                };

                if let Some(value) = self.memos.get(&key) {
                    // eprintln!("Cache Hit! {key:?} -> {value:?}");
                    self.stack.push(value.clone());

                    while let Some(instruction) = self.instructions.pop_front() {
                        if matches!(instruction, Instruction::Marker(tag) if tag == id) {
                            break;
                        }
                    }
                }
            }
            Instruction::MemoStore { id, params } => {
                let mut hasher = DefaultHasher::default();
                for param in params.into_iter() {
                    let value = self.env.get(param).unwrap();
                    value.hash(&mut hasher);
                }

                let key = MemoKey {
                    id,
                    args_hash: hasher.finish(),
                };

                let value = self.stack.last().unwrap();
                self.memos.insert(key, value.clone());
            }
            Instruction::Resume(id) => {
                let value = self.stack.pop().ok_or(Error::EmptyStack)?;

                if let Some(ctx) = self.futures.get_mut(&id) {
                    ctx.result = Some(Ok(value));

                    if let Some(waker) = ctx.waker.take() {
                        waker.wake();
                    }
                }
            }
            Instruction::Await(id) => {
                let mut context = Context::from_waker(self.waker);

                let future = self
                    .native_futures
                    .get_mut(&id)
                    .expect("Native function doesnt exist");

                match future.as_mut().poll(&mut context) {
                    Poll::Ready(result) => {
                        let value = result.unwrap();
                        self.stack.push(value);
                        self.native_futures.remove(&id);
                    }
                    Poll::Pending => {
                        self.instructions.insert(1, Instruction::Await(id));
                    }
                }
            }
        }

        Ok(())
    }

    async fn run_function(&mut self, func: Function, args: Vec<Value>) -> Result<Value, Error> {
        let future_id = self.next_future_id;
        self.next_future_id += 1;

        let mut injector = Injector::new(&mut self.instructions);
        let arity = args.len();
        for arg in args {
            injector.inject(Instruction::Push(arg));
        }

        injector.inject(Instruction::Push(Value::Function(func)));
        injector.inject(Instruction::Call(arity));
        injector.inject(Instruction::Resume(future_id));

        self.futures.insert(future_id, EvalFutureContext::default());

        EvalFuture {
            id: future_id,
            executor: self as *mut Executor,
        }
        .await
    }

    pub fn eval(&mut self, expr: Value) -> EvalFuture {
        let future_id = self.next_future_id;
        self.next_future_id += 1;

        let mut injector = Injector::new(&mut self.instructions);
        Self::compile(expr, &mut injector).unwrap();
        injector.inject(Instruction::Resume(future_id));

        self.futures.insert(future_id, EvalFutureContext::default());

        EvalFuture {
            id: future_id,
            executor: self as *mut Executor,
        }
    }

    pub fn execute(&mut self, value: Value) -> Result<Value, Error> {
        self.next_future_id = 0;
        self.next_function_id = 0;

        let future = self.eval(value);
        let mut main_future = pin!(future);
        let mut context = Context::from_waker(self.waker);

        loop {
            while let Some(instruction) = self.instructions.pop_front() {
                self.execute_instruction(instruction)?;
            }

            match main_future.as_mut().poll(&mut context) {
                Poll::Ready(result) => {
                    let result = result.unwrap();
                    return Ok(result);
                }
                Poll::Pending => continue,
            }
        }
    }
}
