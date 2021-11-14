//! Multitasking primitives.

use alloc::collections::VecDeque;

use concurrency_traits::mutex::{FullAsyncMutex, Mutex, SpinLock};
use concurrency_traits::queue::{SemaphoreQueue, TryQueue};
use concurrency_traits::semaphore::FullAsyncSemaphore;
use crossbeam::queue::SegQueue;
use simple_futures::complete_future::CompleteFutureHandle;

pub use instant::*;
pub use task::*;

use crate::error::Error as VexRtError;
use crate::rtos::free_rtos::FreeRtosConcurrency;
use core::future::Future;
use single_executor::try_spawn_blocking;

pub mod free_rtos;

mod instant;
mod task;

/// A generic async queue.
pub type VexAsyncQueue<T> =
    SemaphoreQueue<T, FullAsyncSemaphore<usize, FreeRtosConcurrency>, FreeRtosConcurrency>;

/// A generic async mutex.
pub type VexAsyncMutex<T> = FullAsyncMutex<T, SegQueue<CompleteFutureHandle>>;

/// A spin lock around a queue. Used for [`VexAsyncMutex`].
#[derive(Debug)]
pub struct LockQueue<T>(SpinLock<VecDeque<T>, FreeRtosConcurrency>);
impl<T> Default for LockQueue<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<T> TryQueue for LockQueue<T> {
    type Item = T;

    fn try_push(&self, value: Self::Item) -> Result<(), Self::Item> {
        self.0.lock().push_back(value);
        Ok(())
    }

    fn try_pop(&self) -> Option<Self::Item> {
        self.0.lock().pop_front()
    }
}

const TIMEOUT_MAX: u32 = 0xffffffff;

/// Spawns a new task that can block. Returns a future that will be the result
/// of the task and a handle to the running thread.
pub fn spawn_blocking<F, T>(function: F) -> Result<(impl Future<Output = T>, Task), VexRtError>
where
    F: FnOnce() -> T + Send + 'static,
    T: 'static + Send,
{
    try_spawn_blocking::<_, _, FreeRtosConcurrency>(function)
}
