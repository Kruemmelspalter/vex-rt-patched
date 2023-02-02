use core::cell::UnsafeCell;

use alloc::{
    string::String,
    sync::{Arc, Weak},
};
use owner_monad::OwnerMut;

use super::{
    handle_event, select, Context, Event, EventHandle, GenericSleep, Mutex, Selectable, Task,
};
use crate::{error::Error, select};

/// Represents an ongoing operation which produces a result.
pub struct Promise<T: 'static = ()>(Arc<(Mutex<PromiseData<T>>, Option<String>)>);

impl<T: Send + Sync + 'static> Promise<T> {
    /// Creates a new lightweight promise and an associated resolve function.
    ///
    /// # Example
    /// ```
    /// let (promise, resolve) = Promise::<i32>::new();
    /// Task::spawn(|| {
    ///     Task::delay(Duration::from_secs(1));
    ///     resolve(10);
    /// })
    /// .unwrap();
    /// println!(
    ///     "n = {}",
    ///     select! {
    ///         n = promise.done() => n,
    ///     }
    /// );
    /// ```
    pub fn new() -> (Self, impl FnOnce(T) + Send) {
        Self::new_ext(None)
    }

    /// Creates a new lightweight promise and associated resolve function, with
    /// the given options.
    pub fn new_ext(name: Option<String>) -> (Self, impl FnOnce(T) + Send) {
        let data = Arc::new((Mutex::new(PromiseData::Incomplete(Event::new())), name));
        let promise = Self(data.clone());
        let resolve = move |r: T| {
            #[cfg(feature = "logging")]
            log::debug!(
                "resolving promise: {}",
                data.1.as_deref().unwrap_or("<anon>")
            );

            let mut l = data.0.lock();
            if let Some(e) = l.event() {
                e.notify();
                *l = PromiseData::Complete(r.into());
            }
        };
        (promise, resolve)
    }

    /// Gets the name of the promise, if there is one.
    pub fn name(&self) -> Option<&str> {
        self.0 .1.as_deref()
    }

    /// A [`Selectable`] event which occurs when the promise is resolved.
    pub fn done(&self) -> impl Selectable<Output = &T> + '_ {
        struct PromiseSelect<'a, T: 'static> {
            promise: &'a Promise<T>,
            handle: EventHandle<PromiseHandle<T>>,
        }

        impl<'a, T> Selectable for PromiseSelect<'a, T> {
            type Output = &'a T;

            fn poll(self) -> Result<Self::Output, Self> {
                self.promise
                    .0
                     .0
                    .lock()
                    .result()
                    // This is safe, since the promise can only be resolved once (thanks to the
                    // resolve function being FnOnce) and the promise's contract and API ensure that
                    // the result is never modified once set. The reference is safe since it is
                    // defined to last only as long as the `Context` object, which has an `Arc`
                    // reference to the object containing the result.
                    .map(|r| unsafe { &*UnsafeCell::<T>::raw_get(r) })
                    .ok_or(self)
            }

            #[inline]
            fn sleep(&self) -> GenericSleep {
                if self.handle.is_done() {
                    GenericSleep::Ready
                } else {
                    GenericSleep::NotifyTake(None)
                }
            }
        }

        PromiseSelect {
            promise: self,
            handle: handle_event(PromiseHandle(Arc::downgrade(&self.0))),
        }
    }

    #[inline]
    /// Spawns a task to run the given function and returns a [`Promise`] that
    /// resolves with the result when it returns. Panics on failure; see
    /// [`Promise::try_spawn()`].
    pub fn spawn(f: impl FnOnce() -> T + Send + 'static) -> Self {
        Self::try_spawn(f).unwrap()
    }

    /// Spawns a task to run the given function and returns a [`Promise`] that
    /// resolves with the result when it returns.
    pub fn try_spawn(f: impl FnOnce() -> T + Send + 'static) -> Result<Self, Error> {
        let (promise, resolve) = Self::new();
        Task::spawn(|| resolve(f()))?;
        Ok(promise)
    }

    #[inline]
    /// Spawns a new promise which, upon the completion of `self`, runs `f`.
    pub fn then<U: Send + Sync + 'static>(
        &self,
        f: impl FnOnce(&T) -> U + Send + 'static,
    ) -> Promise<U> {
        let upstream = self.clone();
        Promise::spawn(move || f(select(upstream.done())))
    }

    #[inline]
    /// Spawns a new promise which, upon the completion of `self`, runs `f`. If
    /// `ctx` is cancelled before `self` completes, the result is `default`.
    pub fn then_or<U: Send + Sync + 'static>(
        &self,
        ctx: Context,
        default: U,
        f: impl FnOnce(&T) -> U + Send + 'static,
    ) -> Promise<U> {
        let upstream = self.clone();
        Promise::spawn(move || {
            select! {
                v = upstream.done() => f(v),
                _ = ctx.done() => default,
            }
        })
    }

    #[inline]
    /// Spawns a new promise which, upon the completion of `self`, runs `f`. If
    /// `ctx` is cancelled before `self` completes, the result is `None`.
    pub fn then_some<U: Send + Sync + 'static>(
        &self,
        ctx: Context,
        f: impl FnOnce(&T) -> U + Send + 'static,
    ) -> Promise<Option<U>> {
        self.then_or(ctx, None, |v| Some(f(v)))
    }
}

impl<T: 'static> Clone for Promise<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

enum PromiseData<T> {
    Incomplete(Event),
    Complete(UnsafeCell<T>),
}

impl<T> PromiseData<T> {
    #[inline]
    fn event(&mut self) -> Option<&mut Event> {
        match self {
            PromiseData::Incomplete(e) => Some(e),
            PromiseData::Complete(_) => None,
        }
    }

    #[inline]
    fn result(&self) -> Option<&UnsafeCell<T>> {
        match self {
            PromiseData::Incomplete(_) => None,
            PromiseData::Complete(r) => Some(r),
        }
    }
}

unsafe impl<T: Send> Send for PromiseData<T> {}

unsafe impl<T: Sync> Sync for PromiseData<T> {}

struct PromiseHandle<T>(Weak<(Mutex<PromiseData<T>>, Option<String>)>);

impl<T> OwnerMut<Event> for PromiseHandle<T> {
    fn with<'a, U>(&'a mut self, f: impl FnOnce(&mut Event) -> U) -> Option<U>
    where
        Event: 'a,
    {
        Some(f(self.0.upgrade()?.0.lock().event()?))
    }
}
