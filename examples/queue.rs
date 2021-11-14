// TODO

#![no_std]
#![no_main]

extern crate alloc;

use core::time::Duration;

use alloc::sync::Arc;
use vex_rt::prelude::*;

struct QueueBot {
    queue: Arc<VexAsyncQueue<i32>>,
}

#[async_trait(?Send)]
impl Robot for QueueBot {
    async fn new(_peripherals: Peripherals) -> Self {
        Self {
            queue: Default::default(),
        }
    }

    async fn initialize(&'static mut self, robot_args: InitializeRobotArgs) -> &'static Self {
        let queue = self.queue.clone();
        let sleep_runner = robot_args.sleep_runner;
        robot_args
            .local_handle
            .submit(
                async move {
                    sleep_runner.sleep_for(Duration::from_secs(1)).await;
                    queue.push_async(3).await;
                },
                "queue-bot::sleep-task",
            )
            .unwrap_or_else(|_| panic!("Could not submit task!"));
        self
    }

    async fn opcontrol(&'static self, _robot_args: RobotArgs) {
        println!("opcontrol");
        // We don't need to use `async_loop!` here because we await on the pop
        // operation.
        loop {
            println!("{}", self.queue.pop_async().await);
        }
    }
}

entry!(QueueBot);
