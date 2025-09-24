use core::{
    pin::Pin,
    task::{Context, Poll, Waker},
};

use crate::{Error, Executor, Value};

#[derive(Debug, Default)]
pub struct EvalFutureContext<
    const MAX_STR: usize,
    const MAX_TOKENS: usize,
    const MAX_LIST: usize,
    const MAX_INTERN: usize,
    const MAX_INSTRUCTIONS: usize,
    const MAX_STACK: usize,
    const MAX_MEMOS: usize,
    const MAX_FUTURES: usize,
> {
    pub result: Option<
        Result<
            Value<
                MAX_STR,
                MAX_TOKENS,
                MAX_LIST,
                MAX_INTERN,
                MAX_INSTRUCTIONS,
                MAX_STACK,
                MAX_MEMOS,
                MAX_FUTURES,
            >,
            Error,
        >,
    >,
    pub waker: Option<Waker>,
}

pub struct EvalFuture<
    const MAX_STR: usize,
    const MAX_TOKENS: usize,
    const MAX_LIST: usize,
    const MAX_INTERN: usize,
    const MAX_INSTRUCTIONS: usize,
    const MAX_STACK: usize,
    const MAX_MEMOS: usize,
    const MAX_FUTURES: usize,
> {
    pub id: usize,
    pub executor: *mut Executor<
        MAX_STR,
        MAX_TOKENS,
        MAX_LIST,
        MAX_INTERN,
        MAX_INSTRUCTIONS,
        MAX_STACK,
        MAX_MEMOS,
        MAX_FUTURES,
    >,
}

impl<
    const MAX_STR: usize,
    const MAX_TOKENS: usize,
    const MAX_LIST: usize,
    const MAX_INTERN: usize,
    const MAX_INSTRUCTIONS: usize,
    const MAX_STACK: usize,
    const MAX_MEMOS: usize,
    const MAX_FUTURES: usize,
> Future
    for EvalFuture<
        MAX_STR,
        MAX_TOKENS,
        MAX_LIST,
        MAX_INTERN,
        MAX_INSTRUCTIONS,
        MAX_STACK,
        MAX_MEMOS,
        MAX_FUTURES,
    >
{
    type Output = Result<
        Value<
            MAX_STR,
            MAX_TOKENS,
            MAX_LIST,
            MAX_INTERN,
            MAX_INSTRUCTIONS,
            MAX_STACK,
            MAX_MEMOS,
            MAX_FUTURES,
        >,
        Error,
    >;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let fut = self.get_mut();
        let executor = unsafe { &mut *fut.executor };

        if let Some(future_data) = executor.futures.get_mut(&fut.id) {
            if let Some(result) = future_data.result.take() {
                executor.futures.remove(&fut.id);
                Poll::Ready(result)
            } else {
                future_data.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        } else {
            panic!("Future {} not found", fut.id)
        }
    }
}
