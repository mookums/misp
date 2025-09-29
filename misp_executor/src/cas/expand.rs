use alloc::vec;
use alloc::vec::Vec;

use crate::{
    Error, Executor,
    operation::{Operation, parse_operation},
    value::Value,
};

pub fn builtin_expand(executor: &mut Executor) -> Result<Value, Error> {
    let arg = executor.stack.pop().ok_or(Error::EmptyStack)?;
    expand(executor, arg)
}

pub fn expand(executor: &mut Executor, expr: Value) -> Result<Value, Error> {
    match expr {
        Value::Atom(name) => Ok(executor.env.get(&name)),
        Value::List(values) => {
            let Value::Atom(op) = values.first().unwrap() else {
                panic!("cant expand broken list");
            };

            match parse_operation(op) {
                Some(operation) if operation.is_associative() => {
                    expand_flatten(executor, &values[1..], operation)
                }
                Some(_) => {
                    let mut iter = values.into_iter();
                    let mut expanded = vec![iter.next().unwrap()];

                    for arg in iter {
                        expanded.push(expand(executor, arg)?);
                    }

                    Ok(Value::List(expanded))
                }
                None => {
                    let mut expanded = Vec::new();
                    for val in values.into_iter() {
                        expanded.push(expand(executor, val)?);
                    }

                    Ok(Value::List(expanded))
                }
            }
        }
        Value::Function(_) => todo!(),
        Value::Symbol(_) | Value::Decimal(_) => Ok(expr),
    }
}

fn expand_flatten(
    executor: &mut Executor,
    args: &[Value],
    operation: Operation,
) -> Result<Value, Error> {
    let mut flattened = Vec::new();

    for arg in args {
        let expanded = expand(executor, arg.clone())?;

        if let Value::List(inner_values) = &expanded
            && let Some(Value::Atom(inner_op)) = inner_values.first()
            && parse_operation(inner_op) == Some(operation)
        {
            flattened.extend_from_slice(&inner_values[1..]);
            continue;
        }

        flattened.push(expanded);
    }

    let mut result = vec![operation.to_atom_value()];
    result.extend(flattened);
    Ok(Value::List(result))
}
