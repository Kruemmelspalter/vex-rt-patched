//! # ADI Analog API.

use crate::bindings;
use crate::error::{get_errno, Error};

/// A struct which represents a V5 ADI port configured as an ADI encoder.
pub struct AdiAnalog {
    port: u8,
    expander_port: u8,
}

impl AdiAnalog {
    /// Initializes an ADI analog reader on one ADI ports.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same ADI analog reader. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8, expander_port: u8) -> Result<Self, AdiAnalogError> {
        match bindings::ext_adi_port_set_config(
            expander_port,
            port,
            bindings::adi_port_config_e_E_ADI_ANALOG_IN,
        ) {
            bindings::PROS_ERR_ => Err(AdiAnalogError::from_errno()),
            _ => Ok(Self {
                port,
                expander_port,
            }),
        }
    }

    /// Calibrates the analog sensor on the specified channel.

    /// This method assumes that the true sensor value is not actively changing
    /// at this time and computes an average from approximately 500 samples, 1
    /// ms apart, for a 0.5 s period of calibration. The average value thus
    /// calculated is returned and stored for later calls to the
    /// ext_adi_analog_read_calibrated and ext_adi_analog_read_calibrated_HR
    /// functions. These functions will return the difference between this value
    /// and the current sensor value when called.

    /// Do not use this function when the sensor value might be unstable (gyro
    /// rotation, accelerometer movement).

    /// Returns: The average sensor value computed by this function.
    pub fn calibrate(&mut self) -> Result<i32, AdiAnalogError> {
        match unsafe { bindings::ext_adi_analog_calibrate(self.expander_port, self.port) } {
            bindings::PROS_ERR_ => Err(AdiAnalogError::from_errno()),
            x => Ok(x),
        }
    }

    /// Reads an analog input channel and returns the 12-bit value.

    /// The value returned is undefined if the analog pin has been switched to a
    /// different mode. The meaning of the returned value varies depending on
    /// the sensor attached.

    /// Returns: The analog sensor value, where a value
    /// of 0 reflects an input voltage of nearly 0 V and a value of 4095
    /// reflects an input voltage of nearly 5 V
    pub fn read(&self) -> Result<i32, AdiAnalogError> {
        match unsafe { bindings::ext_adi_analog_read(self.expander_port, self.port) } {
            bindings::PROS_ERR_ => Err(AdiAnalogError::from_errno()),
            x => Ok(x),
        }
    }

    /// Reads the calibrated value of an analog input channel.

    /// The [`AdiAnalog::read_calibrated()`](crate::adi::analog::AdiAnalog::
    /// read_calibrated()) function must be run first on that channel.
    /// This function is inappropriate for sensor values intended for
    /// integration, as round-off error can accumulate causing drift over time.
    /// Use [`AdiAnalog::read_calibrated_hr()`](crate::adi::analog::AdiAnalog::
    /// read_calibrated_hr()) instead.

    /// Returns: The difference of the sensor value from its calibrated default
    /// from -4095 to 4095.
    pub fn read_calibrated(&self) -> Result<i32, AdiAnalogError> {
        match unsafe { bindings::ext_adi_analog_read_calibrated(self.expander_port, self.port) } {
            bindings::PROS_ERR_ => Err(AdiAnalogError::from_errno()),
            x => Ok(x),
        }
    }

    /// Reads the calibrated value of an analog input channel 1-8 with enhanced
    /// precision.

    /// The [`AdiAnalog::read_calibrated()`](crate::adi::analog::AdiAnalog::
    /// read_calibrated()) function must be run first. This is intended for
    /// integrated sensor values such as gyros and accelerometers
    /// to reduce drift due to round-off, and should not be used on a sensor
    /// such as a line tracker or potentiometer.

    /// The value returned actually has 16 bits of “precision”, even though the
    /// ADC only reads 12 bits, so that errors induced by the average value
    /// being between two values come out in the wash when integrated over time.
    /// Think of the value as the true value times 16.

    /// Returns: The difference of the sensor value from its calibrated default
    /// from -16384 to 16384.
    pub fn read_calibrated_hr(&self) -> Result<i32, AdiAnalogError> {
        match unsafe { bindings::ext_adi_analog_read_calibrated_HR(self.expander_port, self.port) }
        {
            bindings::PROS_ERR_ => Err(AdiAnalogError::from_errno()),
            x => Ok(x),
        }
    }
}

/// Represents possible errors for ADI analog operations.
#[derive(Debug)]
pub enum AdiAnalogError {
    /// Ports are out of range (1-8).
    PortsOutOfRange,
    /// Ports cannot be configured as an ADI Analog input.
    PortsNotAnalogInput,
    /// Unknown error.
    Unknown(i32),
}

impl AdiAnalogError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::ENXIO => Self::PortsOutOfRange,
            libc::EADDRINUSE => Self::PortsNotAnalogInput,
            x => Self::Unknown(x),
        }
    }
}

impl From<AdiAnalogError> for Error {
    fn from(err: AdiAnalogError) -> Self {
        match err {
            AdiAnalogError::PortsOutOfRange => Error::Custom("ports out of range".into()),
            AdiAnalogError::PortsNotAnalogInput => {
                Error::Custom("ports not an adi analog input".into())
            }
            AdiAnalogError::Unknown(n) => Error::System(n),
        }
    }
}
