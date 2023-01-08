#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
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
    controller: Mutex<Controller>,
    drive_train: Mutex<DriveTrain>,
}

impl Robot for ClawBot {
    fn new(p: Peripherals) -> Self {
        ClawBot {
            controller: Mutex::new(p.master_controller),
            drive_train: Mutex::new(DriveTrain {
                left_motor: p
                    .port01
                    .into_motor(Gearset::EighteenToOne, EncoderUnits::Degrees, false)
                    .unwrap(),
                right_motor: p
                    .port02
                    .into_motor(Gearset::EighteenToOne, EncoderUnits::Degrees, true)
                    .unwrap(),
            }),
        }
    }

    fn opcontrol(&self, ctx: Context) {
        let mut l = Loop::new(Duration::from_millis(10));
        let mut drive_train = self.drive_train.lock();
        let mut controller = self.controller.lock();

        controller.screen.rumble(".-  .-");

        loop {
            let velocity = controller.left_stick.get_x().unwrap();
            controller
                .screen
                .print(0, 0, &format!("Vel: {:<4}", velocity));
            drive_train.spin(velocity);

            select! {
                _ = ctx.done() => break,
                _ = l.select() => continue,
            }
        }
    }

    fn disabled(&self, _ctx: Context) {
        self.drive_train.lock().spin(0);
        self.controller.lock().screen.clear();
    }

    fn initialize(&self, _ctx: Context) {
        self.controller.lock().screen.clear();
        println!(
            "level: {}",
            self.controller.lock().get_battery_level().unwrap()
        );
        println!(
            "capacity: {}",
            self.controller.lock().get_battery_capacity().unwrap()
        );
    }
}

entry!(ClawBot);
