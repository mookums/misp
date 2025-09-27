use alloc::vec;
use alloc::vec::Vec;
use misp_num::decimal::Decimal;

use crate::{Error, Executor, Value};

pub fn builtin_simplify(executor: &mut Executor) -> Result<Value, Error> {
    let arg = executor.stack.pop().ok_or(Error::EmptyStack)?;
    simplify(executor, arg)
}

fn simplify(executor: &mut Executor, expr: Value) -> Result<Value, Error> {
    let simplified = match expr {
        Value::List(values) => {
            let Value::Atom(op) = values.first().unwrap() else {
                return Err(Error::InvalidType);
            };

            simplify_operation(executor, op, &values[1..])?
        }
        Value::Atom(str) => executor.env.get(&str),
        Value::Decimal(_) | Value::Symbol(_) => expr,
        _ => expr,
    };

    Ok(simplified)
}

fn simplify_operation(executor: &mut Executor, op: &str, args: &[Value]) -> Result<Value, Error> {
    match op {
        "+" => simplify_addition(executor, args),
        _ => todo!(),
    }
}

fn simplify_addition(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
    let mut sum = Decimal::ZERO;
    let mut symbols = Vec::new();

    for arg in args {
        let simplified = simplify(executor, arg.clone())?;

        match simplified {
            Value::Decimal(n) => sum += n,
            Value::List(list) if list.len() > 1 => {
                if let Value::Atom(op) = &list[0]
                    && op == "+"
                {
                    let flattened = simplify_addition(executor, &list[1..])?;
                    match flattened {
                        Value::List(inner) if inner[0] == Value::Atom("+".into()) => {
                            // Unpack the inner addition terms.
                            for term in &inner[1..] {
                                if let Value::Decimal(n) = term {
                                    sum += *n;
                                } else {
                                    symbols.push(term.clone());
                                }
                            }
                        }
                        Value::Decimal(n) => sum += n,
                        other => symbols.push(other),
                    }

                    continue;
                }
                symbols.push(arg.clone());
            }
            _ => symbols.push(simplified),
        }
    }

    let mut result = Vec::new();

    result.extend(symbols);

    if sum != Decimal::ZERO {
        result.push(Value::Decimal(sum));
    }

    let res = match result.len() {
        0 => Value::Decimal(Decimal::ZERO),
        1 => result[0].clone(),
        _ => Value::List({
            let mut list = vec![Value::Atom("+".into())];
            list.extend(result);
            list
        }),
    };

    Ok(res)
}
