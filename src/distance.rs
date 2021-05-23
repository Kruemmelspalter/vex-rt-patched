//! # Distance Sensor API.

use uom::si::{
    f64::{Length, Velocity},
    length::millimeter,
    velocity::millimeter_per_second,
};

use crate::{
    bindings,
    error::{get_errno, Error},
};

/// A struct which represents a V5 smart port configured as a distance sensor.
#[derive(Debug)]
pub struct DistanceSensor {
    port: u8,
}

impl DistanceSensor {
    /// Constructs a new distance sensor.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same distance sensor. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8) -> DistanceSensor {
        DistanceSensor { port }
    }

    /// Gets the currently measured distance from the sensor.
    pub fn get_distance(&self) -> Result<Length, DistanceSensorError> {
        match unsafe { bindings::distance_get(self.port) } {
            x if x == bindings::PROS_ERR_ => Err(DistanceSensorError::from_errno()),
            x => Ok(Length::new::<millimeter>(x as f64)),
        }
    }

    /// Gets the confidence in the distance reading.
    ///
    /// This is a value that has a range of 0 to 63. 63 means high confidence,
    /// lower values imply less confidence. Confidence is only available when
    /// distance is > 200mm (the value 10 is returned in this scenario).
    // TODO: figure out how to give this units.
    pub fn get_confidence(&self) -> Result<i32, DistanceSensorError> {
        match unsafe { bindings::distance_get_confidence(self.port) } {
            x if x == bindings::PROS_ERR_ => Err(DistanceSensorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the current estimated relative object size.
    ///
    /// This is a value that has a range of 0 to 400. A 18” x 30” grey card will
    /// return a value of approximately 75 in typical room lighting.
    // TODO: figure out how to give this units.
    pub fn get_object_size(&self) -> Result<i32, DistanceSensorError> {
        match unsafe { bindings::distance_get_object_size(self.port) } {
            x if x == bindings::PROS_ERR_ => Err(DistanceSensorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the object velocity.
    pub fn get_object_velocity(&self) -> Result<Velocity, DistanceSensorError> {
        match unsafe { bindings::distance_get_object_velocity(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(DistanceSensorError::from_errno()),
            x => Ok(Velocity::new::<millimeter_per_second>(x)),
        }
    }
}

/// Represents possible errors for distance sensor operations.
#[derive(Debug)]
pub enum DistanceSensorError {
    /// Port is out of range (1-21).
    PortOutOfRange,
    /// Port cannot be configured as a distance sensor.
    PortNotDistanceSensor,
    /// Unknown error.
    Unknown(i32),
}

impl DistanceSensorError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::ENXIO => Self::PortOutOfRange,
            libc::ENODEV => Self::PortNotDistanceSensor,
            x => Self::Unknown(x),
        }
    }
}

impl From<DistanceSensorError> for Error {
    fn from(err: DistanceSensorError) -> Self {
        match err {
            DistanceSensorError::PortOutOfRange => Error::Custom("port out of range".into()),
            DistanceSensorError::PortNotDistanceSensor => {
                Error::Custom("port not a distance sensor".into())
            }
            DistanceSensorError::Unknown(n) => Error::System(n),
        }
    }
}
