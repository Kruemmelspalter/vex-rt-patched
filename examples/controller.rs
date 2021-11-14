#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;

struct DriveTrain {
    left_motor: Motor,
    right_motor: Motor,
}

impl DriveTrain {
    fn spin(&mut self, velocity: i8) {
        self.left_motor.move_i8(velocity).unwrap();
        self.right_motor.move_i8(velocity).unwrap();
    }
}

struct ClawBot {
    controller: Controller,
    drive_train: VexAsyncMutex<DriveTrain>,
}

#[async_trait(?Send)]
impl Robot for ClawBot {
    async fn new(p: Peripherals) -> Self {
        ClawBot {
            controller: p.master_controller,
            drive_train: VexAsyncMutex::new(DriveTrain {
                left_motor: p.port01.into_motor(Gearset::EighteenToOne, false).unwrap(),
                right_motor: p.port02.into_motor(Gearset::EighteenToOne, true).unwrap(),
            }),
        }
    }

    async fn opcontrol(&'static self, robot_args: RobotArgs) {
        async_loop!(robot_args: (Duration::from_secs(1)){
            let mut drive_train = self.drive_train.lock_async().await;
            let velocity = self.controller.left_stick.get_x().unwrap();
            drive_train.spin(velocity);
        });
    }

    async fn disabled(&'static self, _robot_args: RobotArgs) {
        self.drive_train.lock_async().await.spin(0);
    }
}

entry!(ClawBot);
