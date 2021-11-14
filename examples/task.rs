// TODO: Need to support naming and priority of tasks

#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;
use vex_rt::rtos::free_rtos::FreeRtosConcurrency;

struct TaskBot;

#[async_trait(?Send)]
impl Robot for TaskBot {
    async fn new(_peripherals: Peripherals) -> Self {
        let mut x = 0;
        Task::spawn_ext(
            "test",
            Task::DEFAULT_PRIORITY,
            Task::DEFAULT_STACK_DEPTH,
            move || {
                println!("Task name: {}", Task::current().name());
                let mut last = FreeRtosConcurrency::current_time();
                loop {
                    println!("{}", x);
                    x += 1;
                    FreeRtosConcurrency::sleep(
                        FreeRtosConcurrency::current_time() - last + Duration::from_secs(1),
                    );
                    last = FreeRtosConcurrency::current_time();
                }
            },
        )
        .unwrap();
        TaskBot
    }
}

entry!(TaskBot);
