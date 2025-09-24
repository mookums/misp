pub mod combinatorics;
pub mod control;
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
    ($e:ident, $expected:expr) => {
        let Value::Decimal(arity) = $e.stack.pop().unwrap() else {
            return Err(Error::InvalidType);
        };

        let arity_int = arity.to_u128() as usize;
        if arity_int != $expected {
            return Err(Error::FunctionArity {
                expected: $expected,
                actual: arity_int,
            });
        }
    };
}
