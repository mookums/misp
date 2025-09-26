use alloc::{boxed::Box, vec, vec::Vec};
use misp_num::{Sign, decimal::Decimal};

use crate::{Error, Executor, NativeMispFuture, Value, arity_check, quick_eval};

const MAX_VARIADIC_ARGS: usize = 16;

macro_rules! binary_op {
    ($name:ident, $op:tt) => {
        pub fn $name(executor_ptr: *mut Executor) -> NativeMispFuture {
            Box::pin(async move {
                let executor = unsafe { &mut *executor_ptr };
                let mut values: [Decimal; MAX_VARIADIC_ARGS] = [const { Decimal::ZERO } ; MAX_VARIADIC_ARGS];

                let Value::Decimal(arg_count) = executor.stack.pop().ok_or(Error::EmptyStack)?
                else {
                    return Err(Error::InvalidType);
                };

                let arity = arg_count.to_u128() as usize;
                if arity == 0 {
                    return Err(Error::InvalidType);
                }

                let values = &mut values[..arity];
                for value in values.iter_mut().rev() {
                    let thunk = executor.stack.pop().unwrap();
                    *value = match thunk {
                        Value::Decimal(val) => val,
                        other => {
                            let Value::Decimal(val) = executor.eval(other).await? else {
                                return Err(Error::InvalidType);
                            };
                            val
                        }
                    };
                }

                let mut acc = values[0];
                for val in &values[1..arity] {
                    acc = Decimal::from(acc $op *val);
                }

                Ok(Value::Decimal(acc))
            })
        }
    };
}

macro_rules! binary_comparison_op {
    ($name:ident, $op:tt) => {
        pub fn $name(executor_ptr: *mut Executor) -> NativeMispFuture {
            Box::pin(async move {
                let executor = unsafe { &mut *executor_ptr };

                let Value::Decimal(arg_count) = executor.stack.pop().ok_or(Error::EmptyStack)?
                else {
                    return Err(Error::InvalidType);
                };

                let count = arg_count.to_u128() as usize;

                let mut prev = None;

                for _ in 0..count {
                    let thunk = executor.stack.pop().ok_or(Error::EmptyStack)?;

                    let value = if let Value::Decimal(val) = thunk {
                        val
                    } else {
                        let Value::Decimal(val) = executor.eval(thunk).await? else {
                            return Err(Error::InvalidType);
                        };
                        val
                    };

                    if let Some(prev_value) = prev && Decimal::from(value $op prev_value) != Decimal::ONE {
                            return Ok(Value::Decimal(Decimal::ZERO))
                    }

                    prev = Some(value)
                }

                Ok(Value::Decimal(Decimal::ONE))
            })
        }
    };
}

binary_op!(builtin_add, +);
binary_op!(builtin_minus,-);
binary_op!(builtin_multiply, *);
binary_op!(builtin_divide, /);
binary_comparison_op!(builtin_equal, ==);
binary_comparison_op!(builtin_not_equal, !=);
binary_comparison_op!(builtin_lt, <);
binary_comparison_op!(builtin_lte, <=);
binary_comparison_op!(builtin_gt, >);
binary_comparison_op!(builtin_gte, >=);

// pub fn builtin_mod(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
//     if args.len() != 2 {
//         return Err(Error::FunctionArity {
//             name: "%".to_string(),
//             expected: 2,
//             actual: args.len(),
//         });
//     }

//     let left = match executor.eval(&args[0])? {
//         Value::Integer(n) => n,
//         Value::Decimal(d) => {
//             if d.is_integer() {
//                 d.with_scale(0).into_bigint_and_exponent().0
//             } else {
//                 return Err(Error::FunctionCall);
//             }
//         }
//         Value::Rational(r) => {
//             if r.is_integer() {
//                 r.to_integer()
//             } else {
//                 return Err(Error::FunctionCall);
//             }
//         }
//         _ => return Err(Error::FunctionCall),
//     };

//     let right = match executor.eval(&args[1])? {
//         Value::Integer(n) => n,
//         Value::Decimal(d) => {
//             if d.is_integer() {
//                 d.with_scale(0).into_bigint_and_exponent().0
//             } else {
//                 return Err(Error::FunctionCall);
//             }
//         }
//         Value::Rational(r) => {
//             if r.is_integer() {
//                 r.to_integer()
//             } else {
//                 return Err(Error::FunctionCall);
//             }
//         }
//         _ => return Err(Error::FunctionCall),
//     };

//     Ok(Value::Integer(left % right))
// }

