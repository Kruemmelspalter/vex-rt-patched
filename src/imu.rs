//! # Inertial Sensor API.

use crate::{
    bindings,
    error::{get_errno, Error},
};
use alloc::format;
use uom::si::{
    acceleration::meter_per_second_squared,
    angle::degree,
    angular_velocity::degree_per_second,
    f64::{Acceleration, Angle, AngularVelocity},
};

/// A struct which represents a V5 smart port configured as a inertial sensor.
pub struct InertialSensor {
    port: u8,
}

impl InertialSensor {
    /// Constructs a new inertial sensor.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same inertial sensor. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8) -> InertialSensor {
        InertialSensor { port }
    }

    /// Calibrate IMU.
    ///
    /// This calls the reset function from PROS.
    /// This takes approximately 2 seconds, and is a non-blocking operation.
    pub fn calibrate(&mut self) -> Result<(), InertialSensorError> {
        match unsafe { bindings::imu_reset(self.port) } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Get the total angle that the Inertial Sensor has spun about the z-axis.
    ///
    /// This value is theoretically unbounded. Clockwise rotations are
    /// represented with positive values, while counterclockwise rotations are
    /// represented with negative ones.
    pub fn get_rotation(&self) -> Result<Angle, InertialSensorError> {
        match unsafe { bindings::imu_get_rotation(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(InertialSensorError::from_errno()),
            x => Ok(Angle::new::<degree>(x)),
        }
    }

    /// Get the Inertial Sensor’s heading relative to the initial direction of
    /// its x-axis.
    ///
    /// This value is bounded by [0,360) in degrees. Clockwise rotations are
    /// represented with positive values, while counterclockwise rotations are
    /// represented with negative ones.
    pub fn get_heading(&self) -> Result<Angle, InertialSensorError> {
        match unsafe { bindings::imu_get_heading(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(InertialSensorError::from_errno()),
            x => Ok(Angle::new::<degree>(x)),
        }
    }

    /// Get a quaternion representing the Inertial Sensor’s orientation.
    pub fn get_quaternion(&self) -> Result<InertialSensorQuaternion, InertialSensorError> {
        match unsafe { bindings::imu_get_quaternion(self.port) } {
            x if x.x == bindings::PROS_ERR_F_ => Err(InertialSensorError::from_errno()),
            x => Ok(InertialSensorQuaternion {
                x: x.x,
                y: x.y,
                z: x.z,
                w: x.w,
            }),
        }
    }

    /// Get the Euler angles representing the Inertial Sensor’s orientation.
    pub fn get_euler(&self) -> Result<InertialSensorEuler, InertialSensorError> {
        match unsafe { bindings::imu_get_euler(self.port) } {
            x if x.pitch == bindings::PROS_ERR_F_ => Err(InertialSensorError::from_errno()),
            x => Ok(InertialSensorEuler {
                pitch: Angle::new::<degree>(x.pitch),
                roll: Angle::new::<degree>(x.roll),
                yaw: Angle::new::<degree>(x.yaw),
            }),
        }
    }

    /// Get the Inertial Sensor’s pitch angle bounded by (-180,180) degrees.
    pub fn get_pitch(&self) -> Result<Angle, InertialSensorError> {
        match unsafe { bindings::imu_get_pitch(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(InertialSensorError::from_errno()),
            x => Ok(Angle::new::<degree>(x)),
        }
    }

    /// Get the Inertial Sensor’s roll angle bounded by (-180,180) degrees.
    pub fn get_roll(&self) -> Result<Angle, InertialSensorError> {
        match unsafe { bindings::imu_get_roll(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(InertialSensorError::from_errno()),
            x => Ok(Angle::new::<degree>(x)),
        }
    }

    /// Get the Inertial Sensor’s yaw angle bounded by (-180,180) degrees.
    pub fn get_yaw(&self) -> Result<Angle, InertialSensorError> {
        match unsafe { bindings::imu_get_yaw(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(InertialSensorError::from_errno()),
            x => Ok(Angle::new::<degree>(x)),
        }
    }

    /// Get the Inertial Sensor’s raw gyroscope values.
    pub fn get_gyro_rate(&self) -> Result<InertialSensorRawRate, InertialSensorError> {
        match unsafe { bindings::imu_get_gyro_rate(self.port) } {
            x if x.x == bindings::PROS_ERR_F_ => Err(InertialSensorError::from_errno()),
            x => Ok(InertialSensorRawRate {
                x: AngularVelocity::new::<degree_per_second>(x.x),
                y: AngularVelocity::new::<degree_per_second>(x.y),
                z: AngularVelocity::new::<degree_per_second>(x.z),
            }),
        }
    }

    /// Get the Inertial Sensor’s raw gyroscope values.
    pub fn get_accel(&self) -> Result<InertialSensorRawAccel, InertialSensorError> {
        match unsafe { bindings::imu_get_accel(self.port) } {
            x if x.x == bindings::PROS_ERR_F_ => Err(InertialSensorError::from_errno()),
            x => Ok(InertialSensorRawAccel {
                x: Acceleration::new::<meter_per_second_squared>(x.x),
                y: Acceleration::new::<meter_per_second_squared>(x.y),
                z: Acceleration::new::<meter_per_second_squared>(x.z),
            }),
        }
    }

    /// Get the Inertial Sensor’s status.
    pub fn get_status(&self) -> Result<InertialSensorStatus, InertialSensorError> {
        let status = unsafe { bindings::imu_get_status(self.port) };

        if status == bindings::imu_status_e_E_IMU_STATUS_ERROR {
            Err(InertialSensorError::from_errno())
        } else {
            Ok(InertialSensorStatus(status))
        }
    }

    /// Resets the current reading of the Inertial Sensor’s rotation to zero.
    pub fn reset_heading(&mut self) -> Result<(), InertialSensorError> {
        match unsafe { bindings::imu_tare_heading(self.port) } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Resets the current reading of the Inertial Sensor’s rotation to zero.
    pub fn reset_rotation(&mut self) -> Result<(), InertialSensorError> {
        match unsafe { bindings::imu_tare_rotation(self.port) } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Resets the current reading of the Inertial Sensor’s pitch to zero.
    pub fn reset_pitch(&mut self) -> Result<(), InertialSensorError> {
        match unsafe { bindings::imu_tare_pitch(self.port) } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Resets the current reading of the Inertial Sensor’s roll to zero.
    pub fn reset_roll(&mut self) -> Result<(), InertialSensorError> {
        match unsafe { bindings::imu_tare_roll(self.port) } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Resets the current reading of the Inertial Sensor’s yaw to zero.
    pub fn reset_yaw(&mut self) -> Result<(), InertialSensorError> {
        match unsafe { bindings::imu_tare_yaw(self.port) } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Reset all 3 euler values of the Inertial Sensor to 0.
    pub fn reset_euler(&mut self) -> Result<(), InertialSensorError> {
        match unsafe { bindings::imu_tare_euler(self.port) } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Resets all 5 values of the Inertial Sensor to 0.
    pub fn reset(&mut self) -> Result<(), InertialSensorError> {
        match unsafe { bindings::imu_tare(self.port) } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the current reading of the Inertial Sensor’s euler values to target
    /// euler values. Will default to +/- 180 if target exceeds +/- 180.
    pub fn set_zero_euler(
        &mut self,
        euler: InertialSensorEuler,
    ) -> Result<(), InertialSensorError> {
        match unsafe {
            bindings::imu_set_euler(
                self.port,
                bindings::euler_s_t {
                    pitch: euler.pitch.get::<degree>(),
                    roll: euler.roll.get::<degree>(),
                    yaw: euler.yaw.get::<degree>(),
                },
            )
        } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the current reading of the Inertial Sensor’s rotation to target
    /// value.
    pub fn set_rotation(&mut self, rotation: Angle) -> Result<(), InertialSensorError> {
        match unsafe { bindings::imu_set_rotation(self.port, rotation.get::<degree>()) } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the current reading of the Inertial Sensor’s heading to target
    /// value Target will default to 360 degrees if above 360 degrees and
    /// default to 0 if below 0.
    pub fn set_heading(&mut self, heading: Angle) -> Result<(), InertialSensorError> {
        match unsafe { bindings::imu_set_heading(self.port, heading.get::<degree>()) } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the current reading of the Inertial Sensor’s pitch to target value
    /// Will default to +/- 180 degrees if target exceeds +/- 180 degrees.
    pub fn set_pitch(&mut self, pitch: Angle) -> Result<(), InertialSensorError> {
        match unsafe { bindings::imu_set_pitch(self.port, pitch.get::<degree>()) } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the current reading of the Inertial Sensor’s roll to target value
    /// Will default to +/- 180 degrees if target exceeds +/- 180 degrees.
    pub fn set_roll(&mut self, roll: Angle) -> Result<(), InertialSensorError> {
        match unsafe { bindings::imu_set_roll(self.port, roll.get::<degree>()) } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the current reading of the Inertial Sensor’s yaw to target value
    /// Will default to +/- 180 degrees if target exceeds +/- 180 degrees.
    pub fn set_yaw(&mut self, yaw: Angle) -> Result<(), InertialSensorError> {
        match unsafe { bindings::imu_set_yaw(self.port, yaw.get::<degree>()) } {
            bindings::PROS_ERR_ => Err(InertialSensorError::from_errno()),
            _ => Ok(()),
        }
    }
}

/// Represents possible errors for inertial sensor operations.
#[derive(Debug)]
pub enum InertialSensorError {
    /// Port is out of range (1-21).
    PortOutOfRange,
    /// Port cannot be configured as a inertial sensor.
    PortNotInertialSensor,
    /// The sensor is already calibrating.
    SensorAlreadyCalibrating,
    /// The sensor returned an unknown status code.
    UnknownStatusCode(u32),
    /// Unknown error.
    Unknown(i32),
}

impl InertialSensorError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::ENXIO => Self::PortOutOfRange,
            libc::ENODEV => Self::PortNotInertialSensor,
            libc::EAGAIN => Self::SensorAlreadyCalibrating,
            x => Self::Unknown(x),
        }
    }
}

impl From<InertialSensorError> for Error {
    fn from(err: InertialSensorError) -> Self {
        match err {
            InertialSensorError::PortOutOfRange => Error::Custom("port out of range".into()),
            InertialSensorError::PortNotInertialSensor => {
                Error::Custom("port not a inertial sensor".into())
            }
            InertialSensorError::SensorAlreadyCalibrating => {
                Error::Custom("sensor already calibrating".into())
            }
            InertialSensorError::UnknownStatusCode(n) => {
                Error::Custom(format!("sensor returned unknown status code {}", n))
            }
            InertialSensorError::Unknown(n) => Error::System(n),
        }
    }
}

/// Represents raw rate values returned from an inertial sensor.
#[derive(Copy, Clone, Debug)]
pub struct InertialSensorRawRate {
    /// The raw x value returned from the inertial sensor.
    pub x: AngularVelocity,
    /// The raw y value returned from the inertial sensor.
    pub y: AngularVelocity,
    /// The raw z value returned from the inertial sensor.
    pub z: AngularVelocity,
}

/// Represents raw acceleration values returned from an inertial sensor.
#[derive(Copy, Clone, Debug)]
pub struct InertialSensorRawAccel {
    /// The raw x value returned from the inertial sensor.
    pub x: Acceleration,
    /// The raw y value returned from the inertial sensor.
    pub y: Acceleration,
    /// The raw z value returned from the inertial sensor.
    pub z: Acceleration,
}

/// Represents a Quaternion returned from an inertial sensor.
pub struct InertialSensorQuaternion {
    /// The x value of the Quaternion.
    pub x: f64,
    /// The y value of the Quaternion.
    pub y: f64,
    /// The z value of the Quaternion.
    pub z: f64,
    /// The w value of the Quaternion.
    pub w: f64,
}

/// Represents the set of euler angles returned from an inertial sensor.
pub struct InertialSensorEuler {
    /// The counterclockwise rotation on the y axis.
    pub pitch: Angle,
    /// The counterclockwise rotation on the x axis.
    pub roll: Angle,
    /// The counterclockwise rotation on the z axis.
    pub yaw: Angle,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Indicates IMU status.
pub struct InertialSensorStatus(bindings::imu_status_e);
impl InertialSensorStatus {
    #[inline]
    /// Gets the raw status value.
    pub fn into_raw(self) -> bindings::imu_status_e {
        self.0
    }
    #[inline]
    /// Checks whether the status value indicates that the IMU is calibrating.
    pub fn is_calibrating(self) -> bool {
        self.0 & bindings::imu_status_e_E_IMU_STATUS_CALIBRATING != 0
    }
}
