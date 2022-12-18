//! For use with the [`entry!`](crate::entry!) macro.

use crate::{io::println, peripherals::Peripherals, rtos::Context, state_machine2};

/// A trait representing a competition-ready VEX Robot.
pub trait Robot: Send + Sync + 'static {
    /// Runs at startup, constructing your robot. This should be non-blocking,
    /// since the FreeRTOS scheduler doesn't start until it returns.
    fn new(peripherals: Peripherals) -> Self;

    /// Runs immediately after [`Robot::new`]. The FreeRTOS scheduler is running
    /// by this point.
    ///
    /// The purpose of this method is to provide a hook to run things on startup
    /// which require a reference to all or part of the robot structure; since
    /// it takes `&'static self` as its parameter, the lifetime of the robot
    /// object is guaranteed to be static (i.e., forever), and so the
    /// implementation may pass references around (e.g., to new tasks) at will
    /// without issue.
    fn initialize(&mut self, _ctx: Context) {}

    /// Runs during the autonomous period.
    fn autonomous(&mut self, _ctx: Context) {
        println!("autonomous");
    }

    /// Runs during the opcontrol period.
    fn opcontrol(&mut self, _ctx: Context) {
        println!("opcontrol");
    }

    /// Runs when the robot is disabled.
    fn disabled(&mut self, _ctx: Context) {
        println!("disabled");
    }
}

state_machine2! {
    /// Competition state machine.
    pub Competition<R: Robot>(robot: R) {
        robot: R = robot,
    } = initialize;

    /// Runs on initialization.
    initialize(ctx) [robot] {
        robot.initialize(ctx);
    }

    /// Runs during the autonomous period.
    autonomous(ctx) [robot] {
        robot.autonomous(ctx);
    }

    /// Runs during the opcontrol period.
    opcontrol(ctx) [robot] {
        robot.opcontrol(ctx);
    }

    /// Runs when the robot is disabled.
    disabled(ctx) [robot] {
        robot.disabled(ctx);
    }
}
