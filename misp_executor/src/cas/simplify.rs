use alloc::vec;
use alloc::vec::Vec;
use misp_num::decimal::Decimal;

use crate::{
    Error, Executor, Value,
    cas::expand::expand,
    operation::{Operation, UnaryOperation, VariadicOperation, parse_operation},
};

pub fn builtin_simplify(executor: &mut Executor) -> Result<Value, Error> {
    let arg = executor.stack.pop().ok_or(Error::EmptyStack)?;
    simplify(executor, arg)
}

fn simplify(executor: &mut Executor, expr: Value) -> Result<Value, Error> {
    let expanded = expand(executor, expr)?;

    let simplified = match expanded {
        Value::List(values) => {
            let Value::Atom(op) = values.first().unwrap() else {
                return Err(Error::InvalidType);
            };

            simplify_operation(executor, op, &values[1..])?
        }
        Value::Atom(str) => executor.env.get(&str),
        Value::Decimal(_) | Value::Symbol(_) => expanded,
        _ => expanded,
    };

    Ok(simplified)
}

fn simplify_operation(executor: &mut Executor, op: &str, args: &[Value]) -> Result<Value, Error> {
    let Some(operation) = parse_operation(op) else {
        panic!("unsupported simplification");
    };

    match operation {
        Operation::Variadic(variadic) => match variadic {
            VariadicOperation::Add => simplify_addition(executor, args),
            VariadicOperation::Sub => todo!(),
            VariadicOperation::Mult => simplify_multiply(executor, args),
            VariadicOperation::Div => todo!(),
        },
        Operation::Binary(_) => todo!(),
        Operation::Unary(unary) => match unary {
            UnaryOperation::Sqrt => todo!(),
            UnaryOperation::Abs => todo!(),
        },
    }
}

fn simplify_addition(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
    let mut sum = Decimal::ZERO;
    let mut symbols = Vec::new();

    for arg in args {
        let simplified = simplify(executor, arg.clone())?;

        match simplified {
            Value::Decimal(n) => sum += n,
            Value::List(_) => {
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

fn simplify_multiply(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
    let mut product = Decimal::ONE;
    let mut symbols = Vec::new();

    for arg in args {
        let simplified = simplify(executor, arg.clone())?;

        match simplified {
            Value::Decimal(n) => product *= n,
            Value::List(_) => {
                symbols.push(arg.clone());
            }
            _ => symbols.push(simplified),
        }
    }

    let mut result = Vec::new();

    result.extend(symbols);

    if product != Decimal::ONE {
        result.push(Value::Decimal(product));
    }

    let res = match result.len() {
        0 => Value::Decimal(Decimal::ZERO),
        1 => result[0].clone(),
        _ => Value::List({
            let mut list = vec![Value::Atom("*".into())];
            list.extend(result);
            list
        }),
    };

    Ok(res)
}
