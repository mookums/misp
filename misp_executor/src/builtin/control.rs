use crate::Error;
use crate::Executor;
use misp_parser::SExpr;
use num::BigInt;

pub fn builtin_if(executor: &mut Executor, args: &[SExpr]) -> Result<SExpr, Error> {
    if args.len() != 3 {
        return Err(Error::FunctionArity {
            name: "if".to_string(),
            expected: 3,
            actual: args.len(),
        });
    }

    // (if cond first second)
    let SExpr::Integer(condition) = executor.eval(&args[0])? else {
        return Err(Error::FunctionCall);
    };

    if condition != BigInt::ZERO {
        let first = executor.eval(&args[1])?;
        Ok(first)
    } else {
        let second = executor.eval(&args[2])?;
        Ok(second)
    }
}
