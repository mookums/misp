use misp_num::decimal::Decimal;

use crate::{Error, Executor, Value};

macro_rules! binary_op {
    ($name:ident, $op_name:literal, $op:tt) => {
        pub fn $name(executor: &mut Executor) -> Result<Value, Error> {
            let (right_thunk, left_thunk) = (
                executor.stack.pop().ok_or(Error::EmptyStack)?,
                executor.stack.pop().ok_or(Error::EmptyStack)?,
            );

            let (Value::Decimal(left), Value::Decimal(right)) = (executor.eval(left_thunk)?, executor.eval(right_thunk)?) else {
                return Err(Error::InvalidType);
            };

            Ok(Value::Decimal(Decimal::from(left $op right)))
        }
    };
}

binary_op!(builtin_add, "+", +);
binary_op!(builtin_minus, "-",-);
binary_op!(builtin_multiply, "*", *);
binary_op!(builtin_divide, "/", /);
binary_op!(builtin_equal, "==", ==);
binary_op!(builtin_not_equal, "!=", !=);
binary_op!(builtin_lt, "<", <);
binary_op!(builtin_lte, "<=", <=);
binary_op!(builtin_gt, ">", >);
binary_op!(builtin_gte, ">=", >=);

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

pub fn builtin_sqrt(executor: &mut Executor) -> Result<Value, Error> {
    let value = executor.stack.pop().ok_or(Error::EmptyStack)?;

    let Value::Decimal(evaluated) = executor.eval(value)? else {
        return Err(Error::InvalidType);
    };

    Ok(Value::Decimal(evaluated.sqrt()))
}

pub fn builtin_pow(executor: &mut Executor) -> Result<Value, Error> {
    let (pow_thunk, base_thunk) = (
        executor.stack.pop().ok_or(Error::EmptyStack)?,
        executor.stack.pop().ok_or(Error::EmptyStack)?,
    );

    let (Value::Decimal(pow), Value::Decimal(base)) =
        (executor.eval(pow_thunk)?, executor.eval(base_thunk)?)
    else {
        return Err(Error::InvalidType);
    };

    Ok(Value::Decimal(base.pow(pow)))
}

pub fn builtin_summate(executor: &mut Executor) -> Result<Value, Error> {
    let (func, end, start) = (
        executor.stack.pop().ok_or(Error::EmptyStack)?,
        executor.stack.pop().ok_or(Error::EmptyStack)?,
        executor.stack.pop().ok_or(Error::EmptyStack)?,
    );

    let Value::Decimal(start) = executor.eval(start)? else {
        return Err(Error::InvalidType);
    };

    let Value::Decimal(end) = executor.eval(end)? else {
        return Err(Error::InvalidType);
    };

    let Value::Function(f) = executor.eval(func)? else {
        return Err(Error::InvalidType);
    };

    let mut start = start.to_u128() as u64;
    let end = end.to_u128() as u64;
    let mut sum = Decimal::ZERO;

    while start <= end {
        let current_decimal = Value::Decimal(Decimal::from(start));
        let result = executor.eval_function(f.clone(), vec![current_decimal])?;
        let Value::Decimal(result_decimal) = result else {
            return Err(Error::InvalidType);
        };

        sum += result_decimal;
        start += 1;
    }

    Ok(Value::Decimal(sum))
}

pub fn builtin_factorial(executor: &mut Executor) -> Result<Value, Error> {
    let value = executor.stack.pop().ok_or(Error::EmptyStack)?;

    let Value::Decimal(n) = executor.eval(value)? else {
        return Err(Error::InvalidType);
    };

    let n_int = n.to_u128() as u64;

    let mut result = Decimal::ONE;
    for i in 1..=n_int {
        result *= Decimal::from(i);
    }

    Ok(Value::Decimal(result))
}
