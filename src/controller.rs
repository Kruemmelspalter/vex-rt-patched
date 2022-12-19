//! Controller API.

use core::convert::TryInto;

use crate::{
    bindings,
    error::{get_errno, Error},
    rtos::{Broadcast, BroadcastListener},
};

/// Represents a Vex controller.
pub struct Controller {
    id: bindings::controller_id_e_t,
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
            id,
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

    /// Returns battery capacity.
    pub fn get_battery_capacity(&self) -> Result<i32, ControllerError> {
        match unsafe { bindings::controller_get_battery_capacity(self.id) } {
            bindings::PROS_ERR_ => Err(ControllerError::from_errno()),
            x => Ok(x),
        }
    }

    /// Returns battery level.
    pub fn get_battery_level(&self) -> Result<i32, ControllerError> {
        match unsafe { bindings::controller_get_battery_level(self.id) } {
            bindings::PROS_ERR_ => Err(ControllerError::from_errno()),
            x => Ok(x),
        }
    }

    /// Converts `self` into a [`ControllerBroadcast`].
    pub fn into_broadcast(self) -> ControllerBroadcast {
        ControllerBroadcast {
            controller: self,
            bcast: Broadcast::new(ControllerData::default()),
        }
    }

    /// Reads data from all controller inputs.
    pub fn read(&self) -> Result<ControllerData, ControllerError> {
        Ok(ControllerData {
            left_x: self.left_stick.get_x()?,
            left_y: self.left_stick.get_y()?,
            right_x: self.right_stick.get_x()?,
            right_y: self.right_stick.get_y()?,
            l1: self.l1.is_pressed()?,
            l2: self.l2.is_pressed()?,
            r1: self.r1.is_pressed()?,
            r2: self.r2.is_pressed()?,
            up: self.up.is_pressed()?,
            down: self.down.is_pressed()?,
            left: self.left.is_pressed()?,
            right: self.right.is_pressed()?,
            x: self.x.is_pressed()?,
            y: self.y.is_pressed()?,
            a: self.a.is_pressed()?,
            b: self.b.is_pressed()?,
        })
    }
}

/// Wraps a [`Broadcast`] instance which broadcasts controller data.
pub struct ControllerBroadcast {
    controller: Controller,
    bcast: Broadcast<ControllerData>,
}

impl ControllerBroadcast {
    /// Reads the latest data from the controller and broadcasts it to all
    /// listeners.
    ///
    /// Returns a copy of the data which was read.
    pub fn broadcast_update(&self) -> Result<ControllerData, ControllerError> {
        let data = self.controller.read()?;
        self.bcast.publish(data);
        Ok(data)
    }

    /// Creates a new listener for the broadcast event.
    pub fn listen(&self) -> BroadcastListener<'_, ControllerData> {
        self.bcast.listen()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Describes data from all controller inputs.
pub struct ControllerData {
    /// The x-axis of the left analog stick.
    pub left_x: i8,
    /// The y-axis of the left analog stick.
    pub left_y: i8,
    /// The x-axis of the right analog stick.
    pub right_x: i8,
    /// The y-axis of the right analog stick.
    pub right_y: i8,
    /// The top-left shoulder button.
    pub l1: bool,
    /// The bottom-left shoulder button.
    pub l2: bool,
    /// The top-right shoulder button.
    pub r1: bool,
    /// The bottom-right shoulder button.
    pub r2: bool,
    /// The up directional button.
    pub up: bool,
    /// The down directional button.
    pub down: bool,
    /// The left directional button.
    pub left: bool,
    /// The right directional button.
    pub right: bool,
    /// The "X" button.
    pub x: bool,
    /// The "Y" button.
    pub y: bool,
    /// The "A" button.
    pub a: bool,
    /// The "B" button.
    pub b: bool,
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
