//! # Distance Sensor API.

use crate::{
    bindings,
    error::{get_errno, Error},
    rtos::DataSource,
};

/// A struct which represents a V5 smart port configured as a distance sensor.
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

    /// Gets the currently measured distance from the sensor in millimetres.
    pub fn get_distance(&self) -> Result<i32, DistanceSensorError> {
        match unsafe { bindings::distance_get(self.port) } {
            x if x == bindings::PROS_ERR_ => Err(DistanceSensorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the confidence in the distance reading.
    ///
    /// This is a value that has a range of 0 to 63. 63 means high confidence,
    /// lower values imply less confidence. Confidence is only available when
    /// distance is > 200mm (the value 10 is returned in this scenario).
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
    pub fn get_object_size(&self) -> Result<i32, DistanceSensorError> {
        match unsafe { bindings::distance_get_object_size(self.port) } {
            x if x == bindings::PROS_ERR_ => Err(DistanceSensorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the object velocity in metres per second.
    pub fn get_object_velocity(&self) -> Result<f64, DistanceSensorError> {
        match unsafe { bindings::distance_get_object_velocity(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(DistanceSensorError::from_errno()),
            x => Ok(x),
        }
    }
}

impl DataSource for DistanceSensor {
    type Data = DistanceData;

    type Error = DistanceSensorError;

    fn read(&self) -> Result<Self::Data, Self::Error> {
        Ok(DistanceData {
            confidence: self.get_confidence()?,
            size: self.get_object_size()?,
            velocity: self.get_object_velocity()?,
        })
    }
}

/// Represents the data that can be read from a distance sensor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DistanceData {
    /// The confidence; see [`DistanceSensor::get_confidence()`] for details.
    pub confidence: i32,
    /// The object size; see [`DistanceSensor::get_object_size()`] for details.
    pub size: i32,
    /// The object velocity in meters per second.
    pub velocity: f64,
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
