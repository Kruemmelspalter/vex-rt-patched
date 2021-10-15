#![no_std]
#![no_main]

use core::{convert::TryInto, time::Duration};
use vex_rt::prelude::*;

struct MotorBot {
    motor: AdiMotor,
}

impl Robot for MotorBot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            motor: peripherals.port_g.try_into().unwrap(),
        }
    }
    fn opcontrol(&'static self, ctx: Context) {
        let mut l = Loop::new(Duration::from_secs(1));
        loop {
            println!("{}", self.motor.read().unwrap());
            self.motor.write(53).unwrap();
            select! {
                _ = l.select() => {},
                _ = ctx.done() => break,
            }
        }
    }
}

entry!(MotorBot);
