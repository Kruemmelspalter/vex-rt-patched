use alloc::sync::{Arc, Weak};
use by_address::ByAddress;
use core::{cmp::min, fmt, time::Duration};
use owner_monad::OwnerMut;
use raii_map::set::{insert, Set, SetHandle};

use super::{
    handle_event, time_since_start, Event, EventHandle, GenericSleep, Instant, Mutex, Selectable,
};
use crate::select_merge;
use core::fmt::{Debug, Formatter};

type ContextValue = (Option<Instant>, Mutex<Option<ContextData>>);

#[derive(Clone)]
/// Represents an ongoing operation which could be cancelled in the future.
/// Inspired by contexts in the Go programming language.
///
/// # Concepts
///
/// Contexts have a few important concepts: "cancellation", "parent" and
/// "deadline". A context can be cancelled by calling its [`Context::cancel()`]
/// method; this notifies any tasks which are waiting on its [`Context::done()`]
/// event. It is also cancelled automatically if and when its parent context is
/// cancelled, and when the last copy of it goes out of scope. A "deadline"
/// allows a context to be automatically cancelled at a certain timestamp; this
/// is implemented without creating extra tasks/threads.
///
/// # Forking
///
/// A context can be "forked", which creates a new child context. This new
/// context can optionally be created with a deadline.
#[derive(Debug)]
pub struct Context(Arc<ContextValue>);

impl Context {
    /// Creates a new global context (i.e., one which has no parent or
    /// deadline).
    pub fn new_global() -> Self {
        Self(Arc::new((
            None,
            Mutex::new(Some(ContextData {
                _parent: None,
                event: Event::new(),
                children: Set::new(),
            })),
        )))
    }

    #[inline]
    /// Cancels a context. This is a no-op if the context is already cancelled.
    pub fn cancel(&self) {
        cancel(&self.0.as_ref().1);
    }

    #[inline]
    /// Forks a context. The new context's parent is `self`.
    pub fn fork(&self) -> Self {
        self.fork_internal(self.0 .0)
    }

    /// Forks a context. Equivalent to [`Context::fork()`], except that the new
    /// context has a deadline which is the earlier of the one in `self` and
    /// the one provided.
    pub fn fork_with_deadline(&self, deadline: Instant) -> Self {
        self.fork_internal(Some(self.0 .0.map_or(deadline, |d| min(d, deadline))))
    }

    #[inline]
    /// Forks a context. Equivalent to [`Context::fork_with_deadline()`], except
    /// that the deadline is calculated from the current time and the
    /// provided timeout duration.
    pub fn fork_with_timeout(&self, timeout: Duration) -> Self {
        self.fork_with_deadline(time_since_start() + timeout)
    }

    /// A [`Selectable`] event which occurs when the context is
    /// cancelled. The sleep amount takes the context deadline into
    /// consideration.
    pub fn done(&'_ self) -> impl Selectable + '_ {
        struct ContextSelect<'a>(&'a Context, EventHandle<ContextHandle>);

        impl<'a> Selectable for ContextSelect<'a> {
            fn poll(self) -> Result<(), Self> {
                let mut lock = self.0 .0 .1.lock();
                let opt = &mut lock.as_mut();
                if opt.is_some() {
                    if self.0 .0 .0.map_or(false, |v| v <= time_since_start()) {
                        opt.take();
                        Ok(())
                    } else {
                        Err(self)
                    }
                } else {
                    Ok(())
                }
            }
            fn sleep(&self) -> GenericSleep {
                GenericSleep::NotifyTake(self.0 .0 .0)
            }
        }

        ContextSelect(self, handle_event(ContextHandle(Arc::downgrade(&self.0))))
    }

    /// Creates a [`Selectable`] event which occurs when either the given
    /// `event` resolves, or when the context is cancelled, whichever occurs
    /// first.
    pub fn wrap<'a, T: 'a>(
        &'a self,
        event: impl Selectable<T> + 'a,
    ) -> impl Selectable<Option<T>> + 'a {
        select_merge! {
            r = event => Some(r),
            _ = self.done() => None,
        }
    }

    fn fork_internal(&self, deadline: Option<Instant>) -> Self {
        let ctx = Self(Arc::new((deadline, Mutex::new(None))));
        let parent_handle = insert(ContextHandle(Arc::downgrade(&self.0)), ctx.0.clone().into());
        if parent_handle.is_some() {
            *ctx.0 .1.lock() = Some(ContextData {
                _parent: parent_handle,
                event: Event::new(),
                children: Set::new(),
            });
        }
        ctx
    }
}

// TODO: Uncomment when `SetHandle` impls Debug
// #[derive(Debug)]
struct ContextData {
    _parent: Option<SetHandle<ByAddress<Arc<ContextValue>>, ContextHandle>>,
    event: Event,
    children: Set<ByAddress<Arc<ContextValue>>>,
}
impl Debug for ContextData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContextData")
            .field("_parent", &"...")
            .field("event", &self.event)
            .field("children", &self.children)
            .finish()
    }
}

impl Drop for ContextData {
    fn drop(&mut self) {
        self.event.notify();
        for child in self.children.iter() {
            cancel(&child.1)
        }
    }
}

struct ContextHandle(Weak<ContextValue>);

impl OwnerMut<Event> for ContextHandle {
    fn with<'a, U>(&'a mut self, f: impl FnOnce(&mut Event) -> U) -> Option<U>
    where
        Event: 'a,
    {
        Some(f(&mut self.0.upgrade()?.as_ref().1.lock().as_mut()?.event))
    }
}

impl OwnerMut<Set<ByAddress<Arc<ContextValue>>>> for ContextHandle {
    fn with<'a, U>(
        &'a mut self,
        f: impl FnOnce(&mut Set<ByAddress<Arc<ContextValue>>>) -> U,
    ) -> Option<U>
    where
        ContextValue: 'a,
    {
        Some(f(&mut self
            .0
            .upgrade()?
            .as_ref()
            .1
            .lock()
            .as_mut()?
            .children))
    }
}

#[inline]
fn cancel(m: &Mutex<Option<ContextData>>) {
    m.lock().take();
}

/// Provides a wrapper for [`Context`] objects which permits the management of
/// sequential, non-overlapping contexts.
#[derive(Debug)]
pub struct ContextWrapper(Option<Context>);

impl ContextWrapper {
    #[inline]
    /// Creates a new `ContextWrapper` objects.
    pub fn new() -> Self {
        Self(None)
    }

    /// Cancels the last context, creating a new global context in its place
    /// (which is returned).
    pub fn replace(&mut self) -> Context {
        if let Some(ctx) = self.0.take() {
            ctx.cancel();
        }
        let ctx = Context::new_global();
        self.0 = Some(ctx.clone());
        ctx
    }
}

impl Default for ContextWrapper {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
