//! SmartPort.

use crate::adi::AdiExpander;
use crate::imu::InertialSensor;
use crate::prelude::{RotationSensor, RotationSensorError};
use crate::{
    distance::DistanceSensor,
    motor::{Gearset, Motor},
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
    pub fn into_motor(self, gearset: Gearset, reverse: bool) -> Motor {
        unsafe { Motor::new(self.port, gearset, reverse) }
    }

    /// Converts a `SmartPort` into a [`Serial`].
    pub fn into_serial(self, baudrate: i32) -> Serial {
        unsafe { Serial::new(self.port, baudrate) }
    }

    /// Converts a `SmartPort` into a [`AdiExpander`](crate::adi::AdiExpander).
    pub fn into_expander(self) -> AdiExpander {
        unsafe { AdiExpander::new(self.port) }
    }

    /// Converts a `SmartPort` into a
    /// [`DistanceSensor`](crate::distance::DistanceSensor).
    pub fn into_distance(self) -> DistanceSensor {
        unsafe { DistanceSensor::new(self.port) }
    }

    /// Converts a `SmartPort` into a
    /// [`InertialSensor`](crate::imu::InertialSensor).
    pub fn into_imu(self) -> InertialSensor {
        unsafe { InertialSensor::new(self.port) }
    }

    /// Converts a `SmartPort` into a
    /// [`RotationSensor`](crate::rotation::RotationSensor).
    #[inline]
    pub fn into_rotation(self, reversed: bool) -> Result<RotationSensor, RotationSensorError> {
        (self, reversed).try_into()
    }
}

impl TryFrom<(SmartPort, bool)> for RotationSensor {
    type Error = RotationSensorError;

    #[inline]
    fn try_from(port_reversed: (SmartPort, bool)) -> Result<Self, Self::Error> {
        unsafe { RotationSensor::new(port_reversed.0.port, port_reversed.1) }
    }
}
