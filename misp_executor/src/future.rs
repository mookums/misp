use core::{
    pin::Pin,
    task::{Poll, RawWaker, RawWakerVTable, Waker},
};

use alloc::{rc::Rc, task::Wake};

use crate::{Error, Executor, Value};

#[derive(Debug, Default)]
pub struct EvalFutureContext {
    pub result: Option<Result<Value, Error>>,
    pub waker: Option<Waker>,
}

impl EvalFutureContext {
    pub fn new() -> Self {
        Self {
            result: None,
            waker: None,
        }
    }
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

        let future_data = executor
            .futures
            .get_mut(&fut.id)
            .expect("Future must exist");

        if let Some(result) = future_data.result.take() {
            if let Some(waker) = future_data.waker.take() {
                waker.wake();
            }

            executor.futures.remove(&fut.id);
            Poll::Ready(result)
        } else {
            future_data.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct EvalWakerData {
    pub executor: *mut Executor,
    pub future_id: usize,
}

unsafe fn wake_eval(data: *const ()) {
    let waker_data = unsafe { Rc::from_raw(data as *const EvalWakerData) };
    let executor = unsafe { &mut *waker_data.executor };
    executor.ready_future = Some(waker_data.future_id);
    core::mem::drop(waker_data);
}

unsafe fn wake_eval_by_ref(data: *const ()) {
    let waker_data = unsafe { &*(data as *const EvalWakerData) };
    let executor = unsafe { &mut *waker_data.executor };
    {
        extern crate std;
        use std::eprintln;
        eprintln!("Resuming Future with Id: {}", waker_data.future_id);
    }
    executor.ready_future = Some(waker_data.future_id);
}

unsafe fn clone_eval_waker(data: *const ()) -> RawWaker {
    let rc = unsafe { Rc::from_raw(data as *const EvalWakerData) };
    let cloned = rc.clone();
    core::mem::forget(rc);
    RawWaker::new(Rc::into_raw(cloned) as *const (), &EVAL_WAKER_VTABLE)
}

unsafe fn drop_eval_waker(data: *const ()) {
    unsafe { Rc::from_raw(data as *const EvalWakerData) };
}

const EVAL_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    clone_eval_waker,
    wake_eval,
    wake_eval_by_ref,
    drop_eval_waker,
);

pub fn create_eval_waker(exec: *mut Executor, future_id: usize) -> Waker {
    let waker_data = Rc::new(EvalWakerData {
        executor: exec,
        future_id,
    });

    let raw_waker = RawWaker::new(Rc::into_raw(waker_data) as *const (), &EVAL_WAKER_VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}
