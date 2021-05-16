//! # ADI Gyro API.

use crate::bindings;
use crate::error::{get_errno, Error};
use alloc::string::ToString;

/// A struct which represents a V5 ADI port configured to be an ADI gyro.
#[derive(Debug)]
pub struct AdiGyro {
    port: bindings::ext_adi_gyro_t,
}
impl AdiGyro {
    /// Initializes a gyroscope on the given port.
    /// If the given port has not previously been configured as a gyro, then
    /// this function starts a 1300 ms calibration period.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same ADI gyro. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(
        adi_port: u8,
        multiplier: f64,
        extender_port: u8,
    ) -> Result<Self, AdiGyroError> {
        match bindings::ext_adi_gyro_init(extender_port, adi_port, multiplier) {
            bindings::PROS_ERR_ => Err(AdiGyroError::from_errno()),
            x => Ok(Self { port: x }),
        }
    }

    /// Resets the gyroscope value to zero.
    pub fn reset(&mut self) -> Result<(), AdiGyroError> {
        match unsafe { bindings::ext_adi_gyro_reset(self.port) } {
            bindings::PROS_ERR_ => Err(AdiGyroError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Gets the current gyro angle in tenths of a degree.
    /// Unless a multiplier is applied to the gyro, the return value will be a
    /// whole number representing the number of degrees of rotation times 10.
    /// There are 360 degrees in a circle, thus the gyro will return 3600 for
    /// one whole rotation.
    pub fn get(&self) -> Result<f64, AdiGyroError> {
        let out = unsafe { bindings::ext_adi_gyro_get(self.port) };
        if out == bindings::PROS_ERR_F_ {
            Err(AdiGyroError::from_errno())
        } else {
            Ok(out)
        }
    }
}
impl Drop for AdiGyro {
    fn drop(&mut self) {
        if let bindings::PROS_ERR_ = unsafe { bindings::ext_adi_gyro_shutdown(self.port) } {
            panic!(
                "failed to shutdown ADI gyro: {:?}",
                AdiGyroError::from_errno()
            );
        }
    }
}

/// Represents possible errors for ADI gyro operations.
#[derive(Debug)]
pub enum AdiGyroError {
    /// Port is out of range (1-8).
    PortOutOfRange,
    /// Port cannot be configured as an ADI encoder.
    PortNotAdiEncoder,
    /// Unknown error.
    Unknown(i32),
}
impl AdiGyroError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::ENXIO => Self::PortOutOfRange,
            libc::EADDRINUSE => Self::PortNotAdiEncoder,
            x => Self::Unknown(x),
        }
    }
}
impl From<AdiGyroError> for Error {
    fn from(err: AdiGyroError) -> Self {
        match err {
            AdiGyroError::PortOutOfRange => Error::Custom("port out of range".to_string()),
            AdiGyroError::PortNotAdiEncoder => Error::Custom("port not an adi gyro".to_string()),
            AdiGyroError::Unknown(n) => Error::System(n),
        }
    }
}
