use concurrency_traits::queue::{
    PeekQueue, PrependQueue, Queue, TryPeekQueue, TryPrependQueue, TryQueue,
};
use core::{
    convert::TryInto,
    ffi::c_void,
    marker::PhantomData,
    mem::{forget, size_of, MaybeUninit},
    time::Duration,
};

use crate::{bindings, rtos::TIMEOUT_MAX, EnsureSend, EnsureSync};

/// A FreeRTOS queue that blocks the thread.
#[repr(transparent)]
#[derive(Debug)]
pub struct FreeRtosQueue<T> {
    queue: bindings::queue_t,
    phantom_t: PhantomData<T>,
}

impl<T> FreeRtosQueue<T> {
    /// Creates a new queue with a max size of `size`.
    #[inline]
    pub fn new(size: u32) -> Self {
        Self {
            queue: unsafe { bindings::queue_create(size, size_of::<T>().try_into().unwrap()) },
            phantom_t: Default::default(),
        }
    }

    /// Creates a queue from a raw queue_t.
    /// This is unsafe because this queue_t could be from anywhere, even a
    /// nullptr.
    ///
    /// ## Safety
    /// queue must be created from `queue_create`, not have had `queue_delete`
    /// called on it, and have only objects of type `T`
    #[inline]
    pub unsafe fn from_queue_t(queue: bindings::queue_t) -> Self {
        Self {
            queue,
            phantom_t: Default::default(),
        }
    }

    /// Gets the queue_t without dropping or deleting.
    /// Same as `mem::transmute` to `queue_t` due to `#[repr(transparent)]`.
    /// This function is safe because a queue_t itself is not unsafe and memory
    /// leaks are not unsafe.
    #[inline]
    pub fn into_inner(self) -> bindings::queue_t {
        let out = self.queue;
        forget(self);
        out
    }

    /// Adds to the end of the queue blocking until the queue has a slot
    /// available.
    #[inline]
    fn append(&self, val: T) {
        self.append_timeout(val, None)
            .unwrap_or_else(|_| panic!("append failed without timeout!"))
    }
    /// Adds to the end of the queue returning `Err` if no slots available.
    #[inline]
    fn try_append(&self, val: T) -> Result<(), T> {
        self.append_timeout(val, Option::Some(Duration::new(0, 0)))
    }
    /// Adds to the end of the queue blocking until the queue has a slot
    /// available. Will return Err if timeout elapses before slot is
    /// available.
    fn append_timeout(&self, val: T, timeout: Option<Duration>) -> Result<(), T> {
        if unsafe {
            bindings::queue_append(
                self.queue,
                &val as *const T as *const c_void,
                timeout.map_or(TIMEOUT_MAX, |val| val.as_millis() as u32),
            )
        } {
            forget(val);
            Ok(())
        } else {
            Err(val)
        }
    }

    /// Adds to the front of the queue blocking until the queue has a slot
    /// available.
    #[inline]
    fn prepend(&self, val: T) {
        self.prepend_timeout(val, None)
            .unwrap_or_else(|_| panic!("prepend failed without timeout!"))
    }

    /// Adds to the front of the queue returning `Err` if no slots available.
    #[inline]
    fn try_prepend(&self, val: T) -> Result<(), T> {
        self.prepend_timeout(val, Some(Duration::new(0, 0)))
    }

    /// Adds to the front of the queue blocking until the queue has a slot
    /// available. Will return Err if timeout elapses before slot is
    /// available.
    fn prepend_timeout(&self, val: T, timeout: Option<Duration>) -> Result<(), T> {
        if unsafe {
            bindings::queue_prepend(
                self.queue,
                &val as *const T as *const c_void,
                timeout.map_or(TIMEOUT_MAX, |val| val.as_millis() as u32),
            )
        } {
            forget(val);
            Ok(())
        } else {
            Err(val)
        }
    }

