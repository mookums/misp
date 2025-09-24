#![no_std]

mod builtin;
pub mod config;
pub mod environment;
pub mod future;

extern crate alloc;
use alloc::boxed::Box;

use core::{
    hash::SipHasher,
    pin::{Pin, pin},
    task::{Context, Poll, Waker},
};

use heapless::{Deque, String, Vec, index_map::FnvIndexMap};
use misp_common::intern::{StringId, ValueId};
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

#[derive(Debug, Clone, Hash)]
pub struct Lambda<const MAX_STR: usize, const MAX_LIST: usize> {
    pub params: Vec<String<MAX_STR>, MAX_LIST>,
    pub body: ValueId,
}

type NativeMispFuture<
    const MAX_STR: usize,
    const MAX_TOKENS: usize,
    const MAX_LIST: usize,
    const MAX_INTERN: usize,
    const MAX_INSTRUCTIONS: usize,
    const MAX_STACK: usize,
    const MAX_MEMOS: usize,
    const MAX_FUTURES: usize,
> = Pin<
    Box<
        dyn Future<
                Output = Result<
                    Value<
                        MAX_STR,
                        MAX_TOKENS,
                        MAX_LIST,
                        MAX_INTERN,
                        MAX_INSTRUCTIONS,
                        MAX_STACK,
                        MAX_MEMOS,
                        MAX_FUTURES,
                    >,
                    (),
                >,
            > + 'static,
    >,
>;

type NativeMispFunction<
    const MAX_STR: usize,
    const MAX_TOKENS: usize,
    const MAX_LIST: usize,
    const MAX_INTERN: usize,
    const MAX_INSTRUCTIONS: usize,
    const MAX_STACK: usize,
    const MAX_MEMOS: usize,
    const MAX_FUTURES: usize,
> = fn(
    *mut Executor<
        MAX_STR,
        MAX_TOKENS,
        MAX_LIST,
        MAX_INTERN,
        MAX_INSTRUCTIONS,
        MAX_STACK,
        MAX_MEMOS,
        MAX_FUTURES,
    >,
) -> NativeMispFuture<
    MAX_STR,
    MAX_TOKENS,
    MAX_LIST,
    MAX_INTERN,
    MAX_INSTRUCTIONS,
    MAX_STACK,
    MAX_MEMOS,
    MAX_FUTURES,
>;

#[derive(Debug, Clone, Hash)]
pub struct RuntimeMispFunction<const MAX_LIST: usize> {
    pub id: usize,
    pub params: Vec<StringId, MAX_LIST>,
    pub body: ValueId,
}

#[derive(Debug, Clone, Hash)]
pub enum Function<
    const MAX_STR: usize,
    const MAX_TOKENS: usize,
    const MAX_LIST: usize,
    const MAX_INTERN: usize,
    const MAX_INSTRUCTIONS: usize,
    const MAX_STACK: usize,
    const MAX_MEMOS: usize,
    const MAX_FUTURES: usize,
> {
    Native(
        NativeMispFunction<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
    ),
    Runtime(RuntimeMispFunction<MAX_LIST>),
    Lambda(Lambda<MAX_STR, MAX_LIST>),
}

#[derive(Debug, Clone, Hash)]
pub enum Value<
    const MAX_STR: usize,
    const MAX_TOKENS: usize,
    const MAX_LIST: usize,
    const MAX_INTERN: usize,
    const MAX_INSTRUCTIONS: usize,
    const MAX_STACK: usize,
    const MAX_MEMOS: usize,
    const MAX_FUTURES: usize,
> {
    Atom(StringId),
    List(Vec<ValueId, MAX_LIST>),
    Decimal(Decimal),
    Function(
        Function<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
    ),
}

// impl From<SExpr<_>> for Value<_> {
//     fn from(value: SExpr<_>) -> Self {
//         match value {
//             SExpr::Atom(str) => Value::Atom(str),
//             SExpr::List(sexprs) => Value::List(sexprs.into_iter().map(|e| e.into()).collect()),
//             SExpr::Decimal(d) => Value::Decimal(d),
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoKey {
    pub id: usize,
    pub args_hash: u64,
}

#[derive(Debug, Clone)]
pub enum Instruction<const MAX_LIST: usize> {
    Push(ValueId),
    Store(StringId),
    Load(StringId),
    Call(usize),
    PushScope,
    PushDefinedScope(Scope),
    PopScope,
    Resume(usize),
    Await(usize),
    Marker(usize),
    MemoCheck {
        id: usize,
        params: Vec<StringId, MAX_LIST>,
    },
    MemoStore {
        id: usize,
        params: Vec<StringId, MAX_LIST>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid Function Call")]
    FunctionCall,
    #[error("Wrong arity: expected {expected}, got {actual}")]
    FunctionArity { expected: usize, actual: usize },
    #[error("Function not found")]
    FunctionNotFound,
    #[error("Invalid type")]
    InvalidType,
    #[error("Empty Stack")]
    EmptyStack,
}

pub struct Injector<'a, const MAX_LIST: usize, const MAX_INSTRUCTIONS: usize> {
    instructions: &'a mut Deque<Instruction<MAX_LIST>, MAX_INSTRUCTIONS>,
    index: usize,
}

