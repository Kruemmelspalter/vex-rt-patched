#![no_std]
#![no_main]

use core::{convert::TryInto, time::Duration};
use vex_rt::prelude::*;

struct DigitalBot {
    input: AdiDigitalInput,
    output: AdiDigitalOutput,
}

impl Robot for DigitalBot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            input: peripherals.port_g.try_into().unwrap(),
            output: peripherals.port_b.try_into().unwrap(),
        }
    }
    fn opcontrol(&'static self, ctx: Context) {
        let mut l = Loop::new(Duration::from_secs(1));
        loop {
            println!("{}", self.input.read().unwrap());
            self.output.write(false).unwrap();
            select! {
                _ = l.select() => {},
                _ = ctx.done() => break,
            }
        }
    }
}

entry!(DigitalBot);
