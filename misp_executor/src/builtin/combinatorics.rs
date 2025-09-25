use alloc::boxed::Box;
use futures::join;
use misp_num::{Sign, decimal::Decimal};

use crate::{Error, Executor, NativeMispFuture, Value, arity_check};

pub fn builtin_combinations(executor: *mut Executor) -> NativeMispFuture {
    Box::pin(async move {
        let executor = unsafe { &mut *executor };
        arity_check!(executor, "combinations", 2);

        let (r_thunk, n_thunk) = (
            executor.stack.pop().ok_or(Error::EmptyStack)?,
            executor.stack.pop().ok_or(Error::EmptyStack)?,
        );

        let (n_value, r_value) = join!(executor.eval(n_thunk), executor.eval(r_thunk));

        let (Value::Decimal(n), Value::Decimal(r)) = (n_value?, r_value?) else {
            return Err(Error::InvalidType);
        };

        if !n.is_integer()
            || !r.is_integer()
            || n.sign == Sign::Negative
            || r.sign == Sign::Negative
        {
            return Err(Error::InvalidType);
        }

        let n_int = n.to_u128() as u64;
        let r_int = r.to_u128() as u64;

        if r_int > n_int {
            return Ok(Value::Decimal(Decimal::ZERO));
        }

        if r_int == 0 || r_int == n_int {
            return Ok(Value::Decimal(Decimal::ONE));
        }

        let r_optimized = r_int.min(n_int - r_int);
        let mut result = Decimal::ONE;

        for i in 0..r_optimized {
            result *= Decimal::from(n_int - i) / Decimal::from(i + 1);
        }

        Ok(Value::Decimal(result))
    })
}

pub fn builtin_permutations(executor: *mut Executor) -> NativeMispFuture {
    Box::pin(async move {
        let executor = unsafe { &mut *executor };
        arity_check!(executor, "permutations", 2);

        let (r_thunk, n_thunk) = (
            executor.stack.pop().ok_or(Error::EmptyStack)?,
            executor.stack.pop().ok_or(Error::EmptyStack)?,
        );

        let (n_value, r_value) = join!(executor.eval(n_thunk), executor.eval(r_thunk));

        let (Value::Decimal(n), Value::Decimal(r)) = (n_value?, r_value?) else {
            return Err(Error::InvalidType);
        };

        if !n.is_integer()
            || !r.is_integer()
            || n.sign == Sign::Negative
            || r.sign == Sign::Negative
        {
            return Err(Error::InvalidType);
        }

        let n_int = n.to_u128() as u64;
        let r_int = r.to_u128() as u64;

        if r_int > n_int {
            return Ok(Value::Decimal(Decimal::ZERO));
        }

        if r_int == 0 {
            return Ok(Value::Decimal(Decimal::ONE));
        }

        let mut result = Decimal::ONE;

        for i in 0..r_int {
            result *= Decimal::from(n_int - i);
        }

        Ok(Value::Decimal(result))
    })
}

pub fn builtin_factorial(executor: *mut Executor) -> NativeMispFuture {
    Box::pin(async move {
        let executor = unsafe { &mut *executor };
        arity_check!(executor, "factorial", 1);

        let value = executor.stack.pop().ok_or(Error::EmptyStack)?;

        let Value::Decimal(n) = executor.eval(value).await? else {
            return Err(Error::InvalidType);
        };

        Ok(Value::Decimal(n.factorial()))
    })
}
