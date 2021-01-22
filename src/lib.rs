//! A crate for running rust on the VEX V5.

#![no_std]
#![feature(alloc_error_handler)]
#![feature(negative_impls)]
#![feature(unsafe_cell_raw_get)]
#![warn(missing_docs)]

extern crate alloc;

use core::panic::PanicInfo;

mod allocator;
mod bindings;
mod error;
mod util;

pub mod io;
pub mod machine;
pub mod macros;
pub mod motor;
pub mod peripherals;
pub mod prelude;
pub mod robot;
pub mod rtos;
pub mod smart_port;

#[doc(hidden)]
pub use spin::once;

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    crate::io::eprintln!("panic occurred!: {:?}", panic_info);

    unsafe {
        libc::exit(1);
    }
}
