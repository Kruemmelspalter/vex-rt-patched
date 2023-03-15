#![no_std]
#![no_main]

use core::time::Duration;
use qunit::angular_velocity::AngularVelocityExt;
use vex_rt::prelude::*;

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

struct Arm(Motor);

struct Claw(Motor);

struct Bot {
    controller: Controller,
    drivetrain: DriveTrain,
    arm: Arm,
    claw: Claw,
}

impl Bot {
    // Waits for access to the drivetrain, then passes
    // its arguments to the drive method of the drivetrain.
    fn drive(&mut self, x: i8, y: i8) -> Result<(), MotorError> {
        self.drivetrain.drive(x, y)
    }

    // Waits for access to the claw, then tells the motor to
    // rotate at the specified speed.
    // Once the motor begins to rotate, its movement is checked
    // every 100 ms. If the motor fails to move on five
    // consecutive checks, it will be shut down.
    // This allows the claw to close or open as much as it can,
    // then stop automatically.
    // The `grip` call will block until the claw stops. This will
    // take a minimum of 500 ms.
    fn grip(&mut self, speed: i8) -> Result<(), MotorError> {
        let mut flag: u8 = 0;
        self.claw.0.move_i8(speed)?;
        let threshold = 10.0.rpm();
        while flag < 5 {
            flag = match self.claw.0.get_actual_velocity() {
                Ok(v) if (v > -threshold && v < threshold) => flag + 1,
                Ok(_) => 0,
                _ => flag,
            };
            Task::delay(Duration::from_millis(100));
        }
        // If whe set the motor to 0, it will relax. That is probably
        // undesirable for gripping. So if the claw is supposed to close
        // (0 < speed) we will drop the setting way down, but not
        // completely to 0. It is not recommended to keep the claw in a
        // gripping state for extended periods.
        match 0 < speed {
            true => self.claw.0.move_i8(6),
            false => self.claw.0.move_i8(0),
        }
    }

    // Waits for access to the arm, then rotates the motor
    // to move the arm up and down at the specified speed.
    fn lift(&mut self, velocity: i8) -> Result<(), MotorError> {
        self.arm.0.move_i8(velocity)
    }
}

impl Robot for Bot {
    fn new(p: Peripherals) -> Self {
        Bot {
            controller: p.master_controller,
            drivetrain: DriveTrain {
                left: p.port01.into_motor(Gearset::EighteenToOne, false).unwrap(),
                right: p.port10.into_motor(Gearset::EighteenToOne, true).unwrap(),
            },
            arm: Arm(p.port08.into_motor(Gearset::EighteenToOne, true).unwrap()),
            claw: Claw(p.port03.into_motor(Gearset::EighteenToOne, false).unwrap()),
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

            // Each time through the loop we read the y value of the
            // left joystick and feed its value to the arm.
            self.lift(self.controller.left_stick.get_y().unwrap())
                .expect("Arm error");

            // Each time through the loop we check to see if the L1 or L2
            // buttons are being pressed.
            // If the L1 button is being presses we tell the claw to close.
            // If the L2 button is being presses we tell the claw to open.
            // Opening or closing the claw will block and the loop will
            // not continue until the claw finishes blocking.
            if let Ok(pressed) = self.controller.l1.is_pressed() {
                if pressed {
                    self.grip(127).expect("Claw error");
                }
            }
            if let Ok(pressed) = self.controller.l2.is_pressed() {
                if pressed {
                    self.grip(-127).expect("Claw error");
                }
            }

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
