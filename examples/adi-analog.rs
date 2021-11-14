#![no_std]
#![no_main]

use core::{convert::TryInto, time::Duration};
use vex_rt::prelude::*;

struct AnalogBot {
    sensor: AdiAnalog,
}
#[async_trait(?Send)]
impl Robot for AnalogBot {
    async fn new(peripherals: Peripherals) -> Self {
        Self {
            sensor: peripherals.port_g.try_into().unwrap(),
        }
    }

    async fn opcontrol(&'static self, robot_args: RobotArgs) {
        async_loop!(robot_args: (Duration::from_secs(1)){
            println!("{}", self.sensor.read().unwrap());
        });
    }
}

entry!(AnalogBot);
