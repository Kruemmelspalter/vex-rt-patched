#![no_std]
#![no_main]

use core::convert::TryInto;
use core::time::Duration;
use vex_rt::prelude::*;

struct DriveTrain {
    rotation_sensor: RotationSensor,
}

impl DriveTrain {
    fn new(rotation_sensor_port: SmartPort) -> Self {
        Self {
            rotation_sensor: (rotation_sensor_port, false).try_into().unwrap(),
        }
    }
}

struct RotationBot {
    drive_train: DriveTrain,
}

impl Robot for RotationBot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            drive_train: DriveTrain::new(peripherals.port08),
        }
    }

    fn autonomous(&mut self, ctx: Context) {
        println!("autonomous");
        let mut l = Loop::new(Duration::from_millis(20));

        loop {
            println!(
                "{:?}",
                self.drive_train.rotation_sensor.get_position().unwrap()
            );

            select! {
                _ = ctx.done() => break,
                _ = l.select() => {},
            }
        }
    }
}

entry!(RotationBot);
