#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;

struct DistanceBot {
    sensor: DistanceSensor,
}

impl Robot for DistanceBot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            sensor: peripherals.port01.into_distance(),
        }
    }
    fn opcontrol(&'static self, ctx: Context) {
        let mut l = Loop::new(Duration::from_secs(1));
        loop {
            println!("{:#?}", self.sensor.get_distance().unwrap());
            select! {
                _ = l.select() => {},
                _ = ctx.done() => break,
            }
        }
    }
}

entry!(DistanceBot);
