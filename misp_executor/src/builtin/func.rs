use crate::Error;
use crate::Executor;
use crate::environment::RuntimeMispFunction;
use misp_parser::SExpr;

pub fn builtin_func(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
    if args.len() != 3 {
        return Err(Error::FunctionArity {
            name: "func".to_string(),
            expected: 3,
            actual: args.len(),
        });
    }

    let SExpr::Atom(name) = &args[0] else {
        return Err(Error::FunctionCall);
    };

    let params = match &args[1] {
        SExpr::List(param_list) => {
            let mut params = Vec::new();
            for param in param_list {
                match param {
                    SExpr::Atom(p) => params.push(p.clone()),
                    _ => return Err(Error::FunctionCall),
                }
            }
            params
        }
        _ => return Err(Error::FunctionCall),
    };

    let body = args[2].clone();

    executor.env.set_function(
        name,
        RuntimeMispFunction {
            params,
            body: Box::new(body),
        },
    );

    Ok(SExpr::Atom(name.to_string()))
}

pub fn builtin_lambda(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
    if args.len() != 4 {
        return Err(Error::FunctionArity {
            name: "lambda".to_string(),
            expected: 4,
            actual: args.len(),
        });
    }

    let SExpr::Atom(name) = &args[0] else {
        return Err(Error::FunctionCall);
    };

    let params = match &args[1] {
        SExpr::List(param_list) => {
            let mut params = Vec::new();
            for param in param_list {
                match param {
                    SExpr::Atom(p) => params.push(p.clone()),
                    _ => return Err(Error::FunctionCall),
                }
            }
            params
        }
        _ => return Err(Error::FunctionCall),
    };

    let body = args[2].clone();

    executor.env.set_function(
        name,
        RuntimeMispFunction {
            params,
            body: Box::new(body),
        },
    );

    executor.eval(&args[3])
}
