use crate::Error;
use bigdecimal::BigDecimal;
use misp_parser::SExpr;
use num::{BigInt, ToPrimitive};
use num::{BigRational, Zero};

use crate::Executor;

macro_rules! binary_op {
    ($name:ident, $op_name:literal, $op:tt) => {
        pub fn $name(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
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
                (SExpr::Integer(a), SExpr::Integer(b)) => Ok(SExpr::Integer(a $op b)),
                (SExpr::Decimal(a), SExpr::Decimal(b)) => Ok(SExpr::Decimal(a $op b)),
                (SExpr::Rational(a), SExpr::Rational(b)) => Ok(SExpr::Rational(a $op b)),

                (SExpr::Integer(a), SExpr::Decimal(b)) => Ok(SExpr::Decimal(BigDecimal::from(a) $op b)),
                (SExpr::Decimal(a), SExpr::Integer(b)) => Ok(SExpr::Decimal(a $op BigDecimal::from(b))),

                (SExpr::Integer(a), SExpr::Rational(b)) => Ok(SExpr::Rational(BigRational::from(a) $op b)),
                (SExpr::Rational(a), SExpr::Integer(b)) => Ok(SExpr::Rational(a $op BigRational::from(b))),

                _ => Err(Error::FunctionCall),
            }
        }
    };
}

macro_rules! binary_comparison_op {
    ($name:ident, $op_name:literal, $op:tt) => {
        pub fn $name(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
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
                (SExpr::Integer(a), SExpr::Integer(b)) => a $op b,
                (SExpr::Decimal(a), SExpr::Decimal(b)) => a $op b,
                (SExpr::Rational(a), SExpr::Rational(b)) => a $op b,

                (SExpr::Integer(a), SExpr::Decimal(b)) => BigDecimal::from(a) $op b,
                (SExpr::Decimal(a), SExpr::Integer(b)) => a $op BigDecimal::from(b),

                (SExpr::Integer(a), SExpr::Rational(b)) => BigRational::from(a) $op b,
                (SExpr::Rational(a), SExpr::Integer(b)) => a $op BigRational::from(b),

                _ => return Err(Error::FunctionCall),
            };

            Ok(SExpr::Integer(BigInt::from(result)))
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

pub fn builtin_divide(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
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
        (SExpr::Integer(a), SExpr::Integer(b)) => {
            let res = BigDecimal::from(a) / BigDecimal::from(b);

            if res.is_integer() {
                let int_res = res.with_scale(0).as_bigint_and_exponent().0;
                Ok(SExpr::Integer(int_res))
            } else {
                Ok(SExpr::Decimal(res))
            }
        }
        (SExpr::Decimal(a), SExpr::Decimal(b)) => Ok(SExpr::Decimal(a / b)),
        (SExpr::Rational(a), SExpr::Rational(b)) => Ok(SExpr::Rational(a / b)),

        (SExpr::Integer(a), SExpr::Decimal(b)) => Ok(SExpr::Decimal(BigDecimal::from(a) / b)),
        (SExpr::Decimal(a), SExpr::Integer(b)) => Ok(SExpr::Decimal(a / BigDecimal::from(b))),

        (SExpr::Integer(a), SExpr::Rational(b)) => Ok(SExpr::Rational(BigRational::from(a) / b)),
        (SExpr::Rational(a), SExpr::Integer(b)) => Ok(SExpr::Rational(a / BigRational::from(b))),

        // TODO: Currently don't support addition between rationals and decimals...
        _ => Err(Error::FunctionCall),
    }
}

pub fn builtin_mod(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
    if args.len() != 2 {
        return Err(Error::FunctionArity {
            name: "%".to_string(),
            expected: 2,
            actual: args.len(),
        });
    }

    let left = match executor.eval(&args[0])? {
        SExpr::Integer(n) => n,
        SExpr::Decimal(d) => {
            if d.is_integer() {
                d.with_scale(0).into_bigint_and_exponent().0
            } else {
                return Err(Error::FunctionCall);
            }
        }
        SExpr::Rational(r) => {
            if r.is_integer() {
                r.to_integer()
            } else {
                return Err(Error::FunctionCall);
            }
        }
        _ => return Err(Error::FunctionCall),
    };

    let right = match executor.eval(&args[1])? {
        SExpr::Integer(n) => n,
        SExpr::Decimal(d) => {
            if d.is_integer() {
                d.with_scale(0).into_bigint_and_exponent().0
            } else {
                return Err(Error::FunctionCall);
            }
        }
        SExpr::Rational(r) => {
            if r.is_integer() {
                r.to_integer()
            } else {
                return Err(Error::FunctionCall);
            }
        }
        _ => return Err(Error::FunctionCall),
    };

    Ok(SExpr::Integer(left % right))
}

pub fn builtin_sqrt(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
    if args.len() != 1 {
        return Err(Error::FunctionArity {
            name: "sqrt".to_string(),
            expected: 1,
            actual: args.len(),
        });
    }

    let inner = executor.eval(&args[0])?;

    match inner {
        SExpr::Integer(n) => {
            let result = BigDecimal::from(n).sqrt().unwrap_or(BigDecimal::zero());

            if result.is_integer() {
                let (value, _) = result.with_scale(0).into_bigint_and_exponent();
                Ok(SExpr::Integer(value))
            } else {
                Ok(SExpr::Decimal(result))
            }
        }
        SExpr::Decimal(d) => Ok(SExpr::Decimal(d.sqrt().unwrap_or(BigDecimal::zero()))),
        _ => Err(Error::FunctionCall),
    }
}

pub fn builtin_pow(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
    if args.len() != 2 {
        return Err(Error::FunctionArity {
            name: "pow".to_string(),
            expected: 2,
            actual: args.len(),
        });
    }

    let SExpr::Integer(left) = executor.eval(&args[0])? else {
        return Err(Error::FunctionCall);
    };
    let left_int = left.to_u32().unwrap();

    let right = executor.eval(&args[1])?;

    match right {
        SExpr::Integer(n) => Ok(SExpr::Integer(n.pow(left_int))),
        SExpr::Rational(r) => Ok(SExpr::Rational(r.pow(left_int as i32))),
        _ => Err(Error::FunctionCall),
    }
}
