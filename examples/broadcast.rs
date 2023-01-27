#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;

struct BroadcastBot {
    listener: BroadcastListener<i32>,
}

impl Robot for BroadcastBot {
    fn new(_peripherals: Peripherals) -> Self {
        let bcast = Broadcast::new(0);
        let listener = bcast.listen();
        let mut x = 0;
        let mut l = Loop::new(Duration::from_secs(1));
        Task::spawn(move || loop {
            x += 1;
            bcast.publish(x);
            l.delay()
        })
        .unwrap();

        Self { listener }
    }

    fn opcontrol(&mut self, ctx: Context) {
        println!("opcontrol");
        loop {
            select! {
                x = self.listener.select() => println!("{}", x),
                _ = ctx.done() => break,
            }
        }
    }
}

entry!(BroadcastBot);
