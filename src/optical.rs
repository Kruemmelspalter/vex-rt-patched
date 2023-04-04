//! # Optical sensor API.

use core::convert::{TryFrom, TryInto};
use crate::{
    bindings,
    error::{get_errno, Error},
};

use qunit::time::{Time, TimeExt};

pub struct DetectGestures;
pub struct IgnoreGestures;

pub struct OpticalSensor<GestureDetection> {
    port: u8,
    gesture_detection: GestureDetection,
}

impl OpticalSensor<IgnoreGestures> {
    /// Creates a new optical sensor.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same optical sensor. You likely want to implement
    /// [`Robot::new`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8) -> Result<Self, OpticalSensorError> {
        match port {
            1..=21 => Ok(OpticalSensor{
                port,
                gesture_detection: IgnoreGestures
            }),
            _ => Err(OpticalSensorError::PortOutOfRange)
        }
    }
}

impl<GestureDetection> OpticalSensor<GestureDetection> {
    /// Get the Optical Sensor’s current brightness as a value in the range of 0.0 to 1.0.
    pub fn get_brightness(&self) -> Result<f64, OpticalSensorError> {
        match unsafe { bindings::optical_get_brightness(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(OpticalSensorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Get the Optical Sensor’s current hue as a value in the range of 0.0 to 359.999.
    pub fn get_hue(&self) -> Result<f64, OpticalSensorError> {
        match unsafe { bindings::optical_get_hue(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(OpticalSensorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Get the Optical Sensor's current pwm setting for the led.
    ///
    /// Returns a value between 0 and 100, inclusive.
    pub fn get_led_pwm(&self) -> Result<i32, OpticalSensorError> {
        match unsafe { bindings::optical_get_led_pwm(self.port) } {
            x => Ok(x),
            bindings::PROS_ERR_ => Err(OpticalSensorError::from_errno().into()),
        }
    }

    /// Get the Optical Sensor’s current proximity as a value in the range of 0 to 255.
    pub fn get_proximity(&self) -> Result<i32, OpticalSensorError> {
        match unsafe { bindings::optical_get_proximity(self.port) } {
            bindings::PROS_ERR_ => Err(OpticalSensorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Get the Optical Sensor’s raw un-processed RGBC data.
    pub fn get_raw(&self) -> Result<OpticalRaw, OpticalSensorError> {
        match unsafe { bindings::optical_get_raw(self.port) } {
            data if data.clear == bindings::PROS_ERR_U_ => Err(OpticalSensorError::from_errno()),
            data => Ok(OpticalRaw {
                clear: data.clear,
                red: data.red,
                green: data.red,
                blue: data.blue,
            }),
        }
    }

    /// Get the Optical Sensor’s RGB data.
    pub fn get_rgb(&self) -> Result<OpticalRGB, OpticalSensorError> {
        match unsafe { bindings::optical_get_rgb(self.port) } {
            data if data.brightness == bindings::PROS_ERR_F_ => Err(OpticalSensorError::from_errno()),
            data => Ok(OpticalRGB {
                red: data.red,
                green: data.green,
                blue: data.blue,
                brightness: data.brightness,
            }),
        }
    }

    /// Get the Optical Sensor’s current saturation as a value in the range of 0.0 to 1.0.
    pub fn get_saturation(&self) -> Result<f64, OpticalSensorError> {
        match unsafe { bindings::optical_get_saturation(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(OpticalSensorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Set the pwm value for the Optical Sensor's.
    ///
    /// Takes a value between 0 and 100, inclusive.
    pub fn set_led_pwm(&mut self, v: u8) -> Result<(), OpticalSensorError> {
        if 100 < v { return Err(OpticalSensorError::InvalidValue) }
        match unsafe { bindings::optical_set_led_pwm(self.port, v) } {
            1 => Ok(()),
            _ => Err(OpticalSensorError::from_errno()),
        }
    }

    pub fn get_integration_time(&self) -> Result<Time, OpticalSensorError> {
        match unsafe { bindings::optical_get_integration_time(self.port) } {
            bindings::PROS_ERR_F_ => Err(OpticalSensorError::from_errno()),
            x => Ok((x as f64).ms()),
        }
    }
    
    pub fn set_integration_time(&mut self, time: Time) -> Result<(), OpticalSensorError> {
        if time < 3.0.ms() || time > 712.0.ms() { return Err(OpticalSensorError::InvalidValue) }
        match unsafe { bindings::optical_set_integration_time(self.port, time.to_ms()) } {
            1 => Ok(()),
            _ => Err(OpticalSensorError::from_errno()),
        }
    }
}

impl OpticalSensor<DetectGestures> {
//impl OpticalSensor<GestureDetection::Enabled> {

    /// Get the Optical Sensor’s most recent gesture data
    pub fn get_gesture(&self) -> Result<OpticalDirection, OpticalSensorError> {
        match unsafe { bindings::optical_get_gesture(self.port) } {
            bindings::optical_direction_e_ERROR => Err(OpticalSensorError::from_errno()),
            bindings::optical_direction_e_UP => Ok(OpticalDirection::Up),
            bindings::optical_direction_e_DOWN => Ok(OpticalDirection::Down),
            bindings::optical_direction_e_RIGHT => Ok(OpticalDirection::Right),
            bindings::optical_direction_e_LEFT => Ok(OpticalDirection::Left),
            bindings::optical_direction_e_NO_GESTURE => Ok(OpticalDirection::NoGesture),
            x => Err(OpticalSensorError::UnknownUint(x)),
        }
    }

    /// Get the Optical Sensor’s most recent raw gesture data
    pub fn get_gesture_raw(&self) -> Result<OpticalGesture, OpticalSensorError> {
        match unsafe { bindings::optical_get_gesture_raw(self.port) } {
            data if data.time == bindings::PROS_ERR_U_ => Err(OpticalSensorError::from_errno().into()),
            data => Ok(OpticalGesture {
                up: data.udata,
                down: data.ddata,
                left: data.ldata,
                right: data.rdata,
                r#type: data.type_,
                padding: data.rdata,
                count: data.count,
                time: data.time,
            }),
        }
    }
}

impl TryFrom<OpticalSensor<DetectGestures>> for OpticalSensor<IgnoreGestures> {
    type Error = (OpticalSensorError, OpticalSensor<DetectGestures>);
    fn try_from(sensor: OpticalSensor<DetectGestures>) -> Result<OpticalSensor<IgnoreGestures>, (OpticalSensorError, OpticalSensor<DetectGestures>)> {
        match unsafe { bindings::optical_disable_gesture(sensor.port) } {
            1 => Ok(OpticalSensor {
                port: sensor.port,
                gesture_detection: IgnoreGestures,
            }),
            _ => Err((OpticalSensorError::from_errno(), sensor))
        }
    }
}

impl TryFrom<OpticalSensor<IgnoreGestures>> for OpticalSensor<DetectGestures> {
    type Error = (OpticalSensorError, OpticalSensor<IgnoreGestures>);
    fn try_from(sensor: OpticalSensor<IgnoreGestures>) -> Result<OpticalSensor<DetectGestures>, (OpticalSensorError, OpticalSensor<IgnoreGestures>)> {
        match unsafe { bindings::optical_enable_gesture(sensor.port) } {
            1 => Ok(OpticalSensor {
                port: sensor.port,
                gesture_detection: DetectGestures,
            }),
            _ => Err((OpticalSensorError::from_errno(), sensor))
        }
    }
}

/// Represents possible errors for optical sensor operations.
#[derive(Debug)]
pub enum OpticalSensorError {
    /// Port is out of range (1-21).
    PortOutOfRange,
    /// Port cannot be configured as a optical sensor.
    PortNotOpticalSensor,
    /// User supplied an invalid value
    InvalidValue,
    /// Unknown error (signed),
    UnknownInt(i32),
    /// Unknown error (unsigned),
    UnknownUint(u32),
}

impl OpticalSensorError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::ENXIO => Self::PortOutOfRange,
            libc::ENODEV => Self::PortNotOpticalSensor,
            x => Self::UnknownInt(x),
        }
    }
}

impl From<OpticalSensorError> for Error {
    fn from(err: OpticalSensorError) -> Self {
        match err {
            OpticalSensorError::PortOutOfRange => Error::Custom("port out of range".into()),
            OpticalSensorError::PortNotOpticalSensor => {
                Error::Custom("port cannot be configured as an optical sensor".into())
            }
            OpticalSensorError::InvalidValue => Error::Custom("User provided invalid value".into()),
            OpticalSensorError::UnknownInt(n) => Error::System(n),
            OpticalSensorError::UnknownUint(n) => Error::System(n.try_into().expect("OpticalSensorError::UnknownUint cannot convert to an Int")),
        }
    }
}

/// Represents possible directions for optical sensor gestures.
#[derive(Debug)]
pub enum OpticalDirection {
    /// A gesture in the down direction
    Down,
    /// An error in gesture recognition
    Error,
    /// A gesture in the left direction
    Left,
    /// No gesture was recognized
    NoGesture,
    /// A gesture in the right direction
    Right,
    /// A gesture in the up direction
    Up,
}

/// Represents optical sensor raw gesture data.
#[derive(Debug)]
pub struct OpticalGesture {
    /// Up component of gesture
    up: u8,
    /// Down component of gesture
    down: u8,
    /// Left component of gesture
    left: u8,
    /// Right component of gesture
    right: u8,
    /// Type of gesture
    r#type: u8,
    /// Padding
    padding: u8,
    /// Number of gestures
    count: u16,
    /// Time since gesture was recognized
    time: u32,
}

/// Represents optical sensor raw data.
#[derive(Debug)]
pub struct OpticalRaw {
    /// Clear component
    clear: u32,
    /// Red component
    red: u32,
    /// Green component
    green: u32,
    /// Blue component
    blue: u32,
}

/// Represents optical sensor RGB data.
pub struct OpticalRGB {
    /// Red component
    red: f64,
    /// Green component
    green: f64,
    /// Blue component
    blue: f64,
    /// Brightness component
    brightness: f64,
}

