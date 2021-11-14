#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::free_rtos::FreeRtosConcurrency;
use vex_rt::prelude::*;

struct TaskBot;

#[async_trait(?Send)]
impl Robot for TaskBot {
    async fn new(_peripherals: Peripherals) -> Self {
        TaskBot
    }

    async fn opcontrol(&'static self, _robot_args: RobotArgs) {
        println!(
            "n = {}",
            spawn_blocking(|| {
                FreeRtosConcurrency::sleep(Duration::from_secs(1));
                10
            })
            .expect("Could not spawn task")
            .0
            .await
        );
    }
}

entry!(TaskBot);
