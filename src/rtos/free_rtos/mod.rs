//! FreeRTOS concurrency primitives

mod mutex;
mod queue;

pub use mutex::*;
pub use queue::*;

use crate::error::Error as VexRtError;
use crate::rtos::{time_since_start, Instant, Task};
use concurrency_traits::mutex::{ParkMutex, SpinLock};
use concurrency_traits::queue::ParkQueue;
use concurrency_traits::*;
use core::time::Duration;
use single_executor::AsyncExecutor;

/// A FreeRTOS based [`ParkMutex`].
pub type ParkMutexFreeRtos<T> = ParkMutex<T, FreeRtosConcurrency>;
/// A FreeRTOS based [`SpinLock`].
pub type SpinLockFreeRtos<T> = SpinLock<T, FreeRtosConcurrency>;
/// A FreeRTOS based [`ParkQueue`].
pub type ParkQueueFreeRtos<T> = ParkQueue<T, FreeRtosConcurrency>;
/// A FreeRTOS based [`AsyncExecutor`].
pub type AsyncExecutorFreeRtos<Q> = AsyncExecutor<Q, FreeRtosConcurrency>;

/// FreeRTOS concurrency bindings.
#[derive(Copy, Clone, Debug)]
pub struct FreeRtosConcurrency;
impl TimeFunctions for FreeRtosConcurrency {
    type InstantType = Instant;

    #[inline]
    fn current_time() -> Self::InstantType {
        time_since_start()
    }
}
impl ThreadFunctions for FreeRtosConcurrency {
    #[inline]
    fn sleep(duration: Duration) {
        Task::delay(duration);
    }

    #[inline]
    fn yield_now() {
        Task::delay(Duration::from_millis(1));
    }
}
impl TryThreadSpawner<()> for FreeRtosConcurrency {
    type ThreadHandle = Task;
    type SpawnError = VexRtError;

    #[inline]
    fn try_spawn(
        func: impl FnOnce() + Send + 'static,
    ) -> Result<Self::ThreadHandle, Self::SpawnError> {
        Task::spawn(func)
    }
}
impl ThreadParker for FreeRtosConcurrency {
    type ThreadId = Task;

    #[inline]
    fn park() {
        Task::notify_take(true, None);
    }

    #[inline]
    fn unpark(thread: Self::ThreadId) {
        thread.notify();
    }

    #[inline]
    fn current_thread() -> Self::ThreadId {
        Task::current()
    }
}
impl ThreadTimeoutParker for FreeRtosConcurrency {
    #[inline]
    fn park_timeout(timeout: Duration) {
        Task::notify_take(true, Some(timeout));
    }
}
impl ConcurrentSystem<()> for FreeRtosConcurrency {}
impl ThreadHandle for Task {
    type ThreadId = Self;

    #[inline]
    fn thread_id(&self) -> &Self::ThreadId {
        self
    }
}
