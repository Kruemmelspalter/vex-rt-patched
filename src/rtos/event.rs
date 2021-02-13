use crate::{
    bindings,
    rtos::Task,
    util::{owner::Owner, shared_set::*},
};

/// Represents a self-maintaining set of tasks to notify when an event occurs.
pub struct Event(SharedSet<Task>);

impl Event {
    #[inline]
    /// Creates a new event structure with an empty set of tasks.
    pub fn new() -> Self {
        Event(SharedSet::new())
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
pub struct EventHandle<O: Owner<Event>>(Option<SharedSetHandle<Task, EventHandleOwner<O>>>);

impl<O: Owner<Event>> EventHandle<O> {
    /// Returns `true` if the event handle is orphaned, i.e. the parent event
    /// object no longer exists.
    pub fn is_done(&self) -> bool {
        self.with_owner(|_| ()).is_none()
    }

    /// Calls a given function on the underlying owner, if it exists.
    pub fn with_owner<U>(&self, f: impl FnOnce(&O) -> U) -> Option<U> {
        Some(f(&self.0.as_ref()?.owner().0))
    }

    /// Nullifies the handle. This has the same effect as dropping it or
    /// destroying the parent event object.
    pub fn clear(&mut self) {
        self.0.take();
    }
}

struct EventHandleOwner<O: Owner<Event>>(O);

impl<O: Owner<Event>> Owner<SharedSet<Task>> for EventHandleOwner<O> {
    #[inline]
    fn with<U>(&self, f: impl FnOnce(&mut SharedSet<Task>) -> U) -> Option<U> {
        self.0.with(|e| f(&mut e.0))
    }
}

#[inline]
/// Adds the current task to the notification set for an [`Event`], acquiring an
/// [`EventHandle`] to manage the lifetime of that entry.
pub fn handle_event<O: Owner<Event>>(owner: O) -> EventHandle<O> {
    EventHandle(insert(EventHandleOwner(owner), Task::current()))
}
