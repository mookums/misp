use core::{
    pin::Pin,
    task::{Poll, Waker},
};

use crate::{Error, Executor, Value};

#[derive(Debug, Default)]
pub struct EvalFutureContext {
    pub result: Option<Result<Value, Error>>,
    pub waker: Option<Waker>,
}

pub struct EvalFuture {
    pub id: usize,
    pub executor: *mut Executor,
}

impl Future for EvalFuture {
    type Output = Result<Value, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
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
