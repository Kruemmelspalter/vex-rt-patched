use alloc::{
    format,
    string::String,
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

#[cfg(feature = "logging")]
use super::Task;

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
        Self::new_internal(&[], None, None)
    }

    /// Construct a new global context, with additional options.
    pub fn new_global_ext(deadline: Option<Instant>, name: Option<String>) -> Self {
        Self::new_internal(&[], deadline, name)
    }

    #[inline]
    /// Cancels a context. This is a no-op if the context is already cancelled.
    pub fn cancel(&self) {
        #[cfg(feature = "logging")]
        log::debug!("Explicit cancel: {}", self.name());
        cancel(&self.0.data);
    }

    /// Gets the name of the context.
    pub fn name(&self) -> &str {
        self.0.name()
    }

    /// A [`Selectable`] event which occurs when the context is
    /// cancelled. The sleep amount takes the context deadline into
    /// consideration.
    pub fn done(&'_ self) -> impl Selectable<Output = ()> + '_ {
        enum ContextSelect<'a> {
            Waiting(&'a Context, EventHandle<ContextHandle>),
            AlreadyDone,
        }

        impl<'a> Selectable for ContextSelect<'a> {
            type Output = ();

            fn poll(self) -> Result<(), Self> {
                match self {
                    ContextSelect::Waiting(ctx, _) => {
                        let mut lock = ctx.0.data.lock();
                        let opt = &mut lock.as_mut();
                        if opt.is_some() {
                            if ctx.0.deadline.map_or(false, |v| v <= time_since_start()) {
                                #[cfg(feature = "logging")]
                                log::trace!(
                                    "Task {} detected timeout on context {}",
                                    Task::current().name(),
                                    ctx.name(),
                                );

                                opt.take();
                                Ok(())
                            } else {
                                Err(self)
                            }
                        } else {
                            #[cfg(feature = "logging")]
                            log::trace!(
                                "Task {} finished waiting on context {}",
                                Task::current().name(),
                                ctx.name(),
                            );

                            Ok(())
                        }
                    }
                    ContextSelect::AlreadyDone => Ok(()),
                }
            }

            fn sleep(&self) -> GenericSleep {
                match self {
                    ContextSelect::Waiting(ctx, _) => GenericSleep::NotifyTake(ctx.0.deadline),
                    ContextSelect::AlreadyDone => GenericSleep::Ready,
                }
            }
        }

        #[cfg(feature = "logging")]
        log::trace!(
            "Task {} waiting on context {}",
            Task::current().name(),
            self.name(),
        );

        // Keep locked through call to handle_event to avoid race conditions.
        let lock = self.0.data.lock();

        if lock.is_some() {
            ContextSelect::Waiting(self, handle_event(ContextHandle(Arc::downgrade(&self.0))))
        } else {
            ContextSelect::AlreadyDone
        }
    }

    /// Creates a [`Selectable`] event which occurs when either the given
    /// `event` resolves, or when the context is cancelled, whichever occurs
    /// first.
    pub fn wrap<'a, T: 'a>(
        &'a self,
        event: impl Selectable<Output = T> + 'a,
    ) -> impl Selectable<Output = Option<T>> + 'a {
        select_merge! {
            r = event => Some(r),
            _ = self.done() => None,
        }
    }

    fn new_internal(
        parents: &[&Self],
        mut deadline: Option<Instant>,
        name: Option<String>,
    ) -> Self {
        deadline = parents
            .iter()
            .filter_map(|parent| parent.0.deadline)
            .min()
            .map_or(deadline, |d1| Some(deadline.map_or(d1, |d2| min(d1, d2))));
        let ctx = Self(Arc::new(ContextValue {
            deadline,
            name,
            data: Mutex::new(None),
        }));
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
        *ctx.0.data.lock() = Some(ContextData {
            _parents: parent_handles,
            event: Event::new(),
            children: Set::new(),
        });
        ctx
    }
}

struct ContextValue {
    deadline: Option<Instant>,
    name: Option<String>,
    data: Mutex<Option<ContextData>>,
}

impl ContextValue {
    fn name(&self) -> &str {
        self.name.as_ref().map_or("<anon>", String::as_str)
    }
}

