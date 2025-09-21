use crate::{Error, Executor, Injector, Instruction, Value};
use misp_num::decimal::Decimal;

macro_rules! binary_op {
    ($name:ident, $op_name:literal, $op:tt) => {
        pub fn $name(executor: &mut Executor) -> Result<(), Error> {
            let mut injector = Injector {
                instructions: &mut executor.instructions,
                index: 0,
            };

            Executor::inject_compiled(executor.stack.pop().unwrap(), &mut injector)?;
            Executor::inject_compiled(executor.stack.pop().unwrap(), &mut injector)?;
            injector.inject(Instruction::Add);
            Ok(())
        }
    };
}

binary_op!(builtin_add, "+", +);
// binary_op!(builtin_minus, "-", -);
// binary_op!(builtin_multiply, "*", *);
// binary_op!(builtin_divide, "/", /);
// binary_op!(builtin_equal, "==", ==);
// binary_op!(builtin_not_equal, "!=", !=);
// binary_op!(builtin_lt, "<", <);
// binary_op!(builtin_lte, "<=", <=);
// binary_op!(builtin_gt, ">", >);
// binary_op!(builtin_gte, ">=", >=);

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

// pub fn builtin_sqrt(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
//     if args.len() != 1 {
//         return Err(Error::FunctionArity {
//             name: "sqrt".to_string(),
//             expected: 1,
//             actual: args.len(),
//         });
//     }

//     let inner = executor.evaluate(&args[0])?;

//     match inner {
//         Value::Decimal(d) => Ok(Value::Decimal(d.sqrt())),
//         _ => Err(Error::FunctionCall),
//     }
// }

// pub fn builtin_pow(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
//     if args.len() != 2 {
//         return Err(Error::FunctionArity {
//             name: "pow".to_string(),
//             expected: 2,
//             actual: args.len(),
//         });
//     }

//     let Value::Decimal(left) = executor.evaluate(&args[0])? else {
//         return Err(Error::FunctionCall);
//     };

//     let right = executor.evaluate(&args[1])?;

//     match right {
//         Value::Decimal(d) => Ok(Value::Decimal(d.pow(left))),
//         _ => todo!("Unsupported type for pow"),
//     }
// }

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

// pub fn builtin_factorial(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
//     if args.len() != 1 {
//         return Err(Error::FunctionArity {
//             name: "factorial".to_string(),
//             expected: 1,
//             actual: args.len(),
//         });
//     }

//     let Value::Decimal(n) = executor.evaluate(&args[0])? else {
//         return Err(Error::FunctionCall);
//     };

//     let count = n.to_u128() as u64;

//     if count == 0 || count == 1 {
//         return Ok(Value::Decimal(Decimal::ONE));
//     }

//     let mut result = Decimal::ONE;
//     for i in 2..=count {
//         result *= Decimal::from_unsigned(i)
//     }

//     Ok(Value::Decimal(result))
// }
