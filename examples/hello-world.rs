#![no_std]
#![no_main]

use vex_rt::prelude::*;

struct HelloBot;

impl Robot for HelloBot {
    fn initialize() -> Self {
        println!("Hello, world");
        HelloBot
    }
}

entry!(HelloBot);
