#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::{prelude::*, select_merge};

struct SelectRobot;

impl Robot for SelectRobot {
    fn new(_peripherals: Peripherals) -> Self {
        Self
    }

    fn autonomous(&mut self, ctx: Context) {
        println!("autonomous");
        let mut x = 0;
        let mut l = Loop::new(Duration::from_secs(1));
        loop {
            println!("{}", x);
            x += 1;
            let event = select_merge! {
                _ = l.select() => false,
                _ = ctx.done() => true,
            };
            if select! { b = event => b } {
                break;
            }
        }
        println!("auto done")
    }
}

entry!(SelectRobot);
