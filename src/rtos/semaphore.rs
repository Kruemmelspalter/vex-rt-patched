use core::{convert::TryInto, time::Duration};

use crate::{
    bindings,
    error::{from_errno, Error, SentinelError},
};

#[repr(transparent)]
/// Represents a FreeRTOS counting semaphore.
pub struct Semaphore(bindings::sem_t);

impl Semaphore {
    #[inline]
    /// Creates a new semaphore. Panics on failure; see [`Semaphore::try_new`].
    pub fn new(max_count: u32, init_count: u32) -> Self {
        Self::try_new(max_count, init_count)
            .unwrap_or_else(|err| panic!("failed to create semaphore: {}", err))
    }

    /// Creates a new semaphore.
    pub fn try_new(max_count: u32, init_count: u32) -> Result<Self, Error> {
        Ok(Self(
            unsafe { bindings::sem_create(max_count, init_count) }.check()?,
        ))
    }

    #[inline]
    /// Blocks up to `timeout` until an instance of the semaphore can be taken
    /// (i.e., its count decremented). If the semaphore cannot be taken (due
    /// to timeout or other reason), an error is returned.
    pub fn wait(&self, timeout: Duration) -> Result<(), Error> {
        if unsafe { bindings::sem_wait(self.0, timeout.as_millis().try_into()?) } {
            Ok(())
        } else {
            Err(from_errno())
        }
    }

    #[inline]
    /// Increments the semaphore's count. If the semaphore cannot be given, an
    /// error is returned.
    pub fn post(&self) -> Result<(), Error> {
        if unsafe { bindings::sem_post(self.0) } {
            Ok(())
        } else {
            Err(from_errno())
        }
    }

    #[inline]
    /// Gets the semaphore's current count.
    pub fn count(&self) -> u32 {
        unsafe { bindings::sem_get_count(self.0) }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe { bindings::sem_delete(self.0) }
    }
}

unsafe impl Send for Semaphore {}

unsafe impl Sync for Semaphore {}
