use owner_monad::{Owner, OwnerMut};
use raii_map::set::{insert, Set, SetHandle};

use crate::{bindings, rtos::Task};

/// Represents a self-maintaining set of tasks to notify when an event occurs.
pub struct Event(Set<Task>);

impl Event {
    #[inline]
    /// Creates a new event structure with an empty set of tasks.
    pub fn new() -> Self {
        Event(Set::new())
    }

    /// Notify the tasks which are waiting for the event.
    pub fn notify(&self) {
        for t in self.0.iter() {
            unsafe { bindings::task_notify(t.0) };
        }
    }

    #[inline]
    /// Gets the number of tasks currently waiting for the event.
    pub fn task_count(&self) -> usize {
        self.0.len()
    }
}

impl Default for Event {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a handle into the listing of the current task in an [`Event`].
/// When this handle is dropped, that task is removed from the event's set.
pub struct EventHandle<O: OwnerMut<Event>>(Option<SetHandle<Task, EventHandleOwner<O>>>);

impl<O: OwnerMut<Event>> EventHandle<O> {
    /// Returns `true` if the event handle is orphaned, i.e. the parent event
    /// object no longer exists.
    pub fn is_done(&self) -> bool {
        self.with(|_| ()).is_none()
    }

    /// Nullifies the handle. This has the same effect as dropping it or
    /// destroying the parent event object.
    pub fn clear(&mut self) {
        self.0.take();
    }
}

impl<O: OwnerMut<Event>> Owner<O> for EventHandle<O> {
    fn with<'a, U>(&'a self, f: impl FnOnce(&O) -> U) -> Option<U>
    where
        O: 'a,
    {
        self.0.as_ref()?.with(|o| f(&o.0))
    }
}

#[repr(transparent)]
struct EventHandleOwner<O: OwnerMut<Event>>(O);

impl<O: OwnerMut<Event>> OwnerMut<Set<Task>> for EventHandleOwner<O> {
    fn with<'a, U>(&'a mut self, f: impl FnOnce(&mut Set<Task>) -> U) -> Option<U>
    where
        Event: 'a,
    {
        self.0.with(|e| f(&mut e.0))
    }
}

#[inline]
/// Adds the current task to the notification set for an [`Event`], acquiring an
/// [`EventHandle`] to manage the lifetime of that entry.
pub fn handle_event<O: OwnerMut<Event>>(owner: O) -> EventHandle<O> {
    EventHandle(insert(EventHandleOwner(owner), Task::current()))
}
