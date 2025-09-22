use crate::{Error, Executor, Injector, Instruction};

pub fn builtin_if(executor: &mut Executor) -> Result<(), Error> {
    let mut injector = Injector {
        instructions: &mut executor.instructions,
        index: 0,
    };

    injector.inject(Instruction::Push(executor.stack.pop().unwrap()));
    injector.inject(Instruction::Push(executor.stack.pop().unwrap()));
    Executor::inject_compiled(executor.stack.pop().unwrap(), &mut injector)?;
    injector.inject(Instruction::If);

    Ok(())
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
