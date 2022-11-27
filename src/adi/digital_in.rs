//! # ADI Digital Input API.

use crate::bindings;
use crate::error::{get_errno, Error};

/// A struct which represents a V5 ADI port configured as an ADI digital input.
#[derive(Debug)]
pub struct AdiDigitalInput {
    port: u8,
    expander_port: u8,
}

impl AdiDigitalInput {
    /// Initializes an ADI digital input on an ADI port.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same ADI digital input. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8, expander_port: u8) -> Result<Self, AdiDigitalInputError> {
        match bindings::ext_adi_port_set_config(
            expander_port,
            port,
            bindings::adi_port_config_e_E_ADI_DIGITAL_IN,
        ) {
            bindings::PROS_ERR_ => Err(AdiDigitalInputError::from_errno()),
            _ => Ok(Self {
                port,
                expander_port,
            }),
        }
    }

    /// Gets the digital value (true or false) of the input.
    pub fn read(&self) -> Result<bool, AdiDigitalInputError> {
        match unsafe { bindings::ext_adi_digital_read(self.expander_port, self.port) } {
            bindings::PROS_ERR_ => Err(AdiDigitalInputError::from_errno()),
            0 => Ok(false),
            _ => Ok(true),
        }
    }
}

/// Represents possible errors for ADI digital input operations.
#[derive(Debug)]
pub enum AdiDigitalInputError {
    /// Port is out of range (1-8).
    PortsOutOfRange,
    /// Port cannot be configured as an ADI digital input.
    PortsNotDigitalInput,
    /// Unknown error.
    Unknown(i32),
}

impl AdiDigitalInputError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::ENXIO => Self::PortsOutOfRange,
            libc::EADDRINUSE => Self::PortsNotDigitalInput,
            x => Self::Unknown(x),
        }
    }
}

impl From<AdiDigitalInputError> for Error {
    fn from(err: AdiDigitalInputError) -> Self {
        match err {
            AdiDigitalInputError::PortsOutOfRange => Error::Custom("port is out of range".into()),
            AdiDigitalInputError::PortsNotDigitalInput => {
                Error::Custom("port is not an ADI digital input".into())
            }
            AdiDigitalInputError::Unknown(n) => Error::System(n),
        }
    }
}
