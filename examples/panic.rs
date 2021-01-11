#![no_std]
#![no_main]

use vex_rt::prelude::*;

struct PanicBot;

impl Robot for PanicBot {
    fn initialize() -> Self {
        panic!("Panic Message")
    }
}

entry!(PanicBot);
