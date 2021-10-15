//! # ADI Motor API.

use crate::bindings;
use crate::error::{get_errno, Error};

/// A struct which represents a V5 ADI port configured as an ADI motor.
pub struct AdiMotor {
    port: u8,
    expander_port: u8,
}

impl AdiMotor {
    /// Initializes an ADI motor reader on one ADI ports.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same ADI motor reader. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8, expander_port: u8) -> Result<Self, AdiMotorError> {
        match bindings::ext_adi_port_set_config(
            expander_port,
            port,
            bindings::adi_port_config_e_E_ADI_LEGACY_PWM,
        ) {
            bindings::PROS_ERR_ => Err(AdiMotorError::from_errno()),
            _ => Ok(Self {
                port,
                expander_port,
            }),
        }
    }

    /// Returns the last set speed of the motor on the given port.
    pub fn read(&self) -> Result<i32, AdiMotorError> {
        match unsafe { bindings::ext_adi_motor_get(self.expander_port, self.port) } {
            bindings::PROS_ERR_ => Err(AdiMotorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Sets the speed of the motor on the given port, as a signed 8-bit value
    ///
    /// Returns: 1 if the operation was successful, PROS_ERR
    /// otherwise
    pub fn write(&self, speed: i8) -> Result<(), AdiMotorError> {
        match unsafe { bindings::ext_adi_motor_set(self.expander_port, self.port, speed) } {
            bindings::PROS_ERR_ => Err(AdiMotorError::from_errno()),
            _ => Ok(()),
        }
    }
}

/// Represents possible errors for ADI motor operations.
#[derive(Debug)]
pub enum AdiMotorError {
    /// Ports are out of range (1-8).
    PortsOutOfRange,
    /// Ports cannot be configured as an ADI Motor.
    PortsNotMotor,
    /// Unknown error.
    Unknown(i32),
}

impl AdiMotorError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::ENXIO => Self::PortsOutOfRange,
            libc::EADDRINUSE => Self::PortsNotMotor,
            x => Self::Unknown(x),
        }
    }
}

impl From<AdiMotorError> for Error {
    fn from(err: AdiMotorError) -> Self {
        match err {
            AdiMotorError::PortsOutOfRange => Error::Custom("ports out of range".into()),
            AdiMotorError::PortsNotMotor => Error::Custom("ports not an adi motor".into()),
            AdiMotorError::Unknown(n) => Error::System(n),
        }
    }
}
