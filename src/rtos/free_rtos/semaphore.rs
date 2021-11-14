use crate::bindings;
use crate::rtos::TIMEOUT_MAX;
use concurrency_traits::semaphore::{ReadoutSemaphore, Semaphore, TimeoutSemaphore, TrySemaphore};
use core::time::Duration;

/// A FreeRTOS semaphore. Trait signal will panic if `max_count` is reached.
#[derive(Debug)]
pub struct FreeRtosSemaphore {
    sem_t: bindings::sem_t,
}
impl FreeRtosSemaphore {
    /// Creates a new semaphore from a max count and an initial count.
    pub fn new(max_count: u32, init_count: u32) -> Self {
        Self {
            sem_t: unsafe { bindings::sem_create(max_count, init_count) },
        }
    }

    /// Increments the semaphore returning [`true`] if successful. Will not
    /// panic on reaching max count.
    pub fn post(&self) -> bool {
        unsafe { bindings::sem_post(self.sem_t) }
    }
}
impl Default for FreeRtosSemaphore {
    fn default() -> Self {
        Self::new(u32::MAX, 0)
    }
}
impl Drop for FreeRtosSemaphore {
    fn drop(&mut self) {
        unsafe { bindings::sem_delete(self.sem_t) }
    }
}
unsafe impl TrySemaphore for FreeRtosSemaphore {
    fn try_wait(&self) -> bool {
        unsafe { bindings::sem_wait(self.sem_t, 0) }
    }

    fn signal(&self) {
        assert!(unsafe { bindings::sem_post(self.sem_t) });
    }
}
unsafe impl Semaphore for FreeRtosSemaphore {
    fn wait(&self) {
        assert!(unsafe { bindings::sem_wait(self.sem_t, TIMEOUT_MAX) });
    }
}
unsafe impl TimeoutSemaphore for FreeRtosSemaphore {
    fn wait_timeout(&self, timeout: Duration) -> bool {
        unsafe { bindings::sem_wait(self.sem_t, timeout.as_millis() as u32) }
    }
}
impl ReadoutSemaphore for FreeRtosSemaphore {
    type Count = u32;

    fn count(&self) -> Self::Count {
        unsafe { bindings::sem_get_count(self.sem_t) }
    }
}
