#![no_std]
#![no_main]

extern crate alloc;

use core::time::Duration;
use uom::si::{
    angle::{degree, revolution},
    angular_velocity::revolution_per_minute,
    f64::{Angle, AngularVelocity},
};
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

    fn drive_distance(&mut self, distance: Angle, ctx: Context) -> Result<bool, MotorError> {
        let velocity = AngularVelocity::new::<revolution_per_minute>(100.0);
        self.left.move_relative(distance, velocity)?;
        self.right.move_relative(distance, velocity)?;

        let mut pause = Loop::new(Duration::from_millis(10));

        let threshold = Angle::new::<degree>(1.0);
        while (self.left.get_position()? - self.left.get_target_position()?).abs() >= threshold
            || (self.right.get_position()? - self.right.get_target_position()?).abs() >= threshold
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
    idle(_ctx) {
        self.drive.drive(0, 0).unwrap_or_else(|err| {
            eprintln!("idle drive error: {:?}", err);
        });
    }

    /// Manual control state.
    manual(ctx, mut controller: BroadcastListener<ControllerData>) {
        loop {
            select! {
                _ = ctx.done() => break,
                data = controller.select() => self.drive.drive(data.left_x, data.left_y).unwrap_or_else(|err| {
                    eprintln!("manual drive error: {:?}", err);
                }),
            };
        }
    }

    /// Drives forward a set amount.
    ///
    /// Returns whether the movement completed successfully.
    auto_drive(ctx, distance: Angle) -> bool {
        let result = self.drive.drive_distance(distance, ctx).unwrap_or_else(|err| {
            eprintln!("auto drive error: {:?}", err);
            false
        });
        return StateResult::Transition(result, DriveState::Idle);
    }
}

struct Bot {
    controller: BroadcastWrapper<Controller>,
    drive: Drive,
}

impl Robot for Bot {
    fn new(p: Peripherals) -> Self {
        Bot {
            controller: p.master_controller.into_broadcast().unwrap(),
            drive: Drive::new(DriveTrain {
                left: p.port01.into_motor(Gearset::EighteenToOne, false).unwrap(),
                right: p.port10.into_motor(Gearset::EighteenToOne, true).unwrap(),
            }),
        }
    }

    fn autonomous(&mut self, ctx: Context) {
        // Tells the drive to move a given distance, with a time limit of 1 second.
        let auto = self.drive.auto_drive_ext(
            ctx.fork_with_timeout(Duration::from_secs(1)),
            Angle::new::<revolution>(1.0),
        );

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

        self.drive.manual(self.controller.listen());

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
                _ = pause.select() => self.controller.update().unwrap(),
            };
        }
    }

    fn disabled(&mut self, _ctx: Context) {
        self.drive.idle();
    }
}

entry!(Bot);
