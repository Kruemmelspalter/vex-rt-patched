#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;

struct BroadcastBot {
    bcast: Broadcast<i32>,
}

impl Robot for BroadcastBot {
    fn new(_peripherals: Peripherals) -> Self {
        Self {
            bcast: Broadcast::new(0),
        }
    }
    fn initialize(&'static self, _ctx: Context) {
        let mut x = 0;
        let mut l = Loop::new(Duration::from_secs(1));
        Task::spawn(move || loop {
            x += 1;
            self.bcast.publish(x);
            l.delay()
        })
        .unwrap();
    }
    fn opcontrol(&'static self, ctx: Context) {
        println!("opcontrol");
        let mut l = self.bcast.listen();
        loop {
            select! {
                x = l.select() => println!("{}", x),
                _ = ctx.done() => break,
            }
        }
    }
}

entry!(BroadcastBot);
