use crate::{Error, Executor, Function, Lambda, RuntimeMispFunction, Value};

pub fn builtin_func(executor: &mut Executor) -> Result<Value, Error> {
    let (body, params_thunk, name) = (
        executor.stack.pop().ok_or(Error::EmptyStack)?,
        executor.stack.pop().ok_or(Error::EmptyStack)?,
        executor.stack.pop().ok_or(Error::EmptyStack)?,
    );

    let params = match params_thunk {
        Value::List(param_list) => {
            let mut params = Vec::new();
            for param in param_list {
                match param {
                    Value::Atom(p) => params.push(p),
                    _ => return Err(Error::FunctionCall),
                }
            }
            params
        }
        _ => return Err(Error::FunctionCall),
    };

    let Value::Atom(name) = name else {
        return Err(Error::InvalidType);
    };

    let function = Value::Function(Function::Runtime(RuntimeMispFunction {
        params,
        body: Box::new(body),
    }));

    executor.env.set(name, function.clone());
    Ok(function)
}

pub fn builtin_lambda(executor: &mut Executor) -> Result<Value, Error> {
    let (body, params_thunk) = (
        executor.stack.pop().ok_or(Error::EmptyStack)?,
        executor.stack.pop().ok_or(Error::EmptyStack)?,
    );

    let params = match params_thunk {
        Value::List(param_list) => {
            let mut params = Vec::new();
            for param in param_list {
                match param {
                    Value::Atom(p) => params.push(p.clone()),
                    _ => return Err(Error::FunctionCall),
                }
            }
            params
        }
        _ => return Err(Error::FunctionCall),
    };

    let lambda = Value::Function(Function::Lambda(Lambda {
        params,
        body: Box::new(body),
        scope: executor.env.current_scope().clone(),
    }));

    Ok(lambda)
}
