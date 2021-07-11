//! SmartPort.

use crate::{
    adi::AdiExpander,
    distance::DistanceSensor,
    error::Error,
    imu::InertialSensor,
    motor::{Gearset, Motor, MotorError},
    rotation::{RotationSensor, RotationSensorError},
    serial::Serial,
};
use core::convert::{TryFrom, TryInto};

/// A struct which represents an unconfigured smart port.
#[derive(Debug)]
pub struct SmartPort {
    port: u8,
}

impl SmartPort {
    /// Constructs a new smart port.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to a V5 smart port. You likely want to implement
    /// [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8) -> Self {
        assert!(
            (1..22).contains(&port),
            "Cannot construct a smart port on port {}",
            port
        );
        Self { port }
    }

    /// Converts a `SmartPort` into a [`Motor`](crate::motor::Motor).
    pub fn into_motor(self, gearset: Gearset, reverse: bool) -> Result<Motor, MotorError> {
        (self, gearset, reverse).try_into()
    }

    /// Converts a `SmartPort` into a [`Serial`].
    pub fn into_serial(self, baudrate: i32) -> Result<Serial, Error> {
        (self, baudrate).try_into()
    }

    /// Converts a `SmartPort` into an [`AdiExpander`](crate::adi::AdiExpander).
    pub fn into_expander(self) -> AdiExpander {
        self.into()
    }

    /// Converts a `SmartPort` into a
    /// [`DistanceSensor`](crate::distance::DistanceSensor).
    pub fn into_distance(self) -> DistanceSensor {
        self.into()
    }

    /// Converts a `SmartPort` into a
    /// [`InertialSensor`](crate::imu::InertialSensor).
    pub fn into_imu(self) -> InertialSensor {
        self.into()
    }

    /// Converts a `SmartPort` into a
    /// [`RotationSensor`](crate::rotation::RotationSensor).
    #[inline]
    pub fn into_rotation(self, reversed: bool) -> Result<RotationSensor, RotationSensorError> {
        (self, reversed).try_into()
    }
}

impl TryFrom<(SmartPort, Gearset, bool)> for Motor {
    type Error = MotorError;

    fn try_from((port, gearset, reverse): (SmartPort, Gearset, bool)) -> Result<Self, Self::Error> {
        unsafe { Self::new(port.port, gearset, reverse) }
    }
}

impl TryFrom<(SmartPort, i32)> for Serial {
    type Error = Error;

    fn try_from((port, baudrate): (SmartPort, i32)) -> Result<Self, Self::Error> {
        unsafe { Self::new(port.port, baudrate) }
    }
}

impl From<SmartPort> for AdiExpander {
    fn from(port: SmartPort) -> Self {
        unsafe { AdiExpander::new(port.port) }
    }
}

impl From<SmartPort> for DistanceSensor {
    fn from(port: SmartPort) -> Self {
        unsafe { DistanceSensor::new(port.port) }
    }
}
impl From<SmartPort> for InertialSensor {
    fn from(port: SmartPort) -> Self {
        unsafe { InertialSensor::new(port.port) }
    }
}

impl TryFrom<(SmartPort, bool)> for RotationSensor {
    type Error = RotationSensorError;

    #[inline]
    fn try_from((port, reversed): (SmartPort, bool)) -> Result<Self, Self::Error> {
        unsafe { RotationSensor::new(port.port, reversed) }
    }
}
