#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;
use vex_rt::rtos::free_rtos::FreeRtosConcurrency;

struct TimeoutRobot;

#[async_trait(?Send)]
impl Robot for TimeoutRobot {
    async fn new(_peripherals: Peripherals) -> Self {
        Self
    }

    async fn autonomous(&'static self, robot_args: RobotArgs) {
        let sleep_runner = robot_args.sleep_runner.clone();
        let runner_clone = sleep_runner.clone();
        sleep_runner
            .timeout_for(
                async move {
                    let mut x = 0;
                    let mut last = FreeRtosConcurrency::current_time();
                    loop {
                        println!("{}", x);
                        x += 1;
                        runner_clone
                            .sleep_until(last + Duration::from_secs(1))
                            .await;
                        last = FreeRtosConcurrency::current_time();
                    }
                },
                Duration::from_secs(20),
            )
            .await
            .unwrap_err();
    }
}

entry!(TimeoutRobot);
