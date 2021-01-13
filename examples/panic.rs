#![no_std]
#![no_main]

use vex_rt::prelude::*;

struct PanicBot;

impl Robot for PanicBot {
    fn new(_peripherals: Peripherals) -> Self {
        panic!("Panic Message")
    }
}

entry!(PanicBot);
