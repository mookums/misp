use crate::{Error, Executor, Value};
use bigdecimal::BigDecimal;
use num::{BigInt, ToPrimitive};
use num::{BigRational, Zero};

macro_rules! binary_op {
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

            match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a $op b)),
                (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a $op b)),
                (Value::Rational(a), Value::Rational(b)) => Ok(Value::Rational(a $op b)),

                (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Decimal(BigDecimal::from(a) $op b)),
                (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Decimal(a $op BigDecimal::from(b))),

                (Value::Integer(a), Value::Rational(b)) => Ok(Value::Rational(BigRational::from(a) $op b)),
                (Value::Rational(a), Value::Integer(b)) => Ok(Value::Rational(a $op BigRational::from(b))),

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
                (Value::Integer(a), Value::Integer(b)) => a $op b,
                (Value::Decimal(a), Value::Decimal(b)) => a $op b,
                (Value::Rational(a), Value::Rational(b)) => a $op b,

                (Value::Integer(a), Value::Decimal(b)) => BigDecimal::from(a) $op b,
                (Value::Decimal(a), Value::Integer(b)) => a $op BigDecimal::from(b),

                (Value::Integer(a), Value::Rational(b)) => BigRational::from(a) $op b,
                (Value::Rational(a), Value::Integer(b)) => a $op BigRational::from(b),

                _ => return Err(Error::FunctionCall),
            };

            Ok(Value::Integer(BigInt::from(result)))
        }
    };
}

binary_op!(builtin_add, "+", +);
binary_op!(builtin_minus, "-", -);
binary_op!(builtin_multiply, "*", *);

binary_comparison_op!(builtin_equal, "==", ==);
binary_comparison_op!(builtin_not_equal, "!=", !=);
binary_comparison_op!(builtin_lt, "<", <);
binary_comparison_op!(builtin_lte, "<=", <=);
binary_comparison_op!(builtin_gt, ">", >);
binary_comparison_op!(builtin_gte, ">=", >=);

pub fn builtin_divide(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
    if args.len() != 2 {
        return Err(Error::FunctionArity {
            name: "/".to_string(),
            expected: 2,
            actual: args.len(),
        });
    }

    let left = executor.eval(&args[0])?;
    let right = executor.eval(&args[1])?;

    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            let res = BigDecimal::from(a) / BigDecimal::from(b);

            if res.is_integer() {
                let int_res = res.with_scale(0).as_bigint_and_exponent().0;
                Ok(Value::Integer(int_res))
            } else {
                Ok(Value::Decimal(res))
            }
        }
        (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a / b)),
        (Value::Rational(a), Value::Rational(b)) => Ok(Value::Rational(a / b)),

        (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Decimal(BigDecimal::from(a) / b)),
        (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Decimal(a / BigDecimal::from(b))),

        (Value::Integer(a), Value::Rational(b)) => Ok(Value::Rational(BigRational::from(a) / b)),
        (Value::Rational(a), Value::Integer(b)) => Ok(Value::Rational(a / BigRational::from(b))),

        // TODO: Currently don't support addition between rationals and decimals...
        _ => Err(Error::FunctionCall),
    }
}

pub fn builtin_mod(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
    if args.len() != 2 {
        return Err(Error::FunctionArity {
            name: "%".to_string(),
            expected: 2,
            actual: args.len(),
        });
    }

    let left = match executor.eval(&args[0])? {
        Value::Integer(n) => n,
        Value::Decimal(d) => {
            if d.is_integer() {
                d.with_scale(0).into_bigint_and_exponent().0
            } else {
                return Err(Error::FunctionCall);
            }
        }
        Value::Rational(r) => {
            if r.is_integer() {
                r.to_integer()
            } else {
                return Err(Error::FunctionCall);
            }
        }
        _ => return Err(Error::FunctionCall),
    };

    let right = match executor.eval(&args[1])? {
        Value::Integer(n) => n,
        Value::Decimal(d) => {
            if d.is_integer() {
                d.with_scale(0).into_bigint_and_exponent().0
            } else {
                return Err(Error::FunctionCall);
            }
        }
        Value::Rational(r) => {
            if r.is_integer() {
                r.to_integer()
            } else {
                return Err(Error::FunctionCall);
            }
        }
        _ => return Err(Error::FunctionCall),
    };

    Ok(Value::Integer(left % right))
}

pub fn builtin_sqrt(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
    if args.len() != 1 {
        return Err(Error::FunctionArity {
            name: "sqrt".to_string(),
            expected: 1,
            actual: args.len(),
        });
    }

    let inner = executor.eval(&args[0])?;

    match inner {
        Value::Integer(n) => {
            let result = BigDecimal::from(n).sqrt().unwrap_or(BigDecimal::zero());

            if result.is_integer() {
                let (value, _) = result.with_scale(0).into_bigint_and_exponent();
                Ok(Value::Integer(value))
            } else {
                Ok(Value::Decimal(result))
            }
        }
        Value::Decimal(d) => Ok(Value::Decimal(d.sqrt().unwrap_or(BigDecimal::zero()))),
        _ => Err(Error::FunctionCall),
    }
}

pub fn builtin_pow(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
    if args.len() != 2 {
        return Err(Error::FunctionArity {
            name: "pow".to_string(),
            expected: 2,
            actual: args.len(),
        });
    }

    let Value::Integer(left) = executor.eval(&args[0])? else {
        return Err(Error::FunctionCall);
    };
    let left_int = left.to_u32().unwrap();

    let right = executor.eval(&args[1])?;

    match right {
        Value::Integer(n) => Ok(Value::Integer(n.pow(left_int))),
        Value::Rational(r) => Ok(Value::Rational(r.pow(left_int as i32))),
        _ => Err(Error::FunctionCall),
    }
}
