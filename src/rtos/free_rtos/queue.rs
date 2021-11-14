use core::convert::TryInto;
use core::ffi::c_void;
use core::marker::PhantomData;
use core::mem::{forget, size_of, MaybeUninit};
use core::time::Duration;

use crate::bindings;
use crate::prelude::{LengthQueue, PrependTimeoutQueue};
use crate::rtos::TIMEOUT_MAX;
use crate::{EnsureSend, EnsureSync};
use concurrency_traits::queue::{PrependQueue, Queue, TimeoutQueue, TryPrependQueue, TryQueue};

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
        unsafe {
            Self::from_queue_t(bindings::queue_create(
                size,
                size_of::<T>().try_into().unwrap(),
            ))
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
    pub const unsafe fn from_queue_t(queue: bindings::queue_t) -> Self {
        Self {
            queue,
            phantom_t: PhantomData,
        }
    }

    /// Gets the queue_t without dropping or deleting.
    /// Same as `mem::transmute` to `queue_t` due to `#[repr(transparent)]`.
    /// This function is safe because a queue_t itself is not unsafe and memory
    /// leaks are not unsafe.
    #[inline]
    pub const fn into_inner(self) -> bindings::queue_t {
        let out = self.queue;
        forget(self);
        out
    }
}
impl<T> Drop for FreeRtosQueue<T> {
    fn drop(&mut self) {
        while let Some(val) = self.try_pop() {
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
        if unsafe { bindings::queue_append(self.queue, &value as *const T as *const c_void, 0) } {
            forget(value);
            Ok(())
        } else {
            Err(value)
        }
    }

    #[inline]
    fn try_pop(&self) -> Option<Self::Item> {
        let mut uninit = MaybeUninit::uninit();
        unsafe {
            if bindings::queue_recv(
                self.queue,
                &mut uninit as *mut MaybeUninit<T> as *mut c_void,
                0,
            ) {
                Some(uninit.assume_init())
            } else {
                None
            }
        }
    }
}
impl<T> Queue for FreeRtosQueue<T> {
    #[inline]
    fn push(&self, value: Self::Item) {
        assert!(unsafe {
            bindings::queue_append(self.queue, &value as *const T as *const c_void, TIMEOUT_MAX)
        });
        forget(value);
    }

    #[inline]
    fn pop(&self) -> Self::Item {
        let mut uninit = MaybeUninit::uninit();
        unsafe {
            assert!(bindings::queue_recv(
                self.queue,
                &mut uninit as *mut MaybeUninit<T> as *mut c_void,
                TIMEOUT_MAX
            ));
            uninit.assume_init()
        }
    }
}
impl<T> TryPrependQueue for FreeRtosQueue<T> {
    fn try_push_front(&self, value: Self::Item) -> Result<(), Self::Item> {
        if unsafe { bindings::queue_prepend(self.queue, &value as *const T as *const c_void, 0) } {
            forget(value);
            Ok(())
        } else {
            Err(value)
        }
    }
}
impl<T> PrependQueue for FreeRtosQueue<T> {
    fn push_front(&self, value: Self::Item) {
        assert!(unsafe {
            bindings::queue_prepend(self.queue, &value as *const T as *const c_void, TIMEOUT_MAX)
        });
        forget(value);
    }
}
impl<T> TimeoutQueue for FreeRtosQueue<T> {
    fn push_timeout(&self, value: Self::Item, timeout: Duration) -> Result<(), Self::Item> {
        if unsafe {
            bindings::queue_append(
                self.queue,
                &value as *const T as *const c_void,
                timeout.as_millis() as u32,
            )
        } {
            forget(value);
            Ok(())
        } else {
            Err(value)
        }
    }

    fn pop_timeout(&self, timeout: Duration) -> Option<Self::Item> {
        let mut uninit = MaybeUninit::uninit();
        unsafe {
            if bindings::queue_recv(
                self.queue,
                &mut uninit as *mut MaybeUninit<T> as *mut c_void,
                timeout.as_millis() as u32,
            ) {
                Some(uninit.assume_init())
            } else {
                None
            }
        }
    }
}
impl<T> PrependTimeoutQueue for FreeRtosQueue<T> {
    fn push_front_timeout(&self, value: Self::Item, timeout: Duration) -> Result<(), Self::Item> {
        if unsafe {
            bindings::queue_prepend(
                self.queue,
                &value as *const T as *const c_void,
                timeout.as_millis() as u32,
            )
        } {
            forget(value);
            Ok(())
        } else {
            Err(value)
        }
    }
}
impl<T> LengthQueue for FreeRtosQueue<T> {
    fn len(&self) -> usize {
        unsafe { bindings::queue_get_waiting(self.queue) as usize }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<T> From<u32> for FreeRtosQueue<T> {
    fn from(from: u32) -> Self {
        Self::new(from)
    }
}
impl<T> Default for FreeRtosQueue<T> {
    fn default() -> Self {
        Self::new(128)
    }
}

/// Safe because queue_t is safe across thread boundaries
unsafe impl<T> Send for FreeRtosQueue<T> where T: Send {}
/// Safe because there is no way for this to generate a reference to a `T`
unsafe impl<T> Sync for FreeRtosQueue<T> where T: Send {}
