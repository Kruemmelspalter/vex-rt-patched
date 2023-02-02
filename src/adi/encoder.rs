use crate::{
    bindings,
    error::{get_errno, Error},
    rtos::DataSource,
};

#[repr(transparent)]
/// A struct which represents a V5 ADI port configured as an ADI encoder.
pub struct AdiEncoder {
    port: bindings::ext_adi_encoder_t,
}

impl AdiEncoder {
    /// Initializes and enables a quadrature encoder on two ADI ports.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same ADI encoder. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(
        top_port: u8,
        bottom_port: u8,
        reverse: bool,
        extender_port: u8,
    ) -> Result<AdiEncoder, AdiEncoderError> {
        match bindings::ext_adi_encoder_init(extender_port, top_port, bottom_port, reverse) {
            bindings::PROS_ERR_ => Err(AdiEncoderError::from_errno()),
            x => Ok(AdiEncoder { port: x }),
        }
    }

    /// Resets the encoder to zero.
    /// It is safe to use this method while an encoder is enabled. It is not
    /// necessary to call this method before stopping or starting an encoder.
    pub fn reset(&mut self) -> Result<(), AdiEncoderError> {
        match unsafe { bindings::ext_adi_encoder_reset(self.port) } {
            bindings::PROS_ERR_ => Err(AdiEncoderError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Gets the number of ticks recorded by the encoder.
    /// There are 360 ticks in one revolution.
    pub fn get(&self) -> Result<i32, AdiEncoderError> {
        match unsafe { bindings::ext_adi_encoder_get(self.port) } {
            bindings::PROS_ERR_ => Err(AdiEncoderError::from_errno()),
            x => Ok(x),
        }
    }
}

impl DataSource for AdiEncoder {
    type Data = i32;

    type Error = AdiEncoderError;

    fn read(&self) -> Result<Self::Data, Self::Error> {
        self.get()
    }
}

impl Drop for AdiEncoder {
    fn drop(&mut self) {
        if let bindings::PROS_ERR_ = unsafe { bindings::ext_adi_encoder_shutdown(self.port) } {
            panic!(
                "failed to shutdown ADI encoder: {:?}",
                AdiEncoderError::from_errno()
            );
        }
    }
}

/// Represents possible errors for ADI encoder operations.
#[derive(Debug)]
pub enum AdiEncoderError {
    /// Ports are out of range (1-8).
    PortsOutOfRange,
    /// Ports cannot be configured as an ADI encoder.
    PortsNotAdiEncoder,
    /// Ports are from non matching extenders.
    PortNonMatchingExtenders,
    /// Unknown error.
    Unknown(i32),
}

impl AdiEncoderError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::ENXIO => Self::PortsOutOfRange,
            libc::ENODEV => Self::PortsNotAdiEncoder,
            x => Self::Unknown(x),
        }
    }
}

impl From<AdiEncoderError> for Error {
    fn from(err: AdiEncoderError) -> Self {
        match err {
            AdiEncoderError::PortsOutOfRange => Error::Custom("ports out of range".into()),
            AdiEncoderError::PortsNotAdiEncoder => Error::Custom("ports not an adi encoder".into()),
            AdiEncoderError::PortNonMatchingExtenders => {
                Error::Custom("ports from non-matching extenders".into())
            }
            AdiEncoderError::Unknown(n) => Error::System(n),
        }
    }
}
