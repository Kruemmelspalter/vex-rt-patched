#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::{prelude::*, state_machine2};
use vex_rt_macros::make_state_machine;

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
        let left: i8 = match y as i16 + x as i16 {
            v if v < -127 => -127,
            v if v > 127 => 127,
            v => v as i8,
        };
        let right: i8 = match y as i16 - x as i16 {
            v if v < -127 => -127,
            v if v > 127 => 127,
            v => v as i8,
        };
        self.left.move_i8(left)?;
        self.right.move_i8(right)
    }
}

// mod drive_state_machine {
//     vex_rt::state_machine! {
//         pub Drive(drive: super::DriveTrain) {
//             drive: super::DriveTrain = drive,
//         } = idle();

//         idle(ctx) [drive] {
//             drive.drive(0, 0).unwrap();
//         }
//     }
// }

state_machine2! {
    /// Test
    Drive(drive: DriveTrain) {
        drive: DriveTrain = drive,
    } = idle;

    idle(ctx) [drive] {
        drive.drive(0, 0).unwrap();
    }
}

struct Bot {
    controller: Controller,
    drivetrain: Mutex<DriveTrain>,
}

impl Bot {
    // Waits for access to the drivetrain, then passes
    // its arguments to the drive method of the drivetrain.
    fn drive(&self, x: i8, y: i8) -> Result<(), MotorError> {
        self.drivetrain.lock().drive(x, y)
    }
}

impl Robot for Bot {
    fn new(p: Peripherals) -> Self {
        Bot {
            controller: p.master_controller,
            drivetrain: Mutex::new(DriveTrain {
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

    // This function will get invoked when the robot is placed
    // under operator control.
    fn opcontrol(&mut self, ctx: Context) {
        let mut pause = Loop::new(Duration::from_millis(100));

        // We will run a loop to check controls on the controller and
        // perform appropriate actions.
        loop {
            // Each time through the loop we read the right joystick and
            // feed its x and y values to the drivetrain.
            // The joytick is spring-loaded to return to 0 so the robot
            // will stop unless the operator intervenes. The further the
            // joystick is from 0, the faster robot will move.
            self.drive(
                self.controller.right_stick.get_x().unwrap(),
                self.controller.right_stick.get_y().unwrap(),
            )
            .expect("Drivetrain error");

            // At the end of each loop pause.select() will pause for 100 ms,
            // then generate a selectable event. ctx.done() will also generate
            // a selectable event if the opcontrol period has ended. If
            // ctx.done() generates an event before pause generates an event,
            // we will exit the loop.
            select! {
                _ = ctx.done() => break,
                _ = pause.select() => continue
            }
        }
    }
}

entry!(Bot);
