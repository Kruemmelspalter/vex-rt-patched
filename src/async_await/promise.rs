use alloc::rc::Rc;
use core::{
    cell::RefCell,
    pin::Pin,
    task::{Context, Poll, Waker},
};
use futures::Future;
use replace_with::{replace_with_or_abort, replace_with_or_abort_and_return};

/// A thread-local promise with a separate resolve function whose lifetime is
/// not constrained to that of the promise.
pub struct Promise<T>(Rc<RefCell<PromiseState<T>>>);

impl<T> Promise<T> {
    /// Creates a new promise and resolve function pair.
    pub fn new() -> (Self, impl FnOnce(T)) {
        let state = Rc::new(RefCell::new(PromiseState::New));
        let ptr = Rc::downgrade(&state);
        (Self(state), move |value| {
            if let Some(state) = ptr.upgrade() {
                replace_with_or_abort(&mut *state.borrow_mut(), |s| {
                    if let PromiseState::Pending(waker) = s {
                        waker.wake();
                    }
                    PromiseState::Done(value)
                })
            }
        })
    }
}

impl<T> Future for Promise<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        replace_with_or_abort_and_return(&mut *self.0.borrow_mut(), |state| {
            if let PromiseState::Done(value) = state {
                (Poll::Ready(value), PromiseState::New)
            } else {
                (Poll::Pending, PromiseState::Pending(cx.waker().clone()))
            }
        })
    }
}

enum PromiseState<T> {
    New,
    Pending(Waker),
    Done(T),
}
