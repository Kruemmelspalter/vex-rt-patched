#![no_std]
#![no_main]

use core::time::Duration;

use vex_rt::prelude::*;

struct SelectRobot;

impl Robot for SelectRobot {
    fn new(_peripherals: Peripherals) -> Self {
        Self
    }
    fn autonomous(&'static self, ctx: Context) {
        println!("autonomous");
        let mut x = 0;
        let mut l = Loop::new(Duration::from_secs(1));
        loop {
            println!("{}", x);
            x += 1;
            select! {
                _ = l.select() => {},
                _ = ctx.done() => break,
            }
        }
        println!("auto done")
    }
}

entry!(SelectRobot);
