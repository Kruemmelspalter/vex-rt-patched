//! A crate for running rust on the VEX V5.

#![no_main]
#![no_std]
#![feature(alloc_error_handler)]
#![feature(negative_impls)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, unused_import_braces)]

extern crate alloc;

use core::{panic::PanicInfo, time::Duration};

mod allocator;
mod bindings;
mod error;

pub mod adi;
pub mod battery;
pub mod competition;
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

#[cfg(feature = "async-await")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-await")))]
pub mod async_await;

#[doc(hidden)]
pub use spin::once;

pub use uom;

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    crate::io::eprintln!("panic occurred!: {:#?}", panic_info);
    crate::rtos::Task::delay(Duration::from_secs(1));

    unsafe {
        libc::exit(1);
    }
}
