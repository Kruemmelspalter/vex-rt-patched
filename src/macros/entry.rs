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
    ($robot_type:ty, $sleep_queue_size:expr, $executor_queue_size:expr) => {
        #[no_mangle]
        unsafe extern "C" fn initialize() {
            $crate::entry_function::initialize_entry::<$robot_type>(
                $sleep_queue_size,
                $executor_queue_size,
            );
        }

        #[no_mangle]
        unsafe extern "C" fn opcontrol() {
            unreachable!();
        }

        #[no_mangle]
        unsafe extern "C" fn autonomous() {
            unreachable!();
        }

        #[no_mangle]
        unsafe extern "C" fn disabled() {
            unreachable!();
        }
    };
    ($robot_type:ty) => {
        entry!($robot_type, 2048, 2048);
    };
}
