use crate::{
    Error, Executor, Function, Lambda, NativeMispFuture, RuntimeMispFunction, Value, arity_check,
};

pub fn builtin_func(executor: *mut Executor) -> NativeMispFuture {
    Box::pin(async move {
        let executor = unsafe { &mut *executor };
        arity_check!(executor, "func", 3);

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

        let function_id = executor.next_function_id;
        executor.next_function_id += 1;

        let function = Value::Function(Function::Runtime(RuntimeMispFunction {
            id: function_id,
            params,
            body: body.into(),
        }));

        executor.env.set_global(name, function.clone());
        Ok(function)
    })
}

pub fn builtin_lambda(executor: *mut Executor) -> NativeMispFuture {
    Box::pin(async move {
        let executor = unsafe { &mut *executor };
        arity_check!(executor, "lamdba", 2);

        let (body, params_thunk) = (
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

        let lambda = Value::Function(Function::Lambda(Lambda {
            params,
            body: body.into(),
        }));

        Ok(lambda)
    })
}
