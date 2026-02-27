use alloc::vec;
use alloc::vec::Vec;

use crate::{
    Error, Executor,
    operation::{Operation, parse_operation},
    value::{Function, Value},
};

pub fn builtin_expand(executor: &mut Executor) -> Result<Value, Error> {
    let arg = executor.stack.pop().ok_or(Error::EmptyStack)?;
    expand(executor, arg)
}

pub fn expand(executor: &mut Executor, expr: Value) -> Result<Value, Error> {
    match expr {
        Value::Atom(name) => Ok(executor.env.get(&name)),
        Value::List(values) => {
            let mut iter = values.into_iter();
            let name_value = iter.next().unwrap();
            let Value::Atom(ref name) = name_value else {
                panic!("cant expand broken list");
            };

            match parse_operation(name) {
                Some(operation) if operation.is_associative() => {
                    expand_flatten(executor, iter.as_slice(), operation)
                }
                Some(_op) => {
                    let mut expanded = vec![name_value];

                    for arg in iter {
                        expanded.push(expand(executor, arg)?);
                    }

                    Ok(Value::List(expanded))
                }
                None => {
                    // this means it is a function call.
                    //
                    // we want to dynamically substitute values in the function body
                    // with the arguments here
                    //
                    // then we want to put the function body in place here.

                    let Value::Function(func) = executor.env.get(name) else {
                        panic!("unknown function getting expanded");
                    };

                    match func {
                        Function::Runtime(rt) => {
                            let mut body = (*rt.body).clone();

                            for (param, value) in rt.params.iter().zip(iter) {
                                substitute(&mut body, Value::Atom(param.clone()), value);
                            }

                            let expanded = expand(executor, body)?;
                            Ok(expanded)
                        }
                        Function::Lambda(_) => todo!(),
                    }
                }
            }
        }
        Value::Function(_) => panic!(),
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

fn substitute(expr: &mut Value, target: Value, replacement: Value) {
    if *expr == target {
        *expr = replacement;
    } else if let Value::List(list) = expr {
        let iter = list.iter_mut().skip(1);

        for arg in iter {
            substitute(arg, target.clone(), replacement.clone());
        }
    }
}
