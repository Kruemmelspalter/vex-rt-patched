#![no_std]
#![no_main]

use core::{convert::TryInto, time::Duration};
use vex_rt::prelude::*;

struct DigitalOutBot {
    output: AdiDigitalOutput,
}

impl Robot for DigitalOutBot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            output: peripherals.port_a.try_into().unwrap(),
        }
    }
    fn opcontrol(&mut self, ctx: Context) {
        let mut l = Loop::new(Duration::from_secs(1));
        let mut value = false;

        loop {
            println!("{}", value);
            self.output.write(value).unwrap();
            value = !value;
            select! {
                _ = l.select() => {},
                _ = ctx.done() => break,
            }
        }
    }
}

entry!(DigitalOutBot);
