#![no_std]
#![no_main]

extern crate alloc;

use core::time::Duration;

use alloc::format;
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

struct ControllerBot {
    controller: Controller,
    drive_train: DriveTrain,
}

impl Robot for ControllerBot {
    fn new(p: Peripherals) -> Self {
        ControllerBot {
            controller: p.master_controller,
            drive_train: DriveTrain {
                left_motor: p
                    .port01
                    .into_motor(Gearset::EighteenToOne, false)
                    .unwrap(),
                right_motor: p
                    .port02
                    .into_motor(Gearset::EighteenToOne, true)
                    .unwrap(),
            },
        }
    }

    fn opcontrol(&mut self, ctx: Context) {
        let mut l = Loop::new(Duration::from_millis(10));

        loop {
            let velocity = self.controller.left_stick.get_x().unwrap();
            self.controller
                .screen
                .print(0, 0, &format!("Vel: {:<4}", velocity));
            self.drive_train.spin(velocity);

            select! {
                _ = ctx.done() => break,
                _ = l.select() => continue,
            }
        }
    }

    fn disabled(&mut self, _ctx: Context) {
        self.drive_train.spin(0);
        self.controller.screen.clear();
    }

    fn initialize(&mut self, _ctx: Context) {
        self.controller.screen.clear();
        println!("level: {}", self.controller.get_battery_level().unwrap());
        println!(
            "capacity: {}",
            self.controller.get_battery_capacity().unwrap()
        );
    }
}

entry!(ControllerBot);
