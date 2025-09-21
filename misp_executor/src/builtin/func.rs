use crate::Error;
use crate::Executor;
use crate::Function;
use crate::Lambda;
use crate::RuntimeMispFunction;
use crate::Value;

pub fn builtin_func(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
    if args.len() != 3 {
        return Err(Error::FunctionArity {
            name: "func".to_string(),
            expected: 3,
            actual: args.len(),
        });
    }

    let Value::Atom(name) = &args[0] else {
        return Err(Error::FunctionCall);
    };

    let params = match &args[1] {
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

    let body = args[2].clone();

    let function = Value::Function(Function::Runtime(RuntimeMispFunction {
        params,
        body: Box::new(body),
    }));

    executor.env.set(name, function.clone());
    Ok(function)
}

pub fn builtin_let_func(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
    if args.len() != 4 {
        return Err(Error::FunctionArity {
            name: "letFunc".to_string(),
            expected: 4,
            actual: args.len(),
        });
    }

    let Value::Atom(name) = &args[0] else {
        return Err(Error::FunctionCall);
    };

    let params = match &args[1] {
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

    let body = args[2].clone();

    let function = Value::Function(Function::Runtime(RuntimeMispFunction {
        params,
        body: Box::new(body),
    }));

    executor.env.push_scope();

    executor.env.set(name, function.clone());
    let result = executor.eval(&args[3]);

    executor.env.pop_scope();
    result
}

pub fn builtin_lambda(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
    if args.len() != 2 {
        return Err(Error::FunctionArity {
            name: "lambda".to_string(),
            expected: 2,
            actual: args.len(),
        });
    }

    let params = match &args[0] {
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

    let body = args[1].clone();

    Ok(Value::Function(Function::Lambda(Lambda {
        params,
        body: Box::new(body),
        scope: executor.env.current_scope().clone(),
    })))
}
