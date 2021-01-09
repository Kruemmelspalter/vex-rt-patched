#![no_std]
#![no_main]

extern crate alloc;

use core::time::Duration;

use alloc::sync::Arc;
use vex_rt::prelude::*;
use vex_rt::{
    rtos::{Broadcast, Loop, Task},
    select,
};

struct BroadcastBot {
    bcast: Arc<Broadcast<i32>>,
}

impl Robot for BroadcastBot {
    fn initialize() -> Self {
        let bcast = Arc::new(Broadcast::new(0));
        let bot = BroadcastBot {
            bcast: bcast.clone(),
        };
        let mut x = 0;
        let mut l = Loop::new(Duration::from_secs(1));
        Task::spawn(move || loop {
            x += 1;
            bcast.publish(x);
            l.delay()
        })
        .unwrap();
        bot
    }
    fn autonomous(&self, _: Context) {
        println!("autonomous");
    }
    fn opcontrol(&self, ctx: Context) {
        println!("opcontrol");
        let mut l = self.bcast.listen();
        loop {
            select! {
                x = l.select() => println!("{}", x),
                _ = ctx.done() => break,
            }
        }
    }
    fn disabled(&self, _: Context) {
        println!("disabled");
    }
}

entry!(BroadcastBot);
