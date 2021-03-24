//! Common error model.

use alloc::{format, string::*};
use core::{
    fmt::{self, Debug, Display, Formatter},
    num::TryFromIntError,
};
use cstring_interop::from_cstring_raw;

use crate::bindings;

/// Represents a runtime error.
pub enum Error {
    /// Represents a runtime error which comes from the underlying platform
    /// (PROS, FreeRTOS, newlib, etc.). It wraps an `errno` value (i.e., system
    /// error code).
    System(i32),
    /// Represents a runtime error which comes from within Rust. It wraps an
    /// error string.
    Custom(String),
}

impl From<rcstring::Error> for Error {
    fn from(err: rcstring::Error) -> Self {
        Error::Custom(format!("{:?}", err))
    }
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        Error::Custom(format!("{:?}", err))
    }
}

impl<T> From<Error> for Result<T, Error> {
    #[inline]
    fn from(err: Error) -> Self {
        Err(err)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::System(n) => write!(f, "System({}) [{}]", n, unsafe {
                from_cstring_raw(libc::strerror(*n))
            }),
            Error::Custom(s) => write!(f, "Custom({:?})", s),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::System(n) => Display::fmt(unsafe { &from_cstring_raw(libc::strerror(*n)) }, f),
            Error::Custom(s) => Display::fmt(s, f),
        }
    }
}

/// Represents a type which has some sentinel values which represent errors.
///
/// Implementations are provided for `i32` and `f64` based on PROS's sentinel
/// error values, represented by `PROS_ERR` and `PROS_ERR_F` in C/C++.
pub trait SentinelError: Sized {
    /// Checks if the type is a valid (success value), giving an appropriate
    /// error otherwise.
    fn check(self) -> Result<Self, Error>;
}

impl SentinelError for i32 {
    fn check(self) -> Result<Self, Error> {
        if self == bindings::PROS_ERR_ {
            Err(from_errno())
        } else {
            Ok(self)
        }
    }
}

impl SentinelError for f64 {
    fn check(self) -> Result<Self, Error> {
        if self == bindings::PROS_ERR_F_ {
            Err(from_errno())
        } else {
            Ok(self)
        }
    }
}

impl<T> SentinelError for *mut T {
    fn check(self) -> Result<Self, Error> {
        if self.is_null() {
            Err(from_errno())
        } else {
            Ok(self)
        }
    }
}

// Need to manually declare until https://github.com/rust-lang/libc/issues/1995 is resolved.
extern "C" {
    fn __errno() -> *mut i32;
}

/// Gets the value of `errno` for the current task.
#[inline]
pub fn get_errno() -> libc::c_int {
    unsafe { *__errno() }
}

/// Generates an [`Error`] object from the value of `errno` for the current
/// task.
#[inline]
pub fn from_errno() -> Error {
    Error::System(get_errno())
}
