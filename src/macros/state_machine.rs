#[macro_export]
/// Creates an asynchronous state machine with the given visibility, name and
/// state definitions.
macro_rules! state_machine {
    ($($args:tt)*) => {
        $crate::macros::make_state_machine!($crate; $($args)*);
    };
}