impl<'a, const MAX_LIST: usize, const MAX_INSTRUCTIONS: usize>
    Injector<'a, MAX_LIST, MAX_INSTRUCTIONS>
{
    pub fn new(instructions: &'a mut Deque<Instruction<MAX_LIST>, MAX_INSTRUCTIONS>) -> Self {
        Self {
            instructions,
            index: 0,
        }
    }

    pub fn inject(&mut self, instruction: Instruction<MAX_LIST>) {
        self.instructions.insert(self.index, instruction);
        self.index += 1;
    }
}

pub struct Executor<
    const MAX_STR: usize,
    const MAX_TOKENS: usize,
    const MAX_LIST: usize,
    const MAX_INTERN: usize,
    const MAX_INSTRUCTIONS: usize,
    const MAX_STACK: usize,
    const MAX_MEMOS: usize,
    const MAX_FUTURES: usize,
> {
    pub config: Config,
    pub env: Environment,
    pub instructions: Deque<Instruction<MAX_LIST>, MAX_INSTRUCTIONS>,
    pub stack: Vec<
        Value<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
        MAX_STACK,
    >,

    pub memos: FnvIndexMap<
        MemoKey,
        Value<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
        MAX_MEMOS,
    >,
    pub next_function_id: usize,

    pub waker: &'static Waker,
    pub futures: FnvIndexMap<
        usize,
        EvalFutureContext<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
        MAX_FUTURES,
    >,
    pub native_futures: FnvIndexMap<
        usize,
        NativeMispFuture<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
        MAX_FUTURES,
    >,
    pub next_future_id: usize,
}

impl<
    const MAX_STR: usize,
    const MAX_TOKENS: usize,
    const MAX_LIST: usize,
    const MAX_INTERN: usize,
    const MAX_INSTRUCTIONS: usize,
    const MAX_STACK: usize,
    const MAX_MEMOS: usize,
    const MAX_FUTURES: usize,
> Default
    for Executor<
        MAX_STR,
        MAX_TOKENS,
        MAX_LIST,
        MAX_INTERN,
        MAX_INSTRUCTIONS,
        MAX_STACK,
        MAX_MEMOS,
        MAX_FUTURES,
    >
{
    fn default() -> Self {
        let config = Config::default();
        let mut env = Environment::default();

        env.push_scope();
        env.set_prev(Value::Decimal(Decimal::ZERO));

        env.load_constants();

        // Functions
        env.define_native_function("func", builtin_func);
        env.define_native_function("lambda", builtin_lambda);

        // Control Functions
        env.define_native_function("if", builtin_if);

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
        env.define_native_function("min", builtin_min);
        env.define_native_function("max", builtin_max);
        env.define_native_function("pow", builtin_pow);
        env.define_native_function("sqrt", builtin_sqrt);
        env.define_native_function("summate", builtin_summate);

        // Combinatorics
        env.define_native_function("factorial", builtin_factorial);
        env.define_native_function("combinations", builtin_combinations);
        env.define_native_function("permutations", builtin_permutations);

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
            instructions: Deque::default(),
            stack: Vec::default(),
            next_function_id: 0,
            memos: FnvIndexMap::default(),
            waker: Waker::noop(),
            futures: FnvIndexMap::default(),
            native_futures: FnvIndexMap::default(),
            next_future_id: 0,
        }
    }
}

impl<
    const MAX_STR: usize,
    const MAX_TOKENS: usize,
    const MAX_LIST: usize,
    const MAX_INTERN: usize,
    const MAX_INSTRUCTIONS: usize,
    const MAX_STACK: usize,
    const MAX_MEMOS: usize,
    const MAX_FUTURES: usize,
