use alloc::vec::Vec;
use core::{marker::PhantomData, mem::size_of, ptr::null_mut, time::Duration};

use libc::c_void;

use crate::{
    bindings,
    error::{from_errno, Error},
};

use super::{handle_event, Event, EventHandle, GenericSleep, Mutex, Selectable, TIMEOUT_MAX};

/// Represents a FreeRTOS FIFO queue.
///
/// When multiple tasks are simultaneously waiting to send or receive on the
/// same queue, there are no relative ordering guarantees.
pub struct Queue<T: Copy + Send> {
    queue: bindings::queue_t,
    event: Mutex<Event>,
    _phantom: PhantomData<T>,
}

impl<T: Copy + Send> Queue<T> {
    #[inline]
    /// Creates a new queue with the given length. Panics on failure; see
    /// [`Queue::try_new()`].
    pub fn new(length: u32) -> Self {
        Self::try_new(length).unwrap()
    }

    /// Creates a new queue with the given length.
    pub fn try_new(length: u32) -> Result<Self, Error> {
        let queue = unsafe { bindings::queue_create(length, size_of::<T>() as u32) };
        if queue == null_mut() {
            Err(from_errno())
        } else {
            Ok(Self {
                queue,
                event: Mutex::try_new(Event::new())?,
                _phantom: PhantomData,
            })
        }
    }

    /// Posts an item to the front of the queue. If the queue still does not
    /// have an empty slot after `timeout` has passed, the item is returned
    /// via the [`Err`] constructor.
    pub fn prepend(&self, item: T, timeout: Option<Duration>) -> Result<(), T> {
        if unsafe {
            bindings::queue_prepend(
                self.queue,
                &item as *const T as *const c_void,
                timeout.map_or(TIMEOUT_MAX, |v| v.as_secs() as u32),
            )
        } {
            self.event.lock().notify();
            Ok(())
        } else {
            Err(item)
        }
    }

    /// Posts an item to the back of the queue. If the queue still does not have
    /// an empty slot after `timeout` has passed, the item is returned via
    /// the [`Err`] constructor.
    pub fn append(&self, item: T, timeout: Option<Duration>) -> Result<(), T> {
        if unsafe {
            bindings::queue_append(
                self.queue,
                &item as *const T as *const c_void,
                timeout.map_or(TIMEOUT_MAX, |v| v.as_secs() as u32),
            )
        } {
            self.event.lock().notify();
            Ok(())
        } else {
            Err(item)
        }
    }

    /// Obtains a copy of the element at the front of the queue, without
    /// removing it. If the queue is still empty after `timeout` has passed,
    /// [`None`] is returned.
    pub fn peek(&self, timeout: Option<Duration>) -> Option<T> {
        let mut buf = Vec::<T>::new();
        buf.reserve_exact(1);
        unsafe {
            if bindings::queue_peek(
                self.queue,
                buf.as_mut_ptr() as *mut c_void,
                timeout.map_or(TIMEOUT_MAX, |v| v.as_secs() as u32),
            ) {
                buf.set_len(1);
                Some(buf[0])
            } else {
                None
            }
        }
    }

    /// Receives an element from the front of the queue, removing it. If the
    /// queue is still empty after `timeout` has passed, [`None`] is returned.
    pub fn recv(&self, timeout: Option<Duration>) -> Option<T> {
        let mut buf = Vec::<T>::new();
        buf.reserve_exact(1);
        unsafe {
            if bindings::queue_recv(
                self.queue,
                buf.as_mut_ptr() as *mut c_void,
                timeout.map_or(TIMEOUT_MAX, |v| v.as_secs() as u32),
            ) {
                buf.set_len(1);
                Some(buf[0])
            } else {
                None
            }
        }
    }

    #[inline]
    /// Gets the number of elements currently in the queue.
    pub fn waiting(&self) -> u32 {
        unsafe { bindings::queue_get_waiting(self.queue) }
    }

    #[inline]
    /// Gets the number of available slots in the queue (i.e., the number of
    /// elements which can be added to the queue).
    pub fn available(&self) -> u32 {
        unsafe { bindings::queue_get_available(self.queue) }
    }

    #[inline]
    /// Clears the queue (i.e., deletes all elements).
    pub fn clear(&self) {
        unsafe { bindings::queue_reset(self.queue) }
    }

    #[inline]
    /// Gets a [`Selectable`] event which resolves by receiving an element on
    /// the queue when it is able to.
    pub fn select(&self) -> impl Selectable<T> + '_ {
        struct QueueSelect<'a, T: Copy + Send>(&'a Queue<T>, EventHandle<&'a Mutex<Event>>);

        impl<'a, T: Copy + Send> Selectable<T> for QueueSelect<'a, T> {
            #[inline]
            fn poll(self) -> Result<T, Self> {
                self.0.recv(Some(Duration::default())).ok_or(self)
            }
            #[inline]
            fn sleep(&self) -> GenericSleep {
                GenericSleep::NotifyTake(None)
            }
        }

        QueueSelect(self, handle_event(&self.event))
    }
}

impl<T: Copy + Send> Drop for Queue<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { bindings::queue_delete(self.queue) }
    }
}

unsafe impl<T: Copy + Send> Send for Queue<T> {}
unsafe impl<T: Copy + Send> Sync for Queue<T> {}
