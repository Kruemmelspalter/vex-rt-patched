#![no_std]
#![no_main]

use vex_rt::prelude::*;

struct PanicBot;

#[async_trait(?Send)]
impl Robot for PanicBot {
    async fn new(_peripherals: Peripherals) -> Self {
        panic!("Panic Message")
    }
}

entry!(PanicBot);
