#![no_std]
#![no_main]

use core::{convert::TryInto, time::Duration};
use vex_rt::prelude::*;

struct DigitalInBot {
    sensor: AdiDigitalInput,
}

impl Robot for DigitalInBot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            sensor: peripherals.port_a.try_into().unwrap(),
        }
    }
    fn opcontrol(&mut self, ctx: Context) {
        let mut l = Loop::new(Duration::from_secs(1));
        loop {
            println!("{}", self.sensor.read().unwrap());
            select! {
                _ = l.select() => {},
                _ = ctx.done() => break,
            }
        }
    }
}

entry!(DigitalInBot);
