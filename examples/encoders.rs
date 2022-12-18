#![no_std]
#![no_main]

use core::convert::TryInto;
use core::time::Duration;
use vex_rt::prelude::*;

struct DriveTrain {
    encoder: AdiEncoder,
}

impl DriveTrain {
    fn new(encoder_port_left: AdiPort, encoder_port_right: AdiPort) -> Self {
        Self {
            encoder: (encoder_port_left, encoder_port_right).try_into().unwrap(),
        }
    }
}

struct ClawBot {
    drive_train: DriveTrain,
}

impl Robot for ClawBot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            drive_train: DriveTrain::new(peripherals.port_a, peripherals.port_b),
        }
    }

    fn autonomous(&mut self, ctx: Context) {
        println!("autonomous");
        let mut l = Loop::new(Duration::from_millis(20));

        loop {
            println!("{}", self.drive_train.encoder.get().unwrap());

            select! {
                _ = ctx.done() => break,
                _ = l.select() => {},
            }
        }
    }
}

entry!(ClawBot);
