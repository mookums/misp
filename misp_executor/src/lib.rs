#![no_std]
extern crate alloc;

mod builtin;
pub mod config;
pub mod environment;
pub mod future;

use alloc::{boxed::Box, rc::Rc, string::String, vec::Vec};
use compact_str::CompactString;
use futures::FutureExt;
use hashbrown::HashMap;

use core::{
    hash::Hash,
    pin::{Pin, pin},
    task::{Context, Poll, Waker},
};

use misp_num::decimal::Decimal;
use misp_parser::SExpr;

use crate::{
    builtin::math::{
        builtin_abs, builtin_add, builtin_divide, builtin_equal, builtin_gt, builtin_gte,
        builtin_lt, builtin_lte, builtin_minus, builtin_multiply, builtin_not_equal, builtin_pow,
        builtin_sqrt,
    },
    config::Config,
    environment::Environment,
    future::{EvalFuture, EvalFutureContext, create_eval_waker},
};

#[derive(Debug, Clone, Hash)]
pub struct Lambda {
    pub params: Vec<CompactString>,
    pub body: Box<Value>,
}

type NativeMispFuture = Pin<Box<dyn Future<Output = Result<Value, Error>> + 'static>>;
type NativeMispFunction = fn(*mut Executor) -> NativeMispFuture;

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

pub struct Executor {
    pub config: Config,
    pub env: Environment,
    pub stack: Vec<Value>,

    pub memos: HashMap<MemoKey, Value>,
    pub next_function_id: usize,

    pub futures: HashMap<usize, EvalFutureContext>,
    pub native_futures: HashMap<usize, NativeMispFuture>,
    pub current_future: Option<usize>,
    pub ready_future: usize,
    pub next_future_id: usize,
}

impl Default for Executor {
    fn default() -> Self {
        let config = Config::default();
        let mut env = Environment::default();

        env.push_scope();
        env.set_prev(Value::Decimal(Decimal::ZERO));

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

        Self {
            config,
            env,
            stack: Vec::default(),
            next_function_id: 0,
            memos: HashMap::default(),
            futures: HashMap::default(),
            native_futures: HashMap::default(),
            current_future: None,
            ready_future: 0,
            next_future_id: 0,
        }
    }
}

impl Executor {
    // async fn run_function(&mut self, func: Function, args: Vec<Value>) -> Result<Value, Error> {
    //     let future_id = self.next_future_id;
    //     self.next_future_id += 1;

    //     injector.inject(Instruction::Push(Value::Function(func)));
    //     injector.inject(Instruction::Call(arity));
    //     injector.inject(Instruction::Resume(future_id));

    //     self.futures.insert(future_id, EvalFutureContext::default());

    //     EvalFuture {
    //         id: future_id,
    //         executor: self as *mut Executor,
    //     }
    //     .await
    // }

    pub fn eval(&mut self, expr: Value) -> EvalFuture {
        let future_id = self.next_future_id;
        self.next_future_id += 1;

        let parent_waker = self
            .current_future
            .and_then(|cf| self.futures.get(&cf).and_then(|f| f.waker.clone()));

        let context = match expr {
            Value::Atom(ref name) => {
                let result = if let Some(val) = self.env.get(name) {
                    Ok(val.clone())
                } else {
                    Err(Error::UnknownSymbol(name.clone()))
                };

                EvalFutureContext {
                    result: Some(result),
                    waker: parent_waker,
                }
            }
            Value::Decimal(_) | Value::Function(_) => EvalFutureContext {
                result: Some(Ok(expr)),
                waker: parent_waker,
            },
            Value::List(values) => {
                let Value::Atom(ref name) = values[0] else {
                    panic!()
                };

                let Value::Function(func) = self.env.get(name).unwrap().clone() else {
                    panic!()
                };

                match func {
                    Function::Native(f) => {
                        let arity: u64 = (values.len() - 1) as u64;
                        self.stack.extend(values.into_iter().skip(1));
                        self.stack.push(Value::Decimal(arity.into()));

                        let mut native_future = f(self);

                        let waker = Waker::noop();
                        let mut context = Context::from_waker(waker);

                        match native_future.poll_unpin(&mut context) {
                            Poll::Ready(value) => EvalFutureContext {
                                result: Some(value),
                                waker: parent_waker,
                            },
                            Poll::Pending => {
                                self.native_futures.insert(future_id, native_future);
                                self.ready_future = future_id;

                                EvalFutureContext {
                                    result: None,
                                    waker: parent_waker,
                                }
                            }
                        }
                    }
                    Function::Runtime(rt) => todo!(),
                    Function::Lambda(l) => todo!(),
                }
            }
        };

        self.futures.insert(future_id, context);

        EvalFuture {
            id: future_id,
            executor: self,
        }
    }

    fn full_reset(&mut self) {
        self.next_future_id = 0;
        self.next_function_id = 0;
        self.ready_future = 0;

        self.native_futures.clear();
        self.futures.clear();
        self.stack.clear();
        self.memos.clear();
    }

    pub fn execute(&mut self, value: Value) -> Result<Value, Error> {
        self.full_reset();

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
                let waker = Waker::noop();
                let func_id = self.next_function_id;
                self.futures.insert(
                    func_id,
                    EvalFutureContext {
                        result: None,
                        waker: Some(waker.clone()),
                    },
                );
                self.next_function_id += 1;
                self.ready_future = func_id;
                self.current_future = Some(func_id);

                // we actually need to do computation.
                let future = self.eval(value);
                let mut main_future = pin!(future);

                let mut context = Context::from_waker(waker);

                loop {
                    // {
                    //     extern crate std;
                    //     use std::eprintln;

                    //     eprintln!("Memos: {:?}", self.memos.keys());
                    //     eprintln!("Stack: {:?}", self.stack);
                    //     eprintln!("Futures: {:?}", self.futures.keys());
                    //     eprintln!("Native Futures: {:?}", self.native_futures.keys());
                    //     eprintln!("Current Future: {:?}", self.current_future);
                    //     eprintln!("Ready Future: {:?}", self.ready_future);
                    //     eprintln!();
                    //     std::thread::sleep(std::time::Duration::from_secs(1));
                    // }

                    match self.ready_future {
                        ready if ready == func_id => {
                            match main_future.as_mut().poll(&mut context) {
                                Poll::Ready(result) => {
                                    let result = result?;
                                    self.env.set_prev(result.clone());
                                    return Ok(result);
                                }
                                Poll::Pending => continue,
                            }
                        }
                        ready => {
                            self.current_future = Some(ready);

                            let eval_future_waker = create_eval_waker(self, ready);
                            if let Some(native) = self.native_futures.get_mut(&ready) {
                                let mut ctx = Context::from_waker(&eval_future_waker);

                                match native.poll_unpin(&mut ctx) {
                                    Poll::Ready(value) => {
                                        let future_context = self
                                            .futures
                                            .get_mut(&ready)
                                            .expect("Future must exist");

                                        future_context.result = Some(value);

                                        self.native_futures.remove(&ready);

                                        if let Some(parent_waker) = &future_context.waker {
                                            parent_waker.wake_by_ref();
                                        }
                                    }
                                    Poll::Pending => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
