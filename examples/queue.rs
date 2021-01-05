#![no_std]
#![no_main]

extern crate alloc;

use core::time::Duration;

use alloc::sync::Arc;
use vex_rt::{
    prelude::*,
    rtos::{Loop, Queue, Task},
    select,
};

struct QueueRobot {
    queue: Arc<Queue<i32>>,
}

impl Robot for QueueRobot {
    fn initialize() -> Self {
        let queue = Arc::new(Queue::new(1));
        Task::spawn({
            let queue = queue.clone();
            move || {
                let mut x = 0;
                let mut l = Loop::new(Duration::from_secs(1));
                loop {
                    queue.append(x, None).unwrap();
                    x += 1;
                    l.delay();
                }
            }
        })
        .unwrap();
        Self { queue }
    }
    fn autonomous(&self, ctx: Context) {
        println!("autonomous");
        loop {
            select! {
                x = self.queue.select() => println!("{}", x),
                _ = ctx.done() => break,
            }
        }
        println!("auto done")
    }
    fn opcontrol(&self, ctx: Context) {
        println!("opcontrol");
        loop {
            select! {
                x = self.queue.select() => println!("{}", x),
                _ = ctx.done() => break,
            }
        }
    }
    fn disabled(&self, _: Context) {
        println!("disabled");
    }
}

entry!(QueueRobot);
