//! # ADI Digital API.

use crate::bindings;
use crate::error::{get_errno, Error};

/// A struct which represents a V5 ADI port configured as an ADI digital input.
pub struct AdiDigitalInput {
    port: u8,
    expander_port: u8,
}

/// A struct which represents a V5 ADI port configured as an ADI digital output.
pub struct AdiDigitalOutput {
    port: u8,
    expander_port: u8,
}

impl AdiDigitalInput {
    /// Initializes an ADI digital reader on one ADI ports.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same ADI analog reader. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8, expander_port: u8) -> Result<Self, AdiDigitalError> {
        match bindings::ext_adi_port_set_config(
            expander_port,
            port,
            bindings::adi_port_config_e_E_ADI_DIGITAL_IN,
        ) {
            bindings::PROS_ERR_ => Err(AdiDigitalError::from_errno()),
            _ => Ok(Self {
                port,
                expander_port,
            }),
        }
    }

    /// Gets the digital value (1 or 0) of a pin configured as a digital input.
    ///
    /// If the pin is configured as some other mode, the digital value which
    /// reflects the current state of the pin is returned, which may or may
    /// not differ from the currently set value. The return value is
    /// undefined for pins configured as Analog inputs.
    ///
    /// Returns: True if the pin is HIGH, or false if it is LOW.
    pub fn read(&self) -> Result<i32, AdiDigitalError> {
        match unsafe { bindings::ext_adi_digital_read(self.expander_port, self.port) } {
            bindings::PROS_ERR_ => Err(AdiDigitalError::from_errno()),
            x => Ok(x),
        }
    }
}

impl AdiDigitalOutput {
    /// Initializes an ADI digital reader on one ADI ports.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same ADI analog reader. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8, expander_port: u8) -> Result<Self, AdiDigitalError> {
        match bindings::ext_adi_port_set_config(
            expander_port,
            port,
            bindings::adi_port_config_e_E_ADI_DIGITAL_OUT,
        ) {
            bindings::PROS_ERR_ => Err(AdiDigitalError::from_errno()),
            _ => Ok(Self {
                port,
                expander_port,
            }),
        }
    }

    /// Sets the digital value (1 or 0) of a pin configured as a digital output.
    ///
    /// Returns: 1 if the operation was successful, PROS_ERR otherwise.
    pub fn write(&self, value: bool) -> Result<(), AdiDigitalError> {
        match unsafe { bindings::ext_adi_digital_write(self.expander_port, self.port, value) } {
            bindings::PROS_ERR_ => Err(AdiDigitalError::from_errno()),
            _ => Ok(()),
        }
    }
}

/// Represents possible errors for ADI digital operations.
#[derive(Debug)]
pub enum AdiDigitalError {
    /// Ports are out of range (1-8).
    PortsOutOfRange,
    /// Ports cannot be configured as an ADI Digital input.
    PortsNotDigitalInput,
    /// Unknown error.
    Unknown(i32),
}

impl AdiDigitalError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::ENXIO => Self::PortsOutOfRange,
            libc::EADDRINUSE => Self::PortsNotDigitalInput,
            x => Self::Unknown(x),
        }
    }
}

impl From<AdiDigitalError> for Error {
    fn from(err: AdiDigitalError) -> Self {
        match err {
            AdiDigitalError::PortsOutOfRange => Error::Custom("ports out of range".into()),
            AdiDigitalError::PortsNotDigitalInput => {
                Error::Custom("ports not an adi digital input".into())
            }
            AdiDigitalError::Unknown(n) => Error::System(n),
        }
    }
}
