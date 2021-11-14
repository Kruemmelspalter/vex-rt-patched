//! The function used as an entrypoint for the robot.

use crate::competition::CompetitionStatus;
use crate::logger::init_logger;
use crate::peripherals::Peripherals;
use crate::prelude::RobotArgs;
use crate::robot::{abort_clean, InitializeRobotArgs, LimitedHandle, LocalLimitedHandle, Robot};
use crate::rtos::free_rtos::{AsyncExecutorFreeRtos, FreeRtosQueue};
use crate::rtos::VexAsyncMutex;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use concurrency_traits::mutex::AsyncMutex;
use core::ops::DerefMut;
use core::sync::atomic::AtomicBool;
use core::time::Duration;
use futures::future::{AbortHandle, Abortable};
use log::*;
use single_executor::SleepFutureRunner;

/// Used to initialize the runtime. Generally implement with [`entry`].
pub fn initialize_entry<R>(sleep_queue_size: u32, executor_queue_size: u32)
where
    R: Robot,
{
    init_logger(R::MAX_LOG_LEVEL).expect("Could not init logger");
    let sleep_runner = Arc::new(
        SleepFutureRunner::try_new(FreeRtosQueue::new(sleep_queue_size))
            .expect("Could not create sleep_runner"),
    );
    let async_executor = AsyncExecutorFreeRtos::new(FreeRtosQueue::new(executor_queue_size));
    let local_handle = async_executor.local_handle();
    let send_handle = async_executor.handle();
    async_executor.submit(
        async move {
            let robot_args = InitializeRobotArgs {
                local_handle: local_handle.clone(),
                send_handle: send_handle.clone(),
                sleep_runner: sleep_runner.clone(),
            };
            trace!("Initializing Robot");
            let robot = Box::leak(Box::new(R::new(unsafe { Peripherals::new() }).await))
                .initialize(robot_args)
                .await;
            let mut status = CompetitionStatus::INVALID;
            let mut abort_queue: Option<Arc<VexAsyncMutex<Option<VecDeque<AbortHandle>>>>> = None;
            loop {
                let old_status = status;
                status = CompetitionStatus::get();
                if status != old_status {
                    if (status.contains(CompetitionStatus::DISABLED))
                        && (old_status.contains(CompetitionStatus::DISABLED))
                    {
                        continue; // Ignore bit changes when disabled ->
                                  // disabled
                    }

                    if let Some(queue_option) = abort_queue {
                        trace!("Ending previous tasks");
                        let mut guard = queue_option.lock_async().await;
                        let mut queue =
                            guard.deref_mut().take().expect("Queue was taken by other!");
                        while let Some(handle) = queue.pop_front() {
                            handle.abort();
                        }
                    }
                    let mut queue = VecDeque::new();
                    let (abort, reg) = AbortHandle::new_pair();
                    queue.push_back(abort);
                    let abort_queue_inner = Arc::new(VexAsyncMutex::new(Some(queue)));
                    let local_limited = LocalLimitedHandle {
                        local_handle: local_handle.clone(),
                        abort_queue: Arc::downgrade(&abort_queue_inner),
                    };
                    let limited = LimitedHandle {
                        handle: send_handle.clone(),
                        abort_queue: Arc::downgrade(&abort_queue_inner),
                    };
                    abort_queue = Some(abort_queue_inner);
                    let sleep_runner_clone = sleep_runner.clone();
                    let robot_args = RobotArgs {
                        local_handle: local_limited,
                        send_handle: limited,
                        sleep_runner: sleep_runner_clone,
                    };
                    if status.contains(CompetitionStatus::DISABLED) {
                        trace!("Switching to Robot::disabled");
                        const DISABLED_NAME: &str = "vex-rt::disabled";
                        local_handle
                            .submit(
                                abort_clean(
                                    Abortable::new(robot.disabled(robot_args), reg),
                                    DISABLED_NAME,
                                ),
                                DISABLED_NAME,
                            )
                            .unwrap_or_else(|_| panic!("Could not spawn disabled!"));
                    } else if status.contains(CompetitionStatus::AUTONOMOUS) {
                        trace!("Switching to Robot::autonomous");
                        const AUTONOMOUS_NAME: &str = "vex-rt::autonomous";
                        local_handle
                            .submit(
                                abort_clean(
                                    Abortable::new(robot.autonomous(robot_args), reg),
                                    AUTONOMOUS_NAME,
                                ),
                                AUTONOMOUS_NAME,
                            )
                            .unwrap_or_else(|_| panic!("Could not spawn autonomous!"));
                    } else {
                        trace!("Switching to Robot::opcontrol");
                        const OPCONTROL_NAME: &str = "vex-rt::opcontrol";
                        local_handle
                            .submit(
                                abort_clean(
                                    Abortable::new(robot.opcontrol(robot_args), reg),
                                    OPCONTROL_NAME,
                                ),
                                OPCONTROL_NAME,
                            )
                            .unwrap_or_else(|_| panic!("Could not spawn opcontrol!"));
                    }
                }
                sleep_runner.sleep_for(Duration::from_millis(100)).await;
            }
        },
        "vex-rt::launch-control",
    );
    let stop = Arc::new(AtomicBool::new(false));
    async_executor.run(stop);
}
