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

struct ClawBot {
    drive_train: Mutex<DriveTrain>,
}

impl Robot for ClawBot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            drive_train: Mutex::new(DriveTrain::new(peripherals.port08)),
        }
    }

    fn autonomous(&'static self, ctx: Context) {
        println!("autonomous");
        let mut l = Loop::new(Duration::from_millis(20));

        let drive_train = self.drive_train.lock();

        loop {
            println!("{}", drive_train.rotation_sensor.get_position().unwrap());

            select! {
                _ = ctx.done() => break,
                _ = l.select() => {},
            }
        }
    }
}

entry!(ClawBot);
