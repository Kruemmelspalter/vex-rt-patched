use crate::{
    bindings,
    error::{get_errno, Error},
};

/// A struct which represents a V5 ADI port configured as an ADI digital output.
pub struct AdiDigitalOutput {
    port: u8,
    expander_port: u8,
}

impl AdiDigitalOutput {
    /// Initializes an ADI digital output on an ADI port.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same ADI digital output. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8, expander_port: u8) -> Result<Self, AdiDigitalOutputError> {
        match bindings::ext_adi_port_set_config(
            expander_port,
            port,
            bindings::adi_port_config_e_E_ADI_DIGITAL_OUT,
        ) {
            bindings::PROS_ERR_ => Err(AdiDigitalOutputError::from_errno()),
            _ => Ok(Self {
                port,
                expander_port,
            }),
        }
    }

    /// Sets the digital value (true or false) of the output.
    pub fn write(&mut self, value: bool) -> Result<(), AdiDigitalOutputError> {
        match unsafe { bindings::ext_adi_digital_write(self.expander_port, self.port, value) } {
            bindings::PROS_ERR_ => Err(AdiDigitalOutputError::from_errno()),
            _ => Ok(()),
        }
    }
}

/// Represents possible errors for ADI digital output operations.
#[derive(Debug)]
pub enum AdiDigitalOutputError {
    /// Port is out of range (1-8).
    PortsOutOfRange,
    /// Port cannot be configured as an ADI digital output.
    PortsNotDigitalOutput,
    /// Unknown error.
    Unknown(i32),
}

impl AdiDigitalOutputError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::ENXIO => Self::PortsOutOfRange,
            libc::EADDRINUSE => Self::PortsNotDigitalOutput,
            x => Self::Unknown(x),
        }
    }
}

impl From<AdiDigitalOutputError> for Error {
    fn from(err: AdiDigitalOutputError) -> Self {
        match err {
            AdiDigitalOutputError::PortsOutOfRange => Error::Custom("port is out of range".into()),
            AdiDigitalOutputError::PortsNotDigitalOutput => {
                Error::Custom("port is not an adi digital output".into())
            }
            AdiDigitalOutputError::Unknown(n) => Error::System(n),
        }
    }
}
