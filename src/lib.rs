//! A crate for running rust on the VEX V5.

#![no_std]
#![feature(alloc_error_handler)]
#![warn(missing_docs, unused_import_braces, missing_debug_implementations)]

extern crate alloc;

use core::panic::PanicInfo;

mod allocator;
mod bindings;
mod error;

pub mod adi;
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

/// Used to ensure a type is [`Send`] and give a compiler error if it isn't
trait EnsureSend: Send {}
/// Used to ensure a type is [`Sync`] and give a compiler error if it isn't
trait EnsureSync: Sync {}
