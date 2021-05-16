use crate::rtos::TIMEOUT_MAX;
use crate::{bindings, error::*};
use concurrency_traits::mutex::{CustomMutex, RawMutex, RawTimeoutMutex, RawTryMutex};
use core::time::Duration;

/// A FreeRTOS Mutex
pub type FreeRtosMutex<T> = CustomMutex<T, FreeRtosRawMutex>;
/// A Recursive FreeRTOS Mutex
pub type FreeRtosRecursiveMutex<T> = CustomMutex<T, FreeRtosRawRecursiveMutex>;

/// A raw mutex from FreeRTOS
#[derive(Debug)]
pub struct FreeRtosRawMutex(bindings::mutex_t);
impl FreeRtosRawMutex {
    /// Creates a new recursive mutex
    pub fn new() -> Self {
        Self(
            unsafe { bindings::mutex_create() }
                .check()
                .expect("Could not create recursive mutex!"),
        )
    }
}
impl Drop for FreeRtosRawMutex {
    fn drop(&mut self) {
        unsafe { bindings::mutex_delete(self.0) }
    }
}
impl Default for FreeRtosRawMutex {
    fn default() -> Self {
        Self::new()
    }
}
unsafe impl RawTryMutex for FreeRtosRawMutex {
    fn try_lock(&self) -> bool {
        unsafe { bindings::mutex_take(self.0, 0) }
    }

    unsafe fn unlock(&self) {
        if !bindings::mutex_give(self.0) {
            panic!("Failed to unlock recursive mutex! {:?}", from_errno());
        }
    }
}
unsafe impl RawMutex for FreeRtosRawMutex {
    fn lock(&self) {
        if !unsafe { bindings::mutex_take(self.0, TIMEOUT_MAX) } {
            panic!(
                "Failed to lock recursive mutex with timeout max! {:?}",
                from_errno()
            );
        }
    }
}
unsafe impl RawTimeoutMutex for FreeRtosRawMutex {
    fn lock_timeout(&self, timeout: Duration) -> bool {
        unsafe { bindings::mutex_take(self.0, timeout.as_millis() as u32) }
    }
}
unsafe impl Send for FreeRtosRawMutex {}
unsafe impl Sync for FreeRtosRawMutex {}

/// A recursive raw mutex from FreeRTOS
#[derive(Debug)]
pub struct FreeRtosRawRecursiveMutex(bindings::mutex_t);
impl FreeRtosRawRecursiveMutex {
    /// Creates a new recursive mutex
    ///
    /// # Safety
    /// This type of mutex can allow multiple mutable accesses to the same data
    /// from the same task. Ensure that this is never locked twice from the
    /// same task to make this safe.
    pub unsafe fn new() -> Self {
        Self(
            bindings::mutex_recursive_create()
                .check()
                .expect("Could not create recursive mutex!"),
        )
    }
}
impl Drop for FreeRtosRawRecursiveMutex {
    fn drop(&mut self) {
        unsafe { bindings::mutex_delete(self.0) }
    }
}
unsafe impl RawTryMutex for FreeRtosRawRecursiveMutex {
    fn try_lock(&self) -> bool {
        unsafe { bindings::mutex_recursive_take(self.0, 0) }
    }

    unsafe fn unlock(&self) {
        if !bindings::mutex_recursive_give(self.0) {
            panic!("Failed to unlock recursive mutex! {:?}", from_errno());
        }
    }
}
unsafe impl RawMutex for FreeRtosRawRecursiveMutex {
    fn lock(&self) {
        if !unsafe { bindings::mutex_recursive_take(self.0, TIMEOUT_MAX) } {
            panic!(
                "Failed to lock recursive mutex with timeout max! {:?}",
                from_errno()
            );
        }
    }
}
unsafe impl RawTimeoutMutex for FreeRtosRawRecursiveMutex {
    fn lock_timeout(&self, timeout: Duration) -> bool {
        unsafe { bindings::mutex_recursive_take(self.0, timeout.as_millis() as u32) }
    }
}
unsafe impl Send for FreeRtosRawRecursiveMutex {}
unsafe impl Sync for FreeRtosRawRecursiveMutex {}
