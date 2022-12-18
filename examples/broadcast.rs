#![no_std]
#![no_main]

extern crate alloc;

use alloc::sync::Arc;
use core::time::Duration;
use vex_rt::prelude::*;

struct BroadcastBot {
    bcast: Arc<Broadcast<i32>>,
}

impl Robot for BroadcastBot {
    fn new(_peripherals: Peripherals) -> Self {
        let bcast = Arc::new(Broadcast::new(0));
        let bcast2 = bcast.clone();
        let mut x = 0;
        let mut l = Loop::new(Duration::from_secs(1));
        Task::spawn(move || loop {
            x += 1;
            bcast2.publish(x);
            l.delay()
        })
        .unwrap();

        Self { bcast }
    }

    fn opcontrol(&mut self, ctx: Context) {
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