/// Describes an object from which a child context can be created. Implemented
/// for contexts and for slices of contexts.
pub trait ParentContext {
    /// Forks a context, with the given options. The new context's parent(s) are
    /// `self`.
    fn fork_ext(&self, deadline: Option<Instant>, name: Option<String>) -> Context;

    /// Forks a context. The new context's parent(s) are `self`.
    fn fork(&self) -> Context {
        self.fork_ext(None, None)
    }

    /// Forks a context. Equivalent to [`Self::fork()`], except that the new
    /// context has a deadline which is the earliest of those in `self` and
    /// the one provided.
    fn fork_with_deadline(&self, deadline: Instant) -> Context {
        self.fork_ext(Some(deadline), None)
    }

    /// Forks a context. Equivalent to [`Self::fork_with_deadline()`], except
    /// that the deadline is calculated from the current time and the
    /// provided timeout duration.
    fn fork_with_timeout(&self, timeout: Duration) -> Context {
        self.fork_with_deadline(time_since_start() + timeout)
    }
}

impl ParentContext for Context {
    #[inline]
    fn fork_ext(&self, deadline: Option<Instant>, name: Option<String>) -> Context {
        [self].fork_ext(deadline, name)
    }
}

impl ParentContext for Option<&Context> {
    fn fork_ext(&self, deadline: Option<Instant>, name: Option<String>) -> Context {
        if let Some(ctx) = self {
            [*ctx].fork_ext(deadline, name)
        } else {
            [].fork_ext(deadline, name)
        }
    }
}

impl ParentContext for [&Context] {
    #[inline]
    fn fork_ext(&self, deadline: Option<Instant>, name: Option<String>) -> Context {
        Context::new_internal(self, deadline, name)
    }
}

// TODO: Uncomment when `SetHandle` impls Debug
// #[derive(Debug)]
struct ContextData {
    _parents: Vec<SetHandle<ByAddress<Arc<ContextValue>>, ContextHandle>>,
    event: Event,
    children: Set<ByAddress<Arc<ContextValue>>>,
}

impl Drop for ContextData {
    fn drop(&mut self) {
        self.event.notify();
        for child in self.children.iter() {
            #[cfg(feature = "logging")]
            log::trace!("Indirect cancel: {}", child.name());
            cancel(&child.data)
        }
    }
}

#[repr(transparent)]
struct ContextHandle(Weak<ContextValue>);

impl OwnerMut<Event> for ContextHandle {
    fn with<'a, U>(&'a mut self, f: impl FnOnce(&mut Event) -> U) -> Option<U>
    where
        Event: 'a,
    {
        Some(f(&mut self
            .0
            .upgrade()?
            .as_ref()
            .data
            .lock()
            .as_mut()?
            .event))
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
            .data
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
pub struct ContextWrapper {
    ctx: Option<Context>,
    name_and_count: Option<(String, usize)>,
}

impl ContextWrapper {
    #[inline]
    /// Creates a new `ContextWrapper` objects.
    pub fn new() -> Self {
        Self {
            ctx: None,
            name_and_count: None,
        }
    }

    /// Creates a new `ContextWrapper` object with the given properties.
    pub fn new_ext(name: Option<String>) -> Self {
        Self {
            ctx: None,
            name_and_count: name.map(|name| (name, 0)),
        }
    }

    /// Gets the current context, if one exists.
    pub fn current(&self) -> Option<&Context> {
        self.ctx.as_ref()
    }

    /// Cancels the last context, creating a new global context in its place
    /// (which is returned).
    pub fn replace(&mut self) -> Context {
        self.replace_ext([].as_slice())
    }

    /// Cancels the last context, creating a new context as a child of the given
    /// context in its place.
    pub fn replace_ext(&mut self, ctx: &(impl ParentContext + ?Sized)) -> Context {
        if let Some(ctx) = self.ctx.take() {
            ctx.cancel();
        }
        let ctx = ctx.fork_ext(None, self.next_name());
        self.ctx = Some(ctx.clone());
        ctx
    }

    fn next_name(&mut self) -> Option<String> {
        if let Some((name, count)) = &mut self.name_and_count {
            let name = format!("{}.{}", name, count);
            *count += 1;
            Some(name)
        } else {
            None
        }
    }
}

impl Default for ContextWrapper {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
