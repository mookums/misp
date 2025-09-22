mod builtin;
pub mod config;
pub mod environment;

use std::{
    collections::{BTreeMap, VecDeque},
    pin::{Pin, pin},
    task::{Context, Poll, Waker},
};

use misp_num::decimal::Decimal;
use misp_parser::SExpr;

use crate::{
    builtin::{
        control::builtin_if,
        func::builtin_func,
        math::{
            builtin_add, builtin_divide, builtin_equal, builtin_factorial, builtin_gt, builtin_gte,
            builtin_lt, builtin_lte, builtin_minus, builtin_multiply, builtin_not_equal,
            builtin_pow, builtin_sqrt,
        },
    },
    config::Config,
    environment::{Environment, Scope},
};

#[derive(Debug, Clone)]
pub struct Lambda {
    pub params: Vec<String>,
    pub body: Box<Value>,
    pub scope: Scope,
}

type NativeMispFuture = Pin<Box<dyn Future<Output = Result<Value, Error>> + 'static>>;
type NativeMispFunction = fn(*mut Executor) -> NativeMispFuture;

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
    PushScope,
    PushDefinedScope(Scope),
    PopScope,
    Resume(usize),
    Await(usize),
}

#[derive(Debug, Default)]
pub struct EvalFutureContext {
    result: Option<Result<Value, Error>>,
    waker: Option<Waker>,
}

pub struct EvalFuture {
    id: usize,
    executor: *mut Executor,
}

impl Future for EvalFuture {
    type Output = Result<Value, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let fut = self.get_mut();
        let executor = unsafe { &mut *fut.executor };

        if let Some(future_data) = executor.futures.get_mut(&fut.id) {
            if let Some(result) = future_data.result.take() {
                executor.futures.remove(&fut.id);
                Poll::Ready(result)
            } else {
                future_data.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        } else {
            panic!("Future {} not found", fut.id)
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
    #[error("Function not found")]
    FunctionNotFound,
    #[error("Invalid type")]
    InvalidType,
    #[error("Empty Stack")]
    EmptyStack,
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

    pub waker: &'static Waker,
    pub futures: BTreeMap<usize, EvalFutureContext>,
    pub native_futures: BTreeMap<usize, NativeMispFuture>,
    pub next_future_id: usize,
}

impl Default for Executor {
    fn default() -> Self {
        let config = Config::default();
        let mut env = Environment::default();

        env.push_scope();
        env.set_prev(Value::Decimal(Decimal::ZERO));
        env.set("pi", Value::Decimal(Decimal::PI));
        env.set("e", Value::Decimal(Decimal::E));

        env.define_native_function("func", builtin_func);
        // env.define_native_function("lambda", builtin_lambda);

        // Control Flow Functions
        env.define_native_function("if", builtin_if);
        // env.define_native_function("let", builtin_let);

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
        env.define_native_function("pow", builtin_pow);
        env.define_native_function("sqrt", builtin_sqrt);
        // env.define_native_function("summate", builtin_summate);
        env.define_native_function("factorial", builtin_factorial);

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
            instructions: VecDeque::default(),
            stack: Vec::default(),
            waker: Waker::noop(),
            futures: BTreeMap::default(),
            native_futures: BTreeMap::default(),
            next_future_id: 0,
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
                    return Err(Error::InvalidType);
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
                let mut injector = Injector {
                    instructions: &mut self.instructions,
                    index: 0,
                };

                // injector.inject(Instruction::PushScope);

                // Reverse order ensures the correct value->name binding here.
                for param in f.params.into_iter().rev() {
                    Self::inject_compiled(self.stack.pop().unwrap(), &mut injector)?;
                    // injector.inject(Instruction::Push(self.stack.pop().unwrap()));
                    injector.inject(Instruction::Store(param));
                }

                Self::inject_compiled(*f.body, &mut injector)?;
                // injector.inject(Instruction::PopScope);
            }
            Function::Lambda(l) => {
                let mut injector = Injector {
                    instructions: &mut self.instructions,
                    index: 0,
                };

                injector.inject(Instruction::PushDefinedScope(l.scope));

                // Reverse order ensures the correct value->name binding here.
                for param in l.params.into_iter().rev() {
                    Self::inject_compiled(self.stack.pop().unwrap(), &mut injector)?;
                    // injector.inject(Instruction::Push(self.stack.pop().unwrap()));
                    injector.inject(Instruction::Store(param));
                }

                Self::inject_compiled(*l.body, &mut injector)?;
                injector.inject(Instruction::PopScope);
            }
        }

        Ok(())
    }

    fn execute_instruction(self: &mut Executor, instruction: Instruction) -> Result<(), Error> {
        // eprintln!("Current Instruction: {instruction:?}");
        // eprintln!("Instructions: {:?}", self.instructions);
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
            Instruction::Call => {
                let func = self.stack.pop().unwrap();

                match func {
                    Value::Function(f) => {
                        self.inject_function(f)?;
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
                        self.stack.push(result.unwrap());
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

    pub fn eval(&mut self, expr: Value) -> EvalFuture {
        let future_id = self.next_future_id;
        self.next_future_id += 1;

        let mut injector = Injector::new(&mut self.instructions);

        injector.inject(Instruction::PushScope);
        Self::inject_compiled(expr, &mut injector).unwrap();
        injector.inject(Instruction::Resume(future_id));
        injector.inject(Instruction::PopScope);

        self.futures.insert(future_id, EvalFutureContext::default());

        EvalFuture {
            id: future_id,
            executor: self as *mut Executor,
        }
    }

    pub fn execute(&mut self, value: Value) -> Result<Value, Error> {
        self.instructions.clear();
        self.stack.clear();

        let future = self.eval(value);
        let mut main_future = pin!(future);
        let mut context = Context::from_waker(self.waker);

        loop {
            if let Some(instruction) = self.instructions.pop_front() {
                self.execute_instruction(instruction)?;
            }

            match main_future.as_mut().poll(&mut context) {
                Poll::Ready(result) => return result,
                Poll::Pending => continue,
            }
        }
    }
}
