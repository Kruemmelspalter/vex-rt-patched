#[macro_export]
/// Creates a loop meant to be used in an async context.
macro_rules! async_loop {
    ($robot_args:ident: ($loop_period:expr) $loop:expr) => {
        use $crate::prelude::concurrency_traits::TimeFunctions;
        let mut last = $crate::prelude::free_rtos::FreeRtosConcurrency::current_time();
        #[allow(clippy::unnecessary_operation)]
        loop {
            $loop;
            $robot_args
                .sleep_runner
                .sleep_until(last + $loop_period)
                .await;
            last = $crate::prelude::free_rtos::FreeRtosConcurrency::current_time();
        }
    };
}
