#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;

struct PromiseBot;

impl Robot for PromiseBot {
    fn new(_peripherals: Peripherals) -> Self {
        PromiseBot
    }

    fn opcontrol(&mut self, _ctx: Context) {
        println!("opcontrol");
        let promise = Promise::spawn(|| {
            Task::delay(Duration::from_secs(1));
            10
        });
        println!(
            "n = {}",
            select! {
                n = promise.done() => n,
            }
        );
    }
}

entry!(PromiseBot);