>
    Executor<
        MAX_STR,
        MAX_TOKENS,
        MAX_LIST,
        MAX_INTERN,
        MAX_INSTRUCTIONS,
        MAX_STACK,
        MAX_MEMOS,
        MAX_FUTURES,
    >
{
    pub fn compile(
        value: Value<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
        injector: &mut Injector<'_, MAX_LIST, MAX_INSTRUCTIONS>,
    ) -> Result<(), Error> {
        match value {
            Value::Atom(atom) => injector.inject(Instruction::Load(atom)),
            Value::Decimal(_) | Value::Function(_) => {
                injector.inject(Instruction::Push(value));
            }
            Value::List(mut values) => {
                let arity = values.len() - 1;

                let mut drain = values.drain(0..values.len());
                let Value::Atom(name) = drain.next().unwrap() else {
                    return Err(Error::InvalidType);
                };

                for param in drain {
                    injector.inject(Instruction::Push(param));
                }

                injector.inject(Instruction::Load(name));
                injector.inject(Instruction::Call(arity));
            }
        }

        Ok(())
    }

    pub fn compile_function(
        &mut self,
        function: Function<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
    ) -> Result<(), Error> {
        match function {
            Function::Native(f) => {
                let native_future = f(self);

                let future_id = self.next_future_id;
                self.next_future_id += 1;

                self.native_futures.insert(future_id, native_future);

                let mut injector = Injector::new(&mut self.instructions);
                injector.inject(Instruction::Await(future_id));
            }
            Function::Runtime(f) => {
                arity_check!(self, f.params.len());

                let mut injector = Injector {
                    instructions: &mut self.instructions,
                    index: 0,
                };

                injector.inject(Instruction::PushScope);

                // Reverse order ensures the correct value->name binding here.
                for param in f.params.iter().rev() {
                    Self::compile(self.stack.pop().unwrap(), &mut injector)?;
                    injector.inject(Instruction::Store(param.clone()));
                }

                injector.inject(Instruction::MemoCheck {
                    id: f.id,
                    params: f.params.clone(),
                });

                let body = (*f.body).clone();
                Self::compile(body, &mut injector)?;

                injector.inject(Instruction::MemoStore {
                    id: f.id,
                    params: f.params,
                });

                injector.inject(Instruction::Marker(f.id));
                injector.inject(Instruction::PopScope);
            }
            Function::Lambda(l) => {
                arity_check!(self, l.params.len());

                let mut injector = Injector {
                    instructions: &mut self.instructions,
                    index: 0,
                };

                injector.inject(Instruction::PushScope);

                // Reverse order ensures the correct value->name binding here.
                for param in l.params.into_iter().rev() {
                    Self::compile(self.stack.pop().unwrap(), &mut injector)?;
                    injector.inject(Instruction::Store(param));
                }

                Self::compile(*l.body, &mut injector)?;
                injector.inject(Instruction::PopScope);
            }
        }

        Ok(())
    }

    fn execute_instruction(&mut self, instruction: Instruction<MAX_LIST>) -> Result<(), Error> {
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
                let value = self.env.get(&name).ok_or(Error::UnknownSymbol(name))?;
                self.stack.push(value.clone());
            }
            Instruction::Call(arity) => {
                let func = self.stack.pop().unwrap();

                match func {
                    Value::Function(f) => {
                        let arity_decimal = Decimal::from(arity as u64);
                        self.stack.push(Value::Decimal(arity_decimal));
                        self.compile_function(f)?;
                    }
                    _ => return Err(Error::FunctionNotFound),
                }
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
                let mut hasher = SipHasher::default();
                for param in params.iter() {
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
                let mut hasher = SipHasher::default();
                for param in params.iter() {
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
                        self.stack.push(result?);
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

    async fn run_function(
        &mut self,
        func: Function<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
        args: Vec<
            Value<
                MAX_STR,
                MAX_TOKENS,
                MAX_LIST,
                MAX_INTERN,
                MAX_INSTRUCTIONS,
                MAX_STACK,
                MAX_MEMOS,
                MAX_FUTURES,
            >,
            MAX_LIST,
        >,
    ) -> Result<
        Value<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
        Error,
    > {
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

    pub fn eval(
        &mut self,
        expr: Value<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
    ) -> EvalFuture<
        MAX_STR,
        MAX_TOKENS,
        MAX_LIST,
        MAX_INTERN,
        MAX_INSTRUCTIONS,
        MAX_STACK,
        MAX_MEMOS,
        MAX_FUTURES,
    > {
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

    pub fn execute(
        &mut self,
        value: Value<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
    ) -> Result<
        Value<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
        Error,
    > {
        self.instructions.clear();
        self.stack.clear();
        self.memos.clear();

        let future = self.eval(value);
        let mut main_future = pin!(future);
        let mut context = Context::from_waker(self.waker);

        loop {
            while let Some(instruction) = self.instructions.pop_front() {
                self.execute_instruction(instruction)?;
            }

            match main_future.as_mut().poll(&mut context) {
                Poll::Ready(result) => {
                    let result = result?;
                    self.env.set_prev(result.clone());
                    return Ok(result);
                }
                Poll::Pending => continue,
            }
        }
    }
}
