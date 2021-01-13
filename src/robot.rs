//! For use with the [`entry`] macro.

use crate::{
    peripherals::Peripherals,
    rtos::{Context, Mutex},
};
use libc_print::std_name::*;

/// A trait representing a competition-ready VEX Robot.
pub trait Robot {
    /// Runs at startup, contstructing your robot. This should be non-blocking,
    /// since the FreeRTOS scheduler doesn't start until it returns.
    fn new(peripherals: Peripherals) -> Self;

    /// Runs immediately after [`Robot::new`]. This should be
    /// non-blocking, since the FreeRTOS scheduler doesn't start until it
    /// returns.
    ///
    /// The purpose of this method is to provide a hook to run things on startup
    /// which require a reference to all or part of the robot structure; since
    /// it takes `&'static self` as its parameter, the lifetime of the robot
    /// object is guaranteed to be static (i.e., forever), and so the
    /// implementation may pass references around (e.g., to new tasks) at will
    /// without issue.
    fn initialize(&'static self) {}

    /// Runs during the autonomous period.
    fn autonomous(&'static self, _ctx: Context) {
        println!("autonomous");
    }

    /// Runs during the opcontrol period.
    fn opcontrol(&'static self, _ctx: Context) {
        println!("opcontrol");
    }

    /// Runs when the robot is disabled.
    fn disabled(&'static self, _ctx: Context) {
        println!("disabled");
    }
}

#[doc(hidden)]
pub struct ContextWrapper(Mutex<Option<Context>>);

impl ContextWrapper {
    #[doc(hidden)]
    #[inline]
    #[allow(clippy::clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }

    #[doc(hidden)]
    pub fn replace(&self) -> Context {
        let mut opt = self.0.lock();
        if let Some(ctx) = opt.take() {
            ctx.cancel();
        }
        let ctx = Context::new_global();
        *opt = Some(ctx.clone());
        ctx
    }
}
