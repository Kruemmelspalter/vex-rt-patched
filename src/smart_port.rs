//! SmartPort.

use crate::adi::AdiExpander;
use crate::imu::InertialSensor;
use crate::prelude::{RotationSensor, RotationSensorError};
use crate::{
    bindings,
    distance::DistanceSensor,
    motor::{EncoderUnits, Gearset, Motor},
    serial::Serial,
};
use core::convert::{TryFrom, TryInto};

/// A struct which represents an unconfigured smart port.
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

    #[inline]
    /// Checks the type of device currently connected to the port.
    pub fn plugged_type(&self) -> DeviceType {
        unsafe { smart_port_type(self.port) }
    }

    /// Converts a `SmartPort` into a [`Motor`](crate::motor::Motor).
    pub fn into_motor(self, gearset: Gearset, encoder_units: EncoderUnits, reverse: bool) -> Motor {
        unsafe { Motor::new(self.port, gearset, encoder_units, reverse) }
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

/// Represents the type of device plugged into a smart port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceType {
    /// No device.
    None,

    /// V5 Smart Motor.
    Motor,

    /// V5 Rotation Sensor
    Rotation,

    /// V5 Inertial Sensor
    Imu,

    /// V5 Distance Sensor.
    Distance,

    /// V5 Robot Radio.
    Radio,

    /// V5 Vision Sensor.
    Vision,

    /// V5 3-Wire Expander.
    Adi,

    /// V5 Optical Sensor.
    Optical,

    /// Generic serial mode.
    Serial,

    /// Undefined sensor type.
    Undefined,

    /// Unrecognized value from PROS/vexOS.
    Unknown(u32),
}

impl From<bindings::v5_device_e_t> for DeviceType {
    fn from(t: bindings::v5_device_e_t) -> Self {
        match t {
            bindings::v5_device_e_E_DEVICE_NONE => Self::None,
            bindings::v5_device_e_E_DEVICE_MOTOR => Self::Motor,
            bindings::v5_device_e_E_DEVICE_ROTATION => Self::Rotation,
            bindings::v5_device_e_E_DEVICE_IMU => Self::Imu,
            bindings::v5_device_e_E_DEVICE_DISTANCE => Self::Distance,
            bindings::v5_device_e_E_DEVICE_RADIO => Self::Radio,
            bindings::v5_device_e_E_DEVICE_VISION => Self::Vision,
            bindings::v5_device_e_E_DEVICE_ADI => Self::Adi,
            bindings::v5_device_e_E_DEVICE_OPTICAL => Self::Optical,
            bindings::v5_device_e_E_DEVICE_GENERIC => Self::Serial,
            bindings::v5_device_e_E_DEVICE_UNDEFINED => Self::Undefined,
            _ => Self::Unknown(t),
        }
    }
}

/// Checks the type of device currently connected to a smart port.
///
/// # Safety
/// This is unsafe because it may be unsequenced relative to operations on the
/// smart port. Prefer [`SmartPort::plugged_type()`] instead.
pub unsafe fn smart_port_type(port: u8) -> DeviceType {
    bindings::registry_get_plugged_type(port - 1).into()
}
