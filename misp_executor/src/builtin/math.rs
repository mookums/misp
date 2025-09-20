use crate::{Error, Executor, Value};
use misp_num::decimal::Decimal;

macro_rules! binary_op {
    ($name:ident, $op_name:literal, $op:tt) => {
        pub fn $name(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
            if args.is_empty() {
                return Err(Error::FunctionArity {
                    name: $op_name.to_string(),
                    expected: 2,
                    actual: args.len(),
                });
            }

            if args.len() != 2 {
                return Err(Error::FunctionArity {
                    name: $op_name.to_string(),
                    expected: 2,
                    actual: args.len(),
                });
            }

            let left = executor.eval(&args[0])?;
            let right = executor.eval(&args[1])?;

            match (left, right) {
                (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a $op b)),
                _ => Err(Error::FunctionCall),
            }
        }
    };
}

macro_rules! binary_comparison_op {
    ($name:ident, $op_name:literal, $op:tt) => {
        pub fn $name(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
            if args.len() != 2 {
                return Err(Error::FunctionArity {
                    name: $op_name.to_string(),
                    expected: 2,
                    actual: args.len(),
                });
            }

            let left = executor.eval(&args[0])?;
            let right = executor.eval(&args[1])?;

            let result = match (left, right) {
                (Value::Decimal(a), Value::Decimal(b)) => a $op b,
                _ => return Err(Error::FunctionCall),
            };

            Ok(Value::Decimal(Decimal::from(result)))
        }
    };
}

binary_op!(builtin_add, "+", +);
binary_op!(builtin_minus, "-", -);
binary_op!(builtin_multiply, "*", *);
binary_op!(builtin_divide, "/", /);

binary_comparison_op!(builtin_equal, "==", ==);
binary_comparison_op!(builtin_not_equal, "!=", !=);
binary_comparison_op!(builtin_lt, "<", <);
binary_comparison_op!(builtin_lte, "<=", <=);
binary_comparison_op!(builtin_gt, ">", >);
binary_comparison_op!(builtin_gte, ">=", >=);

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

//     let inner = executor.eval(&args[0])?;

//     match inner {
//         Value::Integer(n) => {
//             let result = BigDecimal::from(n).sqrt().unwrap_or(BigDecimal::zero());

//             if result.is_integer() {
//                 let (value, _) = result.with_scale(0).into_bigint_and_exponent();
//                 Ok(Value::Integer(value))
//             } else {
//                 Ok(Value::Decimal(result))
//             }
//         }
//         Value::Decimal(d) => Ok(Value::Decimal(d.sqrt().unwrap_or(BigDecimal::zero()))),
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

//     let Value::Integer(left) = executor.eval(&args[0])? else {
//         return Err(Error::FunctionCall);
//     };
//     let left_int = left.to_u32().unwrap();

//     let right = executor.eval(&args[1])?;

//     match right {
//         Value::Integer(n) => Ok(Value::Integer(n.pow(left_int))),
//         Value::Rational(r) => Ok(Value::Rational(r.pow(left_int as i32))),
//         _ => todo!("Unsupported type for pow"),
//     }
// }
