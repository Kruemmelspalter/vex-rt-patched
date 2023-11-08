//! A crate for running rust on the VEX V5.

#![no_main]
#![no_std]
#![feature(alloc_error_handler)]
#![feature(array_methods)]
#![feature(array_try_map)]
#![feature(generic_const_exprs)]
#![feature(negative_impls)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(incomplete_features)]
#![warn(missing_docs, unused_import_braces)]

#[doc(hidden)]
pub extern crate alloc;

use core::{panic::PanicInfo, time::Duration};

mod allocator;
mod bindings;
mod error;

pub mod adi;
pub mod async_await;
pub mod battery;
pub mod competition;
pub mod controller;
pub mod distance;
pub mod imu;
pub mod io;
pub mod logging;
pub mod machine;
pub mod macros;
pub mod motor;
pub mod optical;
pub mod peripherals;
pub mod prelude;
pub mod robot;
pub mod rotation;
pub mod rtos;
pub mod serial;
pub mod smart_port;
pub mod vexlink;

#[doc(hidden)]
pub use spin::once;

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    crate::io::eprintln!(
        "panic occurred on task {}: {:#?}",
        rtos::Task::current().name(),
        panic_info
    );
    crate::rtos::Task::delay(Duration::from_secs(1));

    unsafe {
        libc::exit(1);
    }
}
