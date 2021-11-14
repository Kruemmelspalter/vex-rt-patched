#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;

struct DistanceBot {
    sensor: DistanceSensor,
}

#[async_trait(?Send)]
impl Robot for DistanceBot {
    async fn new(peripherals: Peripherals) -> Self {
        Self {
            sensor: peripherals.port01.into_distance(),
        }
    }

    async fn opcontrol(&'static self, robot_args: RobotArgs) {
        async_loop!(robot_args: (Duration::from_secs(1)){
            println!("{:#?}", self.sensor.get_distance().unwrap());
        });
    }
}

entry!(DistanceBot);
