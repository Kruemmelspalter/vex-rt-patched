#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;

struct DelayBot;

#[async_trait(?Send)]
impl Robot for DelayBot {
    const MAX_LOG_LEVEL: LevelFilter = LevelFilter::Trace;

    async fn new(_peripherals: Peripherals) -> Self {
        Self
    }

    async fn opcontrol(&'static self, robot_args: RobotArgs) {
        let mut x = 0;
        loop {
            let time = time_since_start() + Duration::from_secs(1);
            info!("x = {}", x);
            debug!("Next time: {}", time);
            robot_args.sleep_runner.sleep_until(time).await;
            x += 1;
        }
    }
}

entry!(DelayBot);
