//! A crate for running rust on the VEX V5.

#![no_std]
#![feature(alloc_error_handler)]
#![feature(negative_impls)]
#![warn(missing_docs)]

extern crate alloc;

use core::panic::PanicInfo;

mod allocator;
mod bindings;
mod error;

pub mod adi;
pub mod battery;
pub mod controller;
pub mod distance;
pub mod imu;
pub mod io;
pub mod machine;
pub mod macros;
pub mod motor;
pub mod peripherals;
pub mod prelude;
pub mod robot;
pub mod rotation;
pub mod rtos;
pub mod serial;
pub mod smart_port;

#[doc(hidden)]
pub use spin::once;

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    crate::io::eprintln!("panic occurred!: {:#?}", panic_info);

    unsafe {
        libc::exit(1);
    }
}
