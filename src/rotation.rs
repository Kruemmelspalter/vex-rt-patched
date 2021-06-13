//! # Rotation Sensor API.

use crate::{
    bindings,
    error::{get_errno, Error},
};
use uom::si::angle::degree;
use uom::si::angular_velocity::degree_per_second;
use uom::si::f64::{Angle, AngularVelocity};

/// A struct which represents a V5 smart port configured as a rotation sensor.
#[derive(Debug)]
pub struct RotationSensor {
    port: u8,
}

impl RotationSensor {
    /// Constructs a new rotation sensor.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same rotation sensor. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8, reversed: bool) -> Result<Self, RotationSensorError> {
        let mut sensor = Self { port };

        sensor.set_reversed(reversed)?;

        Ok(sensor)
    }

    /// Reset the current absolute position to be the same as the Rotation
    /// Sensor angle.
    pub fn reset(&mut self) -> Result<(), RotationSensorError> {
        match unsafe { bindings::rotation_reset(self.port) } {
            bindings::PROS_ERR_ => Err(RotationSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Set the Rotation sensor to a desired rotation value in centidegrees.
    pub fn set_position(&mut self, rotation: Angle) -> Result<(), RotationSensorError> {
        match unsafe {
            bindings::rotation_set_position(self.port, (rotation.get::<degree>() * 100f64) as u32)
        } {
            bindings::PROS_ERR_ => Err(RotationSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Reset the current absolute position to be the same as the Rotation
    /// Sensor angle.
    pub fn reset_position(&mut self) -> Result<(), RotationSensorError> {
        match unsafe { bindings::rotation_reset_position(self.port) } {
            bindings::PROS_ERR_ => Err(RotationSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Get the Rotation Sensor’s current position in centidegrees
    pub fn get_position(&self) -> Result<Angle, RotationSensorError> {
        match unsafe { bindings::rotation_get_position(self.port) } {
            x if x == bindings::PROS_ERR_ => Err(RotationSensorError::from_errno()),
            x => Ok(Angle::new::<degree>(x as f64 / 100f64)),
        }
    }

    /// Get the Rotation Sensor’s current velocity in centidegrees per second
    pub fn get_velocity(&self) -> Result<AngularVelocity, RotationSensorError> {
        match unsafe { bindings::rotation_get_velocity(self.port) } {
            x if x == bindings::PROS_ERR_ => Err(RotationSensorError::from_errno()),
            x => Ok(AngularVelocity::new::<degree_per_second>(x as f64 / 100f64)),
        }
    }

    /// Get the Rotation Sensor’s current angle in centidegrees (0-36000)
    pub fn get_angle(&self) -> Result<Angle, RotationSensorError> {
        match unsafe { bindings::rotation_get_angle(self.port) } {
            x if x == bindings::PROS_ERR_ => Err(RotationSensorError::from_errno()),
            x => Ok(Angle::new::<degree>(x as f64 / 100f64)),
        }
    }

    /// Set the rotation direction of the sensor
    pub fn set_reversed(&mut self, reverse: bool) -> Result<(), RotationSensorError> {
        match unsafe { bindings::rotation_set_reversed(self.port, reverse) } {
            bindings::PROS_ERR_ => Err(RotationSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Reverses the rotational sensor’s direction
    pub fn reverse(&mut self) -> Result<(), RotationSensorError> {
        match unsafe { bindings::rotation_reverse(self.port) } {
            bindings::PROS_ERR_ => Err(RotationSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Get the Rotation Sensor’s reversed flag
    pub fn get_reversed(&self) -> Result<bool, RotationSensorError> {
        match unsafe { bindings::rotation_get_reversed(self.port) } {
            x if x == bindings::PROS_ERR_ => Err(RotationSensorError::from_errno()),
            x => Ok(x != 0),
        }
    }
}

/// Represents possible errors for distance sensor operations.
#[derive(Debug)]
pub enum RotationSensorError {
    /// Port is out of range (1-21).
    PortOutOfRange,
    /// Port cannot be configured as a distance sensor.
    PortNotDistanceSensor,
    /// Unknown error.
    Unknown(i32),
}

impl RotationSensorError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::ENXIO => Self::PortOutOfRange,
            libc::ENODEV => Self::PortNotDistanceSensor,
            x => Self::Unknown(x),
        }
    }
}

impl From<RotationSensorError> for Error {
    fn from(err: RotationSensorError) -> Self {
        match err {
            RotationSensorError::PortOutOfRange => Error::Custom("port out of range".into()),
            RotationSensorError::PortNotDistanceSensor => {
                Error::Custom("port not a rotation sensor".into())
            }
            RotationSensorError::Unknown(n) => Error::System(n),
        }
    }
}
