#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;

struct ChannelBot {
    chan: ReceiveChannel<i32>,
}

impl Robot for ChannelBot {
    fn new(_peripherals: Peripherals) -> Self {
        let (send, receive) = channel();
        let mut x = 0;
        let mut l = Loop::new(Duration::from_secs(1));
        Task::spawn(move || loop {
            x += 1;
            select! {
                _ = send.select(x) => {},
            }
            l.delay();
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

entry!(ChannelBot);
