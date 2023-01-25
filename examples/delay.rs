#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;

struct DelayBot;

impl Robot for DelayBot {
    fn new(_peripherals: Peripherals) -> Self {
        Self
    }

    fn opcontrol(&mut self, _ctx: Context) {
        let x: u32 = 0;
        loop {
            println!("x = {}", x);
            Task::delay(Duration::from_secs(1));
        }
    }
}

entry!(DelayBot);
