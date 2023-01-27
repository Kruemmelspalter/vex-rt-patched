#[macro_export]
/// Specifies the entrypoint for the robot.
///
/// # Examples
///
/// ```
/// #![no_std]
/// #![no_main]
///
/// use vex_rt::prelude::*;
///
/// struct FooBot;
///
/// impl Robot for FooBot {
///     fn new(_p: Peripherals) -> Self {
///         FooBot
///     }
/// }
///
/// entry!(FooBot);
/// ```
macro_rules! entry {
    ($robot_type:ty) => {
        static ROBOT: $crate::once::Once<$crate::robot::Competition<$robot_type>> =
            $crate::once::Once::new();

        #[no_mangle]
        unsafe extern "C" fn initialize() {
            ROBOT.call_once(|| {
                Competition::new($crate::robot::Robot::new(unsafe {
                    $crate::peripherals::Peripherals::new()
                }))
            });
        }

        #[no_mangle]
        extern "C" fn opcontrol() {
            ROBOT.wait().opcontrol();
        }

        #[no_mangle]
        extern "C" fn autonomous() {
            ROBOT.wait().autonomous();
        }

        #[no_mangle]
        extern "C" fn disabled() {
            ROBOT.wait().disabled();
        }
    };
}
