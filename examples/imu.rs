#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;

struct ImuBot {
    sensor: InertialSensor,
}

impl Robot for ImuBot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            sensor: peripherals.port01.into_imu(),
        }
    }

    fn opcontrol(&mut self, ctx: Context) {
        let mut l = Loop::new(Duration::from_secs(1));
        loop {
            println!("{}", self.sensor.get_heading().unwrap());
            select! {
                _ = l.select() => {},
                _ = ctx.done() => break,
            }
        }
    }
}

entry!(ImuBot);
