#[macro_export]
/// Specifies the entrypoint for the robot.
///
/// # Examples
///
/// ```
/// #![no_std]
/// #![no_main]
///
/// use vex_rt::{entry, Robot};
///
/// struct FooBot;
///
/// impl Robot for FooBot {
///     fn initialize() -> Self {
///         FooBot
///     }
/// }
///
/// entry!(FooBot);
/// ```
macro_rules! entry {
    ($robot_type:ty) => {
        static ROBOT: $crate::once::Once<($robot_type, $crate::robot::ContextWrapper)> =
            $crate::once::Once::new();

        #[no_mangle]
        unsafe extern "C" fn initialize() {
            let (robot, _) = ROBOT.call_once(|| {
                (
                    $crate::robot::Robot::initialize(),
                    $crate::robot::ContextWrapper::new(),
                )
            });
            $crate::robot::Robot::post_initialize(robot);
        }

        #[no_mangle]
        extern "C" fn opcontrol() {
            let (robot, wrapper) = ROBOT.get().unwrap();
            $crate::rtos::Task::spawn_ext(
                "opcontrol",
                $crate::rtos::Task::DEFAULT_PRIORITY,
                $crate::rtos::Task::DEFAULT_STACK_DEPTH,
                move || $crate::robot::Robot::opcontrol(robot, wrapper.replace()),
            )
            .unwrap();
        }

        #[no_mangle]
        extern "C" fn autonomous() {
            let (robot, wrapper) = ROBOT.get().unwrap();
            $crate::rtos::Task::spawn_ext(
                "autonomous",
                $crate::rtos::Task::DEFAULT_PRIORITY,
                $crate::rtos::Task::DEFAULT_STACK_DEPTH,
                move || $crate::robot::Robot::autonomous(robot, wrapper.replace()),
            )
            .unwrap();
        }

        #[no_mangle]
        extern "C" fn disabled() {
            let (robot, wrapper) = ROBOT.get().unwrap();
            $crate::rtos::Task::spawn_ext(
                "disabled",
                $crate::rtos::Task::DEFAULT_PRIORITY,
                $crate::rtos::Task::DEFAULT_STACK_DEPTH,
                move || $crate::robot::Robot::disabled(robot, wrapper.replace()),
            )
            .unwrap();
        }
    };
}
