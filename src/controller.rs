//! Controller API.

use core::convert::TryInto;

use crate::{
    bindings,
    error::{get_errno, Error},
};

/// Represents a Vex controller.
pub struct Controller {
    /// The left analog stick.
    pub left_stick: AnalogStick,
    /// The right analog stick.
    pub right_stick: AnalogStick,
    /// The top-left shoulder button.
    pub l1: Button,
    /// The bottom-left shoulder button.
    pub l2: Button,
    /// The top-right shoulder button.
    pub r1: Button,
    /// The bottom-right shoulder button.
    pub r2: Button,
    /// The up directional button.
    pub up: Button,
    /// The down directional button.
    pub down: Button,
    /// The left directional button.
    pub left: Button,
    /// The right directional button.
    pub right: Button,
    /// The "X" button.
    pub x: Button,
    /// The "Y" button.
    pub y: Button,
    /// The "A" button.
    pub a: Button,
    /// The "B" button.
    pub b: Button,
}

impl Controller {
    /// Creates a new controller.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same controller. You likely want to implement
    /// [`Robot::new`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(id: ControllerId) -> Self {
        let id: bindings::controller_id_e_t = id.into();
        Controller {
            left_stick: AnalogStick {
                id,
                x_channel: bindings::controller_analog_e_t_E_CONTROLLER_ANALOG_LEFT_X,
                y_channel: bindings::controller_analog_e_t_E_CONTROLLER_ANALOG_LEFT_Y,
            },
            right_stick: AnalogStick {
                id,
                x_channel: bindings::controller_analog_e_t_E_CONTROLLER_ANALOG_RIGHT_X,
                y_channel: bindings::controller_analog_e_t_E_CONTROLLER_ANALOG_RIGHT_Y,
            },
            l1: Button {
                id,
                button: bindings::controller_digital_e_t_E_CONTROLLER_DIGITAL_L1,
            },
            l2: Button {
                id,
                button: bindings::controller_digital_e_t_E_CONTROLLER_DIGITAL_L2,
            },
            r1: Button {
                id,
                button: bindings::controller_digital_e_t_E_CONTROLLER_DIGITAL_R1,
            },
            r2: Button {
                id,
                button: bindings::controller_digital_e_t_E_CONTROLLER_DIGITAL_R2,
            },
            up: Button {
                id,
                button: bindings::controller_digital_e_t_E_CONTROLLER_DIGITAL_UP,
            },
            down: Button {
                id,
                button: bindings::controller_digital_e_t_E_CONTROLLER_DIGITAL_DOWN,
            },
            right: Button {
                id,
                button: bindings::controller_digital_e_t_E_CONTROLLER_DIGITAL_RIGHT,
            },
            left: Button {
                id,
                button: bindings::controller_digital_e_t_E_CONTROLLER_DIGITAL_LEFT,
            },
            x: Button {
                id,
                button: bindings::controller_digital_e_t_E_CONTROLLER_DIGITAL_X,
            },
            y: Button {
                id,
                button: bindings::controller_digital_e_t_E_CONTROLLER_DIGITAL_Y,
            },
            b: Button {
                id,
                button: bindings::controller_digital_e_t_E_CONTROLLER_DIGITAL_B,
            },
            a: Button {
                id,
                button: bindings::controller_digital_e_t_E_CONTROLLER_DIGITAL_A,
            },
        }
    }
}

/// Represents one of two analog sticks on a Vex controller.
pub struct AnalogStick {
    id: bindings::controller_id_e_t,
    x_channel: bindings::controller_analog_e_t,
    y_channel: bindings::controller_analog_e_t,
}

impl AnalogStick {
    /// Reads an analog stick's x-axis. Returns a value on the range [-127,
    /// 127] where -127 is all the way left, 0 is centered, and 127 is all the
    /// way right. Also returns 0 if controller is not connected.
    pub fn get_x(&self) -> Result<i8, ControllerError> {
        self.get_channel(self.x_channel)
    }

    /// Reads an analog stick's y-axis. Returns a value on the range [-127,
    /// 127] where -127 is all the way down, 0 is centered, and 127 is all the
    /// way up. Also returns 0 if controller is not connected.
    pub fn get_y(&self) -> Result<i8, ControllerError> {
        self.get_channel(self.y_channel)
    }

    fn get_channel(&self, channel: bindings::controller_analog_e_t) -> Result<i8, ControllerError> {
        match unsafe { bindings::controller_get_analog(self.id, channel) } {
            bindings::PROS_ERR_ => Err(ControllerError::from_errno()),
            x => match x.try_into() {
                Ok(converted_x) => Ok(converted_x),
                Err(_) => {
                    panic!(
                        "bindings::motor_get_direction returned unexpected value: {}",
                        x
                    )
                }
            },
        }
    }
}

/// Represents a button on a Vex controller.
pub struct Button {
    id: bindings::controller_id_e_t,
    button: bindings::controller_digital_e_t,
}

impl Button {
    /// Checks if a given button is pressed. Returns 0 if the controller is not
    /// connected.
    pub fn is_pressed(&self) -> Result<bool, ControllerError> {
        match unsafe { bindings::controller_get_digital(self.id, self.button) } {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ControllerError::from_errno()),
        }
    }
}

/// Represents the two types of controller.
pub enum ControllerId {
    /// The primary controller.
    Master,
    /// The tethered/partner controller.
    Partner,
}

impl From<ControllerId> for bindings::controller_id_e_t {
    fn from(id: ControllerId) -> Self {
        match id {
            ControllerId::Master => bindings::controller_id_e_t_E_CONTROLLER_MASTER,
            ControllerId::Partner => bindings::controller_id_e_t_E_CONTROLLER_PARTNER,
        }
    }
}

/// Represents possible error states for a controller.
#[derive(Debug)]
pub enum ControllerError {
    /// Controller ID does not exist.
    InvalidController,
    /// Another resource is currently trying to access the controller port.
    ControllerBusy,
    /// Unknown Errno.
    Unknown(i32),
}

impl ControllerError {
    fn from_errno() -> Self {
        match { get_errno() } {
            libc::EINVAL => Self::InvalidController,
            libc::EACCES => Self::ControllerBusy,
            x => Self::Unknown(x),
        }
    }
}

impl From<ControllerError> for Error {
    fn from(err: ControllerError) -> Self {
        match err {
            ControllerError::InvalidController => Error::Custom("invalid controller id".into()),
            ControllerError::ControllerBusy => Error::Custom("controller is busy".into()),
            ControllerError::Unknown(n) => Error::System(n),
        }
    }
}
