use crate::Error;
use bigdecimal::BigDecimal;
use misp_parser::SExpr;
use num::ToPrimitive;
use num::{BigInt, BigRational, Zero};

use crate::Executor;

pub fn builtin_add(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
    match args.len() {
        0 => Ok(SExpr::Integer(BigInt::ZERO)),
        1 => executor.eval(&args[0]),
        2 => {
            let left = executor.eval(&args[0])?;
            let right = executor.eval(&args[1])?;

            match (left, right) {
                (SExpr::Integer(a), SExpr::Integer(b)) => Ok(SExpr::Integer(a + b)),
                (SExpr::Decimal(a), SExpr::Decimal(b)) => Ok(SExpr::Decimal(a + b)),
                (SExpr::Rational(a), SExpr::Rational(b)) => Ok(SExpr::Rational(a + b)),

                (SExpr::Integer(a), SExpr::Decimal(b)) => {
                    Ok(SExpr::Decimal(BigDecimal::from(a) + b))
                }
                (SExpr::Decimal(a), SExpr::Integer(b)) => {
                    Ok(SExpr::Decimal(a + BigDecimal::from(b)))
                }

                (SExpr::Integer(a), SExpr::Rational(b)) => {
                    Ok(SExpr::Rational(BigRational::from(a) + b))
                }
                (SExpr::Rational(a), SExpr::Integer(b)) => {
                    Ok(SExpr::Rational(a + BigRational::from(b)))
                }

                // TODO: Currently don't support addition between rationals and decimals...
                _ => Err(Error::FunctionCall),
            }
        }
        _ => Err(Error::FunctionCall),
    }
}

pub fn builtin_minus(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
    match args.len() {
        0 => Ok(SExpr::Integer(BigInt::ZERO)),
        1 => executor.eval(&args[0]),
        2 => {
            let left = executor.eval(&args[0])?;
            let right = executor.eval(&args[1])?;

            match (left, right) {
                (SExpr::Integer(a), SExpr::Integer(b)) => Ok(SExpr::Integer(a - b)),
                (SExpr::Decimal(a), SExpr::Decimal(b)) => Ok(SExpr::Decimal(a - b)),
                (SExpr::Rational(a), SExpr::Rational(b)) => Ok(SExpr::Rational(a - b)),

                (SExpr::Integer(a), SExpr::Decimal(b)) => {
                    Ok(SExpr::Decimal(BigDecimal::from(a) - b))
                }
                (SExpr::Decimal(a), SExpr::Integer(b)) => {
                    Ok(SExpr::Decimal(a - BigDecimal::from(b)))
                }

                (SExpr::Integer(a), SExpr::Rational(b)) => {
                    Ok(SExpr::Rational(BigRational::from(a) - b))
                }
                (SExpr::Rational(a), SExpr::Integer(b)) => {
                    Ok(SExpr::Rational(a - BigRational::from(b)))
                }

                // TODO: Currently don't support addition between rationals and decimals...
                _ => Err(Error::FunctionCall),
            }
        }
        _ => Err(Error::FunctionCall),
    }
}

pub fn builtin_multiply(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
    match args.len() {
        0 => Ok(SExpr::Integer(BigInt::ZERO)),
        1 => executor.eval(&args[0]),
        2 => {
            let left = executor.eval(&args[0])?;
            let right = executor.eval(&args[1])?;

            match (left, right) {
                (SExpr::Integer(a), SExpr::Integer(b)) => Ok(SExpr::Integer(a * b)),
                (SExpr::Decimal(a), SExpr::Decimal(b)) => Ok(SExpr::Decimal(a * b)),
                (SExpr::Rational(a), SExpr::Rational(b)) => Ok(SExpr::Rational(a * b)),

                (SExpr::Integer(a), SExpr::Decimal(b)) => {
                    Ok(SExpr::Decimal(BigDecimal::from(a) * b))
                }
                (SExpr::Decimal(a), SExpr::Integer(b)) => {
                    Ok(SExpr::Decimal(a * BigDecimal::from(b)))
                }

                (SExpr::Integer(a), SExpr::Rational(b)) => {
                    Ok(SExpr::Rational(BigRational::from(a) * b))
                }
                (SExpr::Rational(a), SExpr::Integer(b)) => {
                    Ok(SExpr::Rational(a * BigRational::from(b)))
                }

                // TODO: Currently don't support addition between rationals and decimals...
                _ => Err(Error::FunctionCall),
            }
        }
        _ => Err(Error::FunctionCall),
    }
}

pub fn builtin_divide(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
    match args.len() {
        0 => Ok(SExpr::Integer(BigInt::ZERO)),
        1 => executor.eval(&args[0]),
        2 => {
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

                (SExpr::Integer(a), SExpr::Decimal(b)) => {
                    Ok(SExpr::Decimal(BigDecimal::from(a) / b))
                }
                (SExpr::Decimal(a), SExpr::Integer(b)) => {
                    Ok(SExpr::Decimal(a / BigDecimal::from(b)))
                }

                (SExpr::Integer(a), SExpr::Rational(b)) => {
                    Ok(SExpr::Rational(BigRational::from(a) / b))
                }
                (SExpr::Rational(a), SExpr::Integer(b)) => {
                    Ok(SExpr::Rational(a / BigRational::from(b)))
                }

                // TODO: Currently don't support addition between rationals and decimals...
                _ => Err(Error::FunctionCall),
            }
        }
        _ => Err(Error::FunctionCall),
    }
}

pub fn builtin_sqrt(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
    match args.len() {
        0 => Err(Error::FunctionCall),
        1 => {
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
        x => Err(Error::FunctionArity {
            name: "sqrt".to_string(),
            expected: 1,
            actual: x,
        }),
    }
}

pub fn builtin_pow(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
    match args.len() {
        0 => Ok(SExpr::Integer(BigInt::ZERO)),
        1 => executor.eval(&args[0]),
        2 => {
            // exponent
            let SExpr::Integer(left) = executor.eval(&args[0])? else {
                todo!();
            };
            let right = executor.eval(&args[1])?;

            let left_int = left.to_u32().unwrap();

            match right {
                SExpr::Integer(n) => Ok(SExpr::Integer(n.pow(left_int))),
                SExpr::Rational(r) => Ok(SExpr::Rational(r.pow(left_int as i32))),
                _ => Err(Error::FunctionCall),
            }
        }
        _ => Err(Error::FunctionCall),
    }
}
