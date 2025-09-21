use misp_num::decimal::Decimal;

use crate::{Error, Executor, Value};

pub fn builtin_if(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
    if args.len() != 3 {
        return Err(Error::FunctionArity {
            name: "if".to_string(),
            expected: 3,
            actual: args.len(),
        });
    }

    // (if cond first second)
    let Value::Decimal(condition) = executor.evaluate(&args[0])? else {
        return Err(Error::FunctionCall);
    };

    if condition != Decimal::ZERO {
        let first = executor.evaluate(&args[1])?;
        Ok(first)
    } else {
        let second = executor.evaluate(&args[2])?;
        Ok(second)
    }
}

pub fn builtin_let(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
    if args.len() != 3 {
        return Err(Error::FunctionArity {
            name: "let".to_string(),
            expected: 3,
            actual: args.len(),
        });
    }

    let Value::Atom(name) = &args[0] else {
        return Err(Error::FunctionCall);
    };

    let value = &args[1];

    executor.env.push_scope();

    executor.env.set(name, value.clone());
    let result = executor.evaluate(&args[2]);

    executor.env.pop_scope();
    result
}
