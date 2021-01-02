#![no_std]
#![no_main]

use core::time::Duration;

use vex_rt::rtos::{Promise, Task};
use vex_rt::{prelude::*, select};

struct TaskBot;

impl Robot for TaskBot {
    fn initialize() -> Self {
        TaskBot
    }
    fn autonomous(&self, _: Context) {
        println!("autonomous");
    }
    fn opcontrol(&self, _: Context) {
        println!("opcontrol");
        let (promise, resolve) = Promise::<i32>::new();
        Task::spawn(|| {
            Task::delay(Duration::from_secs(1));
            resolve(10);
        })
        .unwrap();
        println!(
            "n = {}",
            select! {
                n = promise.done() => n,
            }
        );
    }
    fn disabled(&self, _: Context) {
        println!("disabled");
    }
}

entry!(TaskBot);
