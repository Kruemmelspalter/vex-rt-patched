//! For use with the [`entry`] macro.

use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use core::future::Future;

use async_trait::async_trait;
use log::LevelFilter;

use concurrency_traits::queue::TimeoutQueue;
use futures::future::{abortable, AbortHandle, Abortable};
use log::debug;
use single_executor::LocalExecutorHandle;
use single_executor::SendExecutorHandle;
use single_executor::{AsyncTask, SleepFutureRunner, SleepMessage};

use crate::rtos::free_rtos::{FreeRtosConcurrency, FreeRtosQueue};
use crate::rtos::VexAsyncMutex;
use crate::{io::println, peripherals::Peripherals};
use alloc::collections::VecDeque;
use alloc::string::String;
use concurrency_traits::mutex::AsyncMutex;
use core::fmt::Display;
use core::ops::DerefMut;

/// A trait representing a competition-ready VEX Robot.
#[async_trait(?Send)]
#[allow(unused_variables)]
pub trait Robot: 'static {
    /// The maximum logged level.
    const MAX_LOG_LEVEL: LevelFilter = LevelFilter::Info;

    /// Runs at startup, constructing your robot. This should be non-blocking,
    /// since the FreeRTOS scheduler doesn't start until it returns.
    async fn new(peripherals: Peripherals) -> Self;

    /// Runs immediately after [`Robot::new`]. The FreeRTOS scheduler is running
    /// by this point.
    ///
    /// The purpose of this method is to provide a hook to run things on startup
    /// which require a reference to all or part of the robot structure; since
    /// it takes `&'static self` as its parameter, the lifetime of the robot
    /// object is guaranteed to be static (i.e., forever), and so the
    /// implementation may pass references around (e.g., to new tasks) at will
    /// without issue.
    async fn initialize(&'static mut self, robot_args: InitializeRobotArgs) -> &'static Self {
        self
    }

    /// Runs during the autonomous period.
    async fn autonomous(&'static self, robot_args: RobotArgs) {
        println!("autonomous");
    }

    /// Runs during the opcontrol period.
    async fn opcontrol(&'static self, robot_args: RobotArgs) {
        println!("opcontrol");
    }

    /// Runs when the robot is disabled.
    async fn disabled(&'static self, robot_args: RobotArgs) {
        println!("disabled");
    }
}

/// Simplified type for the sleep runner used by a robot.
pub type SleepRunner =
    SleepFutureRunner<FreeRtosQueue<SleepMessage<FreeRtosConcurrency>>, FreeRtosConcurrency>;

/// The args sent to robot functions.
#[derive(Debug)]
pub struct RobotArgs {
    /// A handle that launches dependant tasks from a local (![`Send`]) context.
    pub local_handle: LocalLimitedHandle<FreeRtosQueue<AsyncTask>>,
    /// A handle that can be sent across threads to launch dependant tasks.
    pub send_handle: LimitedHandle<FreeRtosQueue<AsyncTask>>,
    /// The runner for sleeping.
    pub sleep_runner: Arc<SleepRunner>,
}

/// The args sent to [`Robot::initialize`].
#[derive(Debug)]
#[non_exhaustive]
pub struct InitializeRobotArgs {
    /// A handle that launches independent tasks from a local (![`Send`])
    /// context.
    pub local_handle: LocalExecutorHandle<FreeRtosQueue<AsyncTask>>,
    /// A handle that can be sent across threads to launch independent tasks.
    pub send_handle: SendExecutorHandle<FreeRtosQueue<AsyncTask>>,
    /// The runner for sleeping.
    pub sleep_runner: Arc<SleepRunner>,
}

pub(crate) async fn abort_clean<F>(abort: Abortable<F>, task_name: impl Display)
where
    F: Future<Output = ()>,
{
    match abort.await {
        Ok(_) => debug!("{} exited by itself", task_name),
        Err(_) => debug!("{} was aborted by competition switch", &task_name),
    }
}

/// A handle that launches tasks that end with the current state. This version
/// is not [`Send`] and can take futures that are not [`Send`].
#[derive(Debug)]
#[non_exhaustive]
pub struct LocalLimitedHandle<Q> {
    pub(crate) local_handle: LocalExecutorHandle<Q>,
    pub(crate) abort_queue: Weak<VexAsyncMutex<Option<VecDeque<AbortHandle>>>>,
}
impl<Q> LocalLimitedHandle<Q>
where
    Q: 'static + TimeoutQueue<Item = AsyncTask> + Send + Sync,
{
    /// Submits a future that will end with the current state.
    pub async fn submit(
        &self,
        future: impl Future<Output = ()> + 'static,
        task_name: impl Into<String>,
    ) {
        let queue = match self.abort_queue.upgrade() {
            None => {
                debug!("Task {} was submitted to dead queue", task_name.into());
                return;
            }
            Some(queue) => queue,
        };
        let mut guard = queue.lock_async().await;
        let queue = match guard.deref_mut() {
            Some(queue) => queue,
            None => {
                debug!("Task {} was submitted to dead queue", task_name.into());
                return;
            }
        };
        let (abort, handle) = abortable(future);
        let task_name = task_name.into();
        let name_clone = task_name.clone();
        self.local_handle
            .submit(abort_clean(abort, name_clone), task_name)
            .unwrap_or_else(|_| panic!("Could not submit task"));
        queue.push_back(handle);
    }
}

/// A handle that launches tasks that end with the current state. This version
/// is [`Send`] and can only take futures that are [`Send`].
#[derive(Debug)]
pub struct LimitedHandle<Q> {
    pub(crate) handle: SendExecutorHandle<Q>,
    pub(crate) abort_queue: Weak<VexAsyncMutex<Option<VecDeque<AbortHandle>>>>,
}
impl<Q> LimitedHandle<Q>
where
    Q: 'static + TimeoutQueue<Item = AsyncTask> + Send + Sync,
{
    /// Submits a future that will end with the current state. Must be [`Send`].
    pub async fn submit<F>(
        &self,
        future: impl Future<Output = ()> + 'static + Send,
        task_name: impl Into<String>,
    ) {
        let queue = match self.abort_queue.upgrade() {
            None => {
                debug!("Task {} was submitted to dead queue", task_name.into());
                return;
            }
            Some(queue) => queue,
        };
        let mut guard = queue.lock_async().await;
        let queue = match guard.deref_mut() {
            Some(queue) => queue,
            None => {
                debug!("Task {} was submitted to dead queue", task_name.into());
                return;
            }
        };
        let (abort, handle) = abortable(future);
        let task_name = task_name.into();
        let name_clone = task_name.clone();
        self.handle
            .submit(abort_clean(abort, name_clone), task_name)
            .unwrap_or_else(|_| panic!("Could not submit task"));
        queue.push_back(handle);
    }
}
