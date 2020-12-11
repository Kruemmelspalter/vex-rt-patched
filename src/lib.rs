#![no_std]
#![feature(alloc_error_handler)]

use core::panic::PanicInfo;
use libc_print::libc_println;

mod alloc;
mod bindings;
mod motor;
mod peripherals;
mod smart_port;

pub use motor::*;
pub use peripherals::*;
pub use smart_port::*;

pub use vex_rt_macros::*;

pub use spin::*;

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        libc_println!("panic occurred!: {:?}", s);
    } else {
        libc_println!("panic occurred!");
    }

    loop {}
}
