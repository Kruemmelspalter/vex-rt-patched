#![no_std]
#![no_main]

use core::time::Duration;

use queue_model::StaticPriorityQueue;
use vex_rt::prelude::*;

struct QueueBot {
    chan: ReceiveQueue<i32>,
}

impl Robot for QueueBot {
    fn new(_peripherals: Peripherals) -> Self {
        let (send, receive) = queue(StaticPriorityQueue::<i32, 20>::new());
        for x in 0.. {
            if !send.send(x) {
                break;
            }
        }
        Task::spawn(move || {
            Task::delay(Duration::from_secs(1));
            send.send(3);
        })
        .unwrap();
        Self { chan: receive }
    }

    fn opcontrol(&mut self, ctx: Context) {
        println!("opcontrol");
        loop {
            select! {
                x = self.chan.select() => println!("{}", x),
                _ = ctx.done() => break,
            }
        }
    }
}

entry!(QueueBot);
