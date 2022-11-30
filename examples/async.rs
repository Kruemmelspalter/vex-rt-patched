#![no_std]
#![no_main]

use core::time::Duration;
use futures::FutureExt;
use vex_rt::{async_await::launch, prelude::*};

struct AsyncBot;

impl Robot for AsyncBot {
    fn new(_peripherals: Peripherals) -> Self {
        Self
    }

    fn opcontrol(&'static self, ctx: Context) {
        println!("opcontrol");

        let dispatcher = launch(ctx.clone());
        let (send, receive) = channel();

        dispatcher.dispatch(20, move |ec| {
            println!("async init");
            async move {
                println!("async start");
                loop {
                    let x = ec.proxy(receive.select()).await;
                    println!("{}", x);
                }
            }
            .boxed_local()
        });

        let mut l = Loop::new(Duration::from_millis(750));
        let mut x = 0;
        loop {
            select! {
                _ = send.select(x) => {
                    println!("sent: {}", x);
                    x += 1;
                },
                _ = ctx.done() => break,
            }
            select! {
                _ = l.select() => continue,
                _ = ctx.done() => break,
            }
        }
    }
}

entry!(AsyncBot);
