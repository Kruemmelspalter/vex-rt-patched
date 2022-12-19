#![no_std]
#![no_main]

extern crate alloc;

use alloc::sync::Arc;
use core::time::Duration;
use num_traits::float::FloatCore;
use vex_rt::{prelude::*, state_machine};

struct DriveTrain {
    left: Motor,
    right: Motor,
}

impl DriveTrain {
    // This is meant to take input directly from a joystick
    // and rotate the left and right tires at different speeds.
    // It is possible that the combination of x value and y value
    // could exceed 127 and be over the limit of an i8.
    // So we catch those cases and bring them back within bounds.
    fn drive(&mut self, x: i8, y: i8) -> Result<(), MotorError> {
        let left = (y as i16 + x as i16).clamp(-127, 127) as i8;
        let right = (y as i16 - x as i16).clamp(-127, 127) as i8;
        self.left.move_i8(left)?;
        self.right.move_i8(right)?;
        Ok(())
    }

    fn drive_distance(&mut self, distance: f64, ctx: Context) -> Result<bool, MotorError> {
        self.left.move_relative(distance, 100)?;
        self.right.move_relative(distance, 100)?;

        let mut pause = Loop::new(Duration::from_millis(10));

        while (self.left.get_position()? - self.left.get_target_position()?).abs() >= 1.0
            || (self.right.get_position()? - self.right.get_target_position()?).abs() >= 1.0
        {
            select! {
                _ = ctx.done() => return Ok(false),
                _ = pause.select() => continue,
            };
        }

        Ok(true)
    }
}

state_machine! {
    /// Test
    Drive(drive: DriveTrain) {
        drive: DriveTrain = drive,
    } = idle;

    /// Idle state.
    idle(_ctx) [drive] {
        drive.drive(0, 0).unwrap();
    }

    /// Manual control state.
    manual(ctx, controller: Arc<ControllerBroadcast>) [drive] {
        let mut l = controller.listen();

        loop {
            select! {
                _ = ctx.done() => break,
                data = l.select() => drive.drive(data.left_x, data.left_y).unwrap(),
            };
        }
    }

    /// Drives forward a set amount.
    auto_drive(ctx, distance: f64) [drive] -> bool {
        drive.drive_distance(distance, ctx).unwrap_or_else(|err| {
            eprintln!("drive error: {:?}", err);
            false
        })
    }
}

struct Bot {
    controller: Arc<ControllerBroadcast>,
    drive: Drive,
}

impl Robot for Bot {
    fn new(p: Peripherals) -> Self {
        Bot {
            controller: Arc::new(p.master_controller.into_broadcast()),
            drive: Drive::new(DriveTrain {
                left: p
                    .port01
                    .into_motor(Gearset::EighteenToOne, EncoderUnits::Degrees, false)
                    .unwrap(),
                right: p
                    .port10
                    .into_motor(Gearset::EighteenToOne, EncoderUnits::Degrees, true)
                    .unwrap(),
            }),
        }
    }

    fn autonomous(&mut self, ctx: Context) {
        let auto = self.drive.auto_drive(100.0);
        select! {
            success = auto.done() => if *success {
                println!("success");
            } else {
                println!("failed");
            },
            _ = ctx.done() => {},
        }
    }

    // This function will get invoked when the robot is placed
    // under operator control.
    fn opcontrol(&mut self, ctx: Context) {
        let mut pause = Loop::new(Duration::from_millis(100));

        self.drive.manual(self.controller.clone());

        // We will run a loop to check controls on the controller and
        // perform appropriate actions.
        loop {
            // At the end of each loop pause.select() will pause for 100 ms,
            // then generate a selectable event. ctx.done() will also generate
            // a selectable event if the opcontrol period has ended. If
            // ctx.done() generates an event before pause generates an event,
            // we will exit the loop.
            select! {
                _ = ctx.done() => break,
                _ = pause.select() => self.controller.broadcast_update().unwrap(),
            };
        }
    }

    fn disabled(&mut self, _ctx: Context) {
        self.drive.idle();
    }
}

entry!(Bot);
