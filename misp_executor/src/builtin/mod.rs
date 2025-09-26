// pub mod combinatorics;
// pub mod control;
pub mod func;
pub mod math;
// pub mod trig;

#[macro_export]
macro_rules! async_builtin {
    ($sync_name:ident, $async_name:ident) => {
        pub fn $sync_name(executor: *mut Executor) -> NativeMispFuture {
            Box::pin($async_name(executor))
        }
    };
}

#[macro_export]
macro_rules! arity_check {
    ($e:ident, $name:expr, $expected:expr) => {
        let Value::Decimal(arity) = $e.stack.pop().unwrap() else {
            return Err(Error::InvalidType);
        };

        let arity_int = arity.to_u128() as usize;
        if arity_int != $expected {
            use alloc::string::ToString;

            return Err(Error::FunctionArity {
                name: $name.to_string(),
                expected: $expected,
                actual: arity_int,
            });
        }
    };
}

#[macro_export]
macro_rules! quick_eval {
    ($e:ident, $pat:ident) => {{
        let val = $e.stack.pop().ok_or(Error::EmptyStack)?;
        match val {
            Value::$pat(inner_val) => inner_val,
            _ => match $e.eval(val).await? {
                Value::$pat(inner_val) => inner_val,
                _ => return Err(Error::InvalidType),
            },
        }
    }};
}
