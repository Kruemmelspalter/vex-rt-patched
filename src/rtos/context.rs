use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use by_address::ByAddress;
use core::{cmp::min, time::Duration};
use owner_monad::OwnerMut;
use raii_map::set::{insert, Set, SetHandle};

use super::{
    handle_event, time_since_start, Event, EventHandle, GenericSleep, Instant, Mutex, Selectable,
};
use crate::select_merge;

type ContextValue = (Option<Instant>, Mutex<Option<ContextData>>);

#[derive(Clone)]
#[repr(transparent)]
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
pub struct Context(Arc<ContextValue>);

impl Context {
    #[inline]
    /// Creates a new global context (i.e., one which has no parent or
    /// deadline).
    pub fn new_global() -> Self {
        Self::new_internal(&[], None)
    }

    #[inline]
    /// Cancels a context. This is a no-op if the context is already cancelled.
    pub fn cancel(&self) {
        cancel(&self.0.as_ref().1);
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

    fn new_internal(parents: &[&Self], mut deadline: Option<Instant>) -> Self {
        deadline = parents
            .iter()
            .filter_map(|parent| parent.0 .0)
            .min()
            .map_or(deadline, |d1| Some(deadline.map_or(d1, |d2| min(d1, d2))));
        let ctx = Self(Arc::new((deadline, Mutex::new(None))));
        let mut parent_handles = Vec::new();
        parent_handles.reserve_exact(parents.len());
        for parent in parents {
            if let Some(handle) = insert(
                ContextHandle(Arc::downgrade(&parent.0)),
                ctx.0.clone().into(),
            ) {
                parent_handles.push(handle);
            } else {
                return ctx;
            }
        }
        *ctx.0 .1.lock() = Some(ContextData {
            _parents: parent_handles,
            event: Event::new(),
            children: Set::new(),
        });
        ctx
    }
}

/// Describes an object from which a child context can be created. Implemented
/// for contexts and for slices of contexts.
pub trait ParentContext {
    /// Forks a context. The new context's parent(s) are `self`.
    fn fork(&self) -> Context;

    /// Forks a context. Equivalent to [`Self::fork()`], except that the new
    /// context has a deadline which is the earliest of those in `self` and
    /// the one provided.
    fn fork_with_deadline(&self, deadline: Instant) -> Context;

    /// Forks a context. Equivalent to [`Self::fork_with_deadline()`], except
    /// that the deadline is calculated from the current time and the
    /// provided timeout duration.
    fn fork_with_timeout(&self, timeout: Duration) -> Context {
        self.fork_with_deadline(time_since_start() + timeout)
    }
}

impl ParentContext for Context {
    #[inline]
    fn fork(&self) -> Context {
        [self].fork()
    }

    #[inline]
    fn fork_with_deadline(&self, deadline: Instant) -> Context {
        [self].fork_with_deadline(deadline)
    }
}

impl ParentContext for [&Context] {
    #[inline]
    fn fork(&self) -> Context {
        Context::new_internal(self, None)
    }

    #[inline]
    fn fork_with_deadline(&self, deadline: Instant) -> Context {
        Context::new_internal(self, Some(deadline))
    }
}

struct ContextData {
    _parents: Vec<SetHandle<ByAddress<Arc<ContextValue>>, ContextHandle>>,
    event: Event,
    children: Set<ByAddress<Arc<ContextValue>>>,
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

fn cancel(m: &Mutex<Option<ContextData>>) {
    m.lock().take();
}

/// Provides a wrapper for [`Context`] objects which permits the management of
/// sequential, non-overlapping contexts.
pub struct ContextWrapper(Option<Context>);

impl ContextWrapper {
    #[inline]
    /// Creates a new `ContextWrapper` objects.
    pub fn new() -> Self {
        Self(None)
    }

    /// Gets the current context, if one exists.
    pub fn current(&self) -> Option<&Context> {
        self.0.as_ref()
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
