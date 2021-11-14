#![no_std]
#![no_main]

use vex_rt::prelude::*;

struct HelloBot;

#[async_trait(?Send)]
impl Robot for HelloBot {
    async fn new(_peripherals: Peripherals) -> Self {
        println!("Hello, world");
        HelloBot
    }
}

entry!(HelloBot);
