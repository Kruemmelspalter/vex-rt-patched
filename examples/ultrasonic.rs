#![no_std]
#![no_main]

use core::{convert::TryInto, time::Duration};
use vex_rt::prelude::*;

struct UltrasonicBot {
    sensor: AdiUltrasonic,
}

impl Robot for UltrasonicBot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            sensor: (peripherals.port_a, peripherals.port_b).try_into().unwrap(),
        }
    }
    fn opcontrol(&'static self, ctx: Context) {
        let mut l = Loop::new(Duration::from_secs(1));
        loop {
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
            select! {
                _ = l.select() => {},
                _ = ctx.done() => break,
            }
        }
    }
}

entry!(UltrasonicBot);
