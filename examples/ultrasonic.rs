#![no_std]
#![no_main]

use core::{convert::TryInto, time::Duration};
use vex_rt::prelude::*;

struct UltrasonicBot {
    sensor: AdiUltrasonic,
}

#[async_trait(?Send)]
impl Robot for UltrasonicBot {
    async fn new(peripherals: Peripherals) -> Self {
        Self {
            sensor: (peripherals.port_a, peripherals.port_b).try_into().unwrap(),
        }
    }

    async fn opcontrol(&'static self, robot_args: RobotArgs) {
        async_loop!(robot_args: (Duration::from_secs(1)){
            match self.sensor.get() {
                Ok(r) => {
                    println!("{}", r);
                }
                Err(AdiUltrasonicError::NoReading) => {
                    println!("<no reading>");
                }
                e => {
                    e.unwrap();
                }
            }
        });
    }
}

entry!(UltrasonicBot);