    /// Clones the front of the queue blocking until an item is added.
    #[inline]
    fn peek(&self) -> T
    where
        T: Clone,
    {
        self.peek_timeout(None)
            .expect("peek failed without timeout!")
    }
    /// Clones the front of the queue returning `None` if empty.
    #[inline]
    fn try_peek(&self) -> Option<T>
    where
        T: Clone,
    {
        self.peek_timeout(Some(Duration::new(0, 0)))
    }
    /// Clones the front of the queue blocking an item is added.
    /// Returns None if timeout elapses before an item was added.
    fn peek_timeout(&self, timeout: Option<Duration>) -> Option<T>
    where
        T: Clone,
    {
        let mut uninit = MaybeUninit::uninit();
        unsafe {
            if bindings::queue_peek(
                self.queue,
                &mut uninit as *mut MaybeUninit<T> as *mut c_void,
                timeout.map_or(TIMEOUT_MAX, |val| val.as_millis() as u32),
            ) {
                let copied: T = uninit.assume_init();
                let out = copied.clone();
                forget(copied);
                Some(out)
            } else {
                None
            }
        }
    }

    /// Pops the front of the queue blocking until an item is added.
    #[inline]
    fn recv(&self) -> T {
        self.recv_timeout(None)
            .expect("recv failed without timeout!")
    }

    /// Pops the front of the queue returning `None` if empty.
    #[inline]
    fn try_recv(&self) -> Option<T> {
        self.recv_timeout(Some(Duration::new(0, 0)))
    }

    /// Pops the front of the queue blocking until an item is added.
    /// Returns None if timeout elapses before an item was added.
    //TODO: Make private
    pub fn recv_timeout(&self, timeout: Option<Duration>) -> Option<T> {
        let mut uninit = MaybeUninit::uninit();
        unsafe {
            if bindings::queue_recv(
                self.queue,
                &mut uninit as *mut MaybeUninit<T> as *mut c_void,
                timeout.map_or(TIMEOUT_MAX, |val| val.as_millis() as u32),
            ) {
                Some(uninit.assume_init())
            } else {
                None
            }
        }
    }

    /// Gets the length of the queue
    #[inline]
    pub fn len(&self) -> u32 {
        unsafe { bindings::queue_get_waiting(self.queue) }
    }

    /// Tells whether the queue is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Drop for FreeRtosQueue<T> {
    fn drop(&mut self) {
        while let Some(val) = self.recv_timeout(Some(Duration::from_millis(0))) {
            drop(val);
        }
        unsafe { bindings::queue_delete(self.queue) }
    }
}

impl<T> EnsureSend for FreeRtosQueue<T> where T: Send {}

impl<T> EnsureSync for FreeRtosQueue<T> where T: Send {}

impl<T> TryQueue for FreeRtosQueue<T> {
    type Item = T;

    #[inline]
    fn try_push(&self, value: Self::Item) -> Result<(), Self::Item> {
        self.try_append(value)
    }

    #[inline]
    fn try_pop(&self) -> Option<Self::Item> {
        self.try_recv()
    }
}

impl<T> Queue for FreeRtosQueue<T> {
    #[inline]
    fn push(&self, value: Self::Item) {
        self.append(value);
    }

    #[inline]
    fn pop(&self) -> Self::Item {
        self.recv()
    }
}

impl<T> TryPrependQueue for FreeRtosQueue<T> {
    fn try_push_front(&self, value: Self::Item) -> Result<(), Self::Item> {
        self.try_prepend(value)
    }
}

impl<T> PrependQueue for FreeRtosQueue<T> {
    fn push_front(&self, value: Self::Item) {
        self.prepend(value)
    }
}

impl<T: Clone> TryPeekQueue for FreeRtosQueue<T> {
    type Peeked = T;

    fn try_peek(&self) -> Option<Self::Peeked> {
        self.try_peek()
    }
}

impl<T: Clone> PeekQueue for FreeRtosQueue<T> {
    fn peek(&self) -> Self::Peeked {
        self.peek()
    }
}

impl<T> From<u32> for FreeRtosQueue<T> {
    fn from(from: u32) -> Self {
        Self::new(from)
    }
}

/// Safe because queue_t is safe across thread boundaries
unsafe impl<T> Send for FreeRtosQueue<T> where T: Send {}

/// Safe because there is no way for this to generate a reference to a `T`
unsafe impl<T> Sync for FreeRtosQueue<T> where T: Send {}
