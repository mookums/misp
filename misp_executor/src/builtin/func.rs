use crate::{Error, Executor, Function, Injector, Instruction, Lambda, RuntimeMispFunction, Value};

pub fn builtin_func(executor: &mut Executor) -> Result<(), Error> {
    let mut injector = Injector {
        instructions: &mut executor.instructions,
        index: 0,
    };

    let body = executor.stack.pop().unwrap();

    let params = match executor.stack.pop().unwrap() {
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

    let Value::Atom(name) = executor.stack.pop().unwrap() else {
        return Err(Error::FunctionCall);
    };

    let function = Value::Function(Function::Runtime(RuntimeMispFunction {
        params,
        body: Box::new(body),
    }));

    injector.inject(Instruction::Push(function.clone()));
    injector.inject(Instruction::Store(name));
    injector.inject(Instruction::Push(function));

    Ok(())
}

pub fn builtin_lambda(executor: &mut Executor) -> Result<(), Error> {
    let mut injector = Injector {
        instructions: &mut executor.instructions,
        index: 0,
    };

    let body = executor.stack.pop().unwrap();

    let params = match executor.stack.pop().unwrap() {
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

    injector.inject(Instruction::Push(lambda));
    Ok(())
}
