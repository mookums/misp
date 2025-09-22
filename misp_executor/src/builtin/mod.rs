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
