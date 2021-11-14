// TODO

#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;

struct SelectRobot;

#[async_trait(?Send)]
impl Robot for SelectRobot {
    async fn new(_peripherals: Peripherals) -> Self {
        Self
    }

    async fn autonomous(&'static self, robot_args: RobotArgs) {
        println!("autonomous");
        let mut x = 0;
        async_loop!(robot_args: (Duration::from_secs(1)){
            println!("{}", x);
            x += 1;
        });
    }
}

entry!(SelectRobot);
