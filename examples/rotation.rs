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
    drive_train: VexAsyncMutex<DriveTrain>,
}

#[async_trait(?Send)]
impl Robot for ClawBot {
    async fn new(peripherals: Peripherals) -> Self {
        Self {
            drive_train: VexAsyncMutex::new(DriveTrain::new(peripherals.port08)),
        }
    }

    async fn autonomous(&'static self, robot_args: RobotArgs) {
        println!("autonomous");
        let drive_train = self.drive_train.lock_async().await;

        async_loop!(robot_args: (Duration::from_millis(20)) {
            println!("{:?}", drive_train.rotation_sensor.get_position().unwrap());
        });
    }
}

entry!(ClawBot);
