use misp_num::decimal::Decimal;

use crate::{Error, Executor, Injector, Instruction, Value};

macro_rules! binary_op {
    ($name:ident, $op_name:literal, $instr:ident) => {
        pub fn $name(executor: &mut Executor) -> Result<(), Error> {
            let (first, second) = (
                executor.stack.pop().ok_or(Error::EmptyStack)?,
                executor.stack.pop().ok_or(Error::EmptyStack)?,
            );

            let mut injector = Injector {
                instructions: &mut executor.instructions,
                index: 0,
            };

            Executor::inject_compiled(first, &mut injector)?;
            Executor::inject_compiled(second, &mut injector)?;
            injector.inject(Instruction::$instr);
            Ok(())
        }
    };
}

binary_op!(builtin_add, "+", Add);
binary_op!(builtin_minus, "-", Sub);
binary_op!(builtin_multiply, "*", Mult);
binary_op!(builtin_divide, "/", Div);
binary_op!(builtin_equal, "==", Eq);
binary_op!(builtin_not_equal, "!=", NotEq);
binary_op!(builtin_lt, "<", Lt);
binary_op!(builtin_lte, "<=", Lte);
binary_op!(builtin_gt, ">", Gt);
binary_op!(builtin_gte, ">=", Gte);

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

pub fn builtin_sqrt(executor: &mut Executor) -> Result<(), Error> {
    let value = executor.stack.pop().ok_or(Error::EmptyStack)?;
    let evaluated = executor.eval(value)?;

    let mut injector = Injector {
        instructions: &mut executor.instructions,
        index: 0,
    };

    injector.inject(Instruction::Push(evaluated));
    injector.inject(Instruction::Sqrt);
    Ok(())
}

pub fn builtin_pow(executor: &mut Executor) -> Result<(), Error> {
    let (first, second) = (
        executor.stack.pop().ok_or(Error::EmptyStack)?,
        executor.stack.pop().ok_or(Error::EmptyStack)?,
    );

    let mut injector = Injector {
        instructions: &mut executor.instructions,
        index: 0,
    };

    Executor::inject_compiled(first, &mut injector)?;
    Executor::inject_compiled(second, &mut injector)?;
    injector.inject(Instruction::Pow);
    Ok(())
}

// pub fn builtin_summate(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
//     if args.len() != 3 {
//         return Err(Error::FunctionArity {
//             name: "summate".to_string(),
//             expected: 3,
//             actual: args.len(),
//         });
//     }

//     let Value::Decimal(start) = executor.evaluate(&args[0])? else {
//         return Err(Error::FunctionCall);
//     };

//     let Value::Decimal(end) = executor.evaluate(&args[1])? else {
//         return Err(Error::FunctionCall);
//     };

//     let mut start = start.to_u128() as u64;
//     let end = end.to_u128() as u64;

//     let ret = match executor.evaluate(&args[2])? {
//         Value::Function(function) => {
//             let mut sum = Decimal::ZERO;

//             while start <= end {
//                 let current = Decimal::from_unsigned(start);

//                 let incr = match executor.run_func(&function, &[Value::Decimal(current)])? {
//                     Value::Decimal(decimal) => decimal,
//                     _ => return Err(Error::FunctionCall),
//                 };

//                 sum += incr;
//                 start += 1
//             }

//             sum
//         }
//         _ => return Err(Error::FunctionCall),
//     };

//     Ok(Value::Decimal(ret))
// }

// pub fn builtin_factorial(executor: &mut Executor) -> Result<(), Error> {
//     let mut injector = Injector {
//         instructions: &mut executor.instructions,
//         index: 0,
//     };

//     Executor::inject_compiled(executor.stack.pop().unwrap(), &mut injector)?;

//     Ok(())
// }

pub fn builtin_factorial(executor: &mut Executor) -> Result<(), Error> {
    let value = executor.stack.pop().ok_or(Error::EmptyStack)?;

    let Value::Decimal(n) = executor.eval(value)? else {
        return Err(Error::InvalidType);
    };

    let n_int = n.to_u128() as u64;

    let mut result = Decimal::ONE;
    for i in 1..=n_int {
        result *= Decimal::from(i);
    }

    let mut injector = Injector {
        instructions: &mut executor.instructions,
        index: 0,
    };

    injector.inject(Instruction::Push(Value::Decimal(result)));

    Ok(())
}
