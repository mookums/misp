use misp_num::decimal::Decimal;

use crate::{Error, Executor, Value};

pub fn builtin_if(executor: &mut Executor) -> Result<Value, Error> {
    let (else_thunk, then_thunk, condition_thunk) = (
        executor.stack.pop().ok_or(Error::EmptyStack)?,
        executor.stack.pop().ok_or(Error::EmptyStack)?,
        executor.stack.pop().ok_or(Error::EmptyStack)?,
    );

    let Value::Decimal(condition) = executor.eval(condition_thunk)? else {
        return Err(Error::InvalidType);
    };

    let next = if condition == Decimal::ONE {
        executor.eval(then_thunk)?
    } else {
        executor.eval(else_thunk)?
    };

    Ok(next)
}

// pub fn builtin_let(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
//     if args.len() != 3 {
//         return Err(Error::FunctionArity {
//             name: "let".to_string(),
//             expected: 3,
//             actual: args.len(),
//         });
//     }

//     let Value::Atom(name) = &args[0] else {
//         return Err(Error::FunctionCall);
//     };

//     let value = &args[1];

//     executor.env.push_scope();

//     executor.env.set(name, value.clone());
//     let result = executor.evaluate(&args[2]);

//     executor.env.pop_scope();
//     result
// }
