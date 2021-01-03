use core::cell::UnsafeCell;

use alloc::sync::{Arc, Weak};

use super::{handle_event, Event, EventHandle, GenericSleep, Mutex, Selectable, Task};
use crate::{error::Error, util::owner::Owner};

#[derive(Clone)]
/// Represents an ongoing operation which produces a result.
pub struct Promise<T: 'static = ()>(Arc<Mutex<PromiseData<T>>>);

impl<T: 'static> Promise<T> {
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
    pub fn new() -> (Self, impl FnOnce(T)) {
        let data = Arc::new(Mutex::new(PromiseData::Incomplete(Event::new())));
        let promise = Self(data.clone());
        let resolve = move |r: T| {
            let mut l = data.lock();
            if let Some(e) = l.event() {
                e.notify();
                *l = PromiseData::Complete(r.into());
            }
        };
        (promise, resolve)
    }

    /// A [`Selectable`] event which occurs when the promise is resolved.
    pub fn done(&'_ self) -> impl Selectable<&'_ T> + '_ {
        struct PromiseSelect<'a, T: 'static> {
            promise: &'a Promise<T>,
            _handle: EventHandle<PromiseHandle<T>>,
        };

        impl<'a, T> Selectable<&'a T> for PromiseSelect<'a, T> {
            fn poll(self) -> Result<&'a T, Self> {
                self.promise
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
                GenericSleep::NotifyTake(None)
            }
        }

        PromiseSelect {
            promise: self,
            _handle: handle_event(PromiseHandle(Arc::downgrade(&self.0))),
        }
    }
}

impl<T: Send + Sync + 'static> Promise<T> {
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

struct PromiseHandle<T>(Weak<Mutex<PromiseData<T>>>);

impl<T> Owner<Event> for PromiseHandle<T> {
    fn with<U>(&self, f: impl FnOnce(&mut Event) -> U) -> Option<U> {
        Some(f(self.0.upgrade()?.as_ref().lock().event()?))
    }
}
