use crate::{
    bindings,
    error::{get_errno, Error},
    rtos::DataSource,
};

#[repr(transparent)]
/// Represents a V5 ADI port pair configured as an ultrasonic sensor.
pub struct AdiUltrasonic {
    port: bindings::ext_adi_ultrasonic_t,
}

impl AdiUltrasonic {
    /// Initializes an ultrasonic sensor on two ADI ports.
    ///
    /// # Safety
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same ADI ultrasonic sensor. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(
        out_port: u8,
        in_port: u8,
        smart_port: u8,
    ) -> Result<Self, AdiUltrasonicError> {
        match bindings::ext_adi_ultrasonic_init(smart_port, out_port, in_port) {
            bindings::PROS_ERR_ => Err(AdiUltrasonicError::from_errno()),
            port => Ok(Self { port }),
        }
    }

    /// Gets the current value of the ultrasonic sensor.
    pub fn get(&self) -> Result<u32, AdiUltrasonicError> {
        match unsafe { bindings::ext_adi_ultrasonic_get(self.port) } {
            bindings::PROS_ERR_ => Err(AdiUltrasonicError::from_errno()),
            r if r < 0 => Err(AdiUltrasonicError::NoReading),
            r => Ok(r as u32),
        }
    }
}

impl DataSource for AdiUltrasonic {
    type Data = u32;

    type Error = AdiUltrasonicError;

    fn read(&self) -> Result<Self::Data, Self::Error> {
        self.get()
    }
}

impl Drop for AdiUltrasonic {
    fn drop(&mut self) {
        if unsafe { bindings::ext_adi_ultrasonic_shutdown(self.port) } == bindings::PROS_ERR_ {
            panic!(
                "failed to shutdown ADI ultrasonic: {:?}",
                AdiUltrasonicError::from_errno()
            )
        }
    }
}

/// Represents possible errors for ADI ultrasonic operations.
#[derive(Debug)]
pub enum AdiUltrasonicError {
    /// Ports are out of range (1-8).
    PortsOutOfRange,
    /// Ports cannot be configured as an ADI encoder.
    PortsNotAdiUltrasonic,
    /// Ports are from non matching extenders
    PortNonMatchingExtenders,
    /// Sensor did not hear an echo.
    NoReading,
    /// Unknown error.
    Unknown(i32),
}

impl AdiUltrasonicError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::ENXIO => Self::PortsOutOfRange,
            libc::EADDRINUSE => Self::PortsNotAdiUltrasonic,
            x => Self::Unknown(x),
        }
    }
}

impl From<AdiUltrasonicError> for Error {
    fn from(err: AdiUltrasonicError) -> Self {
        match err {
            AdiUltrasonicError::PortsOutOfRange => Error::Custom("ports out of range".into()),
            AdiUltrasonicError::PortsNotAdiUltrasonic => {
                Error::Custom("ports not an adi ultrasonic".into())
            }
            AdiUltrasonicError::PortNonMatchingExtenders => {
                Error::Custom("ports from non-matching extenders".into())
            }
            AdiUltrasonicError::NoReading => Error::Custom("sensor did not hear an echo".into()),
            AdiUltrasonicError::Unknown(n) => Error::System(n),
        }
    }
}
