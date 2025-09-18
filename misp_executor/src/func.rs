use crate::Executor;
use crate::{Error, Function};
use misp_parser::SExpr;

pub fn builtin_func(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
    if args.len() != 3 {
        return Err(Error::FunctionCall);
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

    executor.env.functions.insert(
        name.to_string(),
        Function::UserDefined {
            params,
            body: Box::new(body),
        },
    );

    Ok(SExpr::Atom(name.to_string()))
}