pub fn builtin_abs(executor: *mut Executor, values: Vec<Value>) -> NativeMispFuture {
    Box::pin(async move {
        let executor = unsafe { &mut *executor };

        let Value::Decimal(mut evaluated) = executor.eval(values[0].clone()).await? else {
            panic!()
        };

        // arity_check!(executor, "abs", 1);

        // let mut evaluated = quick_eval!(executor, Decimal);

        evaluated.sign = Sign::Positive;
        Ok(Value::Decimal(evaluated))
    })
}

pub fn builtin_min(executor: *mut Executor) -> NativeMispFuture {
    Box::pin(async move {
        let executor = unsafe { &mut *executor };

        let Value::Decimal(arg_count) = executor.stack.pop().ok_or(Error::EmptyStack)? else {
            return Err(Error::InvalidType);
        };

        let count = arg_count.to_u128() as usize;
        if count == 0 {
            return Err(Error::InvalidType);
        }

        let mut thunks = Vec::with_capacity(count);
        for _ in 0..count {
            thunks.push(executor.stack.pop().ok_or(Error::EmptyStack)?);
        }

        let mut values = Vec::with_capacity(count);
        for thunk in thunks.into_iter().rev() {
            let Value::Decimal(val) = executor.eval(thunk).await? else {
                return Err(Error::InvalidType);
            };
            values.push(val);
        }

        let min = values.into_iter().reduce(|acc, x| acc.min(x)).unwrap();
        Ok(Value::Decimal(min))
    })
}

pub fn builtin_max(executor: *mut Executor) -> NativeMispFuture {
    Box::pin(async move {
        let executor = unsafe { &mut *executor };

        let Value::Decimal(arg_count) = executor.stack.pop().ok_or(Error::EmptyStack)? else {
            return Err(Error::InvalidType);
        };

        let count = arg_count.to_u128() as usize;
        if count == 0 {
            return Err(Error::InvalidType);
        }

        let mut thunks = Vec::with_capacity(count);
        for _ in 0..count {
            thunks.push(executor.stack.pop().ok_or(Error::EmptyStack)?);
        }

        let mut values = Vec::with_capacity(count);
        for thunk in thunks.into_iter().rev() {
            let Value::Decimal(val) = executor.eval(thunk).await? else {
                return Err(Error::InvalidType);
            };
            values.push(val);
        }

        let max = values.into_iter().reduce(|acc, x| acc.max(x)).unwrap();
        Ok(Value::Decimal(max))
    })
}

pub fn builtin_sqrt(executor: *mut Executor, values: Vec<Value>) -> NativeMispFuture {
    Box::pin(async move {
        let executor = unsafe { &mut *executor };
        // arity_check!(executor, "sqrt", 1);

        let Value::Decimal(evaluated) = executor.eval(values[0].clone()).await? else {
            panic!()
        };

        // let evaluated = quick_eval!(executor, Decimal);

        Ok(Value::Decimal(evaluated.sqrt()))
    })
}

pub fn builtin_pow(executor: *mut Executor, values: Vec<Value>) -> NativeMispFuture {
    Box::pin(async move {
        let executor = unsafe { &mut *executor };
        // arity_check!(executor, "pow", 2);

        let Value::Decimal(pow) = executor.eval(values[0].clone()).await? else {
            panic!()
        };

        let Value::Decimal(base) = executor.eval(values[1].clone()).await? else {
            panic!()
        };

        // let pow = quick_eval!(executor, Decimal);
        // let base = quick_eval!(executor, Decimal);

        Ok(Value::Decimal(base.pow(pow)))
    })
}

// pub fn builtin_summate(executor: *mut Executor) -> NativeMispFuture {
//     Box::pin(async move {
//         let executor = unsafe { &mut *executor };
//         arity_check!(executor, "summate", 3);

//         let func = quick_eval!(executor, Function);
//         let end = quick_eval!(executor, Decimal);
//         let start = quick_eval!(executor, Decimal);

//         let mut start = start.to_u128() as u64;
//         let end = end.to_u128() as u64;
//         let mut sum = Decimal::ZERO;

//         while start <= end {
//             let current_decimal = Value::Decimal(Decimal::from(start));
//             let result = executor
//                 .run_function(func.clone(), vec![current_decimal])
//                 .await?;
//             let Value::Decimal(result_decimal) = result else {
//                 return Err(Error::InvalidType);
//             };

//             sum += result_decimal;
//             start += 1;
//         }

//         Ok(Value::Decimal(sum))
//     })
// }
