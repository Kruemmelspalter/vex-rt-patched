//! Controller API.

use alloc::collections::VecDeque;
use core::{convert::TryInto, fmt, time::Duration};
use slice_copy::copy;

use crate::{
    bindings,
    error::{get_errno, Error},
    io::eprintln,
    rtos::{delay_until, queue, time_since_start, DataSource, SendQueue, Task},
    select,
};

const SCREEN_SUCCESS_DELAY: Duration = Duration::from_millis(50);
const SCREEN_FAILURE_DELAY: Duration = Duration::from_millis(5);

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
    /// The LCD screen
    pub screen: Screen,
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
            screen: Screen { id, queue: None },
        }
    }

    /// Returns false or true if the controller is connected.
    pub fn is_connected(&self) -> Result<bool, ControllerError> {
        match unsafe { bindings::controller_is_connected(self.id) } {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ControllerError::from_errno()),
        }
    }

    /// Gets the battery level of the controller.
    pub fn get_battery_level(&self) -> Result<i32, ControllerError> {
        match unsafe { bindings::controller_get_battery_level(self.id) } {
            bindings::PROS_ERR_ => Err(ControllerError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the battery capacity of the controller.
    pub fn get_battery_capacity(&self) -> Result<i32, ControllerError> {
        match unsafe { bindings::controller_get_battery_capacity(self.id) } {
            bindings::PROS_ERR_ => Err(ControllerError::from_errno()),
            x => Ok(x),
        }
    }
}

impl fmt::Debug for Controller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Controller").field("id", &self.id).finish()
    }
}

impl DataSource for Controller {
    type Data = ControllerData;

    type Error = ControllerError;

    fn read(&self) -> Result<Self::Data, Self::Error> {
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
            battery_level: self.get_battery_level()?,
            battery_capacity: self.get_battery_capacity()?,
        })
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
    /// The battery level of the controller.
    pub battery_level: i32,
    /// The battery capacity of the controller.
    pub battery_capacity: i32,
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
                        "bindings::controller_get_analog returned unexpected value: {}",
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
    /// Checks if a given button is pressed. Returns false if the controller is
    /// not connected.
    pub fn is_pressed(&self) -> Result<bool, ControllerError> {
        match unsafe { bindings::controller_get_digital(self.id, self.button) } {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ControllerError::from_errno()),
        }
    }
}

/// Represents the screen on a Vex controller
pub struct Screen {
    id: bindings::controller_id_e_t,
    queue: Option<SendQueue<ScreenCommand>>,
}

impl Screen {
    /// Clears all of the lines of the controller screen
    pub fn clear(&mut self) {
        self.command(ScreenCommand::Clear);
    }

    /// Clears an individual line of the controller screen. Lines range from 0
    /// to 2
    pub fn clear_line(&mut self, line: u8) {
        if line > 2 {
            return;
        }
        self.command(ScreenCommand::ClearLine(line));
    }

    /// Prints text to the controller LCD screen. Lines range from 0 to 2.
    /// Columns range from 0 to 18
    pub fn print(&mut self, line: u8, column: u8, str: &str) {
        if line > 2 || column > 18 {
            return;
        }
        let mut chars: [libc::c_char; 19] = Default::default();
        copy(&mut chars, str.as_bytes());
        self.command(ScreenCommand::Print {
            chars,
            line,
            column,
            length: str.as_bytes().len() as u8,
        });
    }

    /// Rumble the controller. Rumble pattern is a string consisting of the
    /// characters ‘.’, ‘-’, and ‘ ‘, where dots are short rumbles, dashes are
    /// long rumbles, and spaces are pauses; all other characters are ignored.
    /// Maximum supported length is 8 characters.
    pub fn rumble(&mut self, rumble_pattern: &str) {
        let mut pattern: [libc::c_char; 8] = Default::default();
        let mut i = 0;
        for c in rumble_pattern.chars() {
            match c {
                '.' | '-' | '_' => {
                    pattern[i] = c as libc::c_char;
                    i += 1;
                }
                _ => {}
            }
            if i >= pattern.len() {
                break;
            }
        }
        self.command(ScreenCommand::Rumble(pattern));
    }

    fn command(&mut self, cmd: ScreenCommand) {
        self.queue().send(cmd);
    }

    fn queue(&mut self) -> &mut SendQueue<ScreenCommand> {
        self.queue.get_or_insert_with(|| {
            let name = match self.id {
                bindings::controller_id_e_t_E_CONTROLLER_MASTER => "controller-screen-master",
                bindings::controller_id_e_t_E_CONTROLLER_PARTNER => "controller-screen-partner",
                _ => "",
            };
            let id = self.id;
            let (send, recv) = queue(VecDeque::<ScreenCommand>::new());
            Task::spawn_ext(
                name,
                bindings::TASK_PRIORITY_MAX,
                bindings::TASK_STACK_DEPTH_DEFAULT as u16,
                move || {
                    let mut delay_target = None;
                    let mut offset = 0usize;
                    let mut clear = false;
                    let mut buffer = [ScreenRow::default(); 3];
                    let mut rumble: Option<[libc::c_char; 9]> = None;
                    'main: loop {
                        let command: Option<ScreenCommand> = select! {
                            cmd = recv.select() => Some(cmd),
                            _ = delay_until(t); Some(t) = delay_target => None,
                        };
                        if let Some(cmd) = command {
                            match cmd {
                                ScreenCommand::Clear => {
                                    offset = 0;
                                    clear = true;
                                    buffer = Default::default();
                                }
                                ScreenCommand::ClearLine(line) => {
                                    let row = &mut buffer[line as usize];
                                    *row = ScreenRow::default();
                                    row.needs_clear = true;
                                }
                                ScreenCommand::Print {
                                    chars,
                                    line,
                                    column,
                                    length,
                                } => {
                                    let row = &mut buffer[line as usize];
                                    copy(
                                        &mut row.chars[column as usize..],
                                        &chars[..length as usize],
                                    );
                                    row.dirty = true;
                                }
                                ScreenCommand::Rumble(pattern) => {
                                    let mut buf: [libc::c_char; 9] = Default::default();
                                    copy(&mut buf, &pattern);
                                    rumble = Some(buf);
                                }
                                ScreenCommand::Stop => break,
                            }
                        }
                        if let Some(pattern) = rumble {
                            match unsafe { bindings::controller_rumble(id, pattern.as_ptr()) } {
                                1 => {
                                    delay_target = Some(time_since_start() + SCREEN_SUCCESS_DELAY);
                                    rumble = None;
                                }
                                _ => {
                                    delay_target = Some(time_since_start() + SCREEN_FAILURE_DELAY);
                                    Self::print_error()
                                }
                            }
                        } else if clear {
                            match unsafe { bindings::controller_clear(id) } {
                                1 => {
                                    delay_target = Some(time_since_start() + SCREEN_SUCCESS_DELAY);
                                    clear = false;
                                }
                                _ => {
                                    delay_target = Some(time_since_start() + SCREEN_FAILURE_DELAY);
                                    Self::print_error()
                                }
                            }
                        } else {
                            for i in 0..3 {
                                let index = (offset + i) % buffer.len();
                                let row = &mut buffer[index];
                                if row.needs_clear {
                                    match unsafe {
                                        bindings::controller_clear_line(id, index as u8)
                                    } {
                                        1 => {
                                            delay_target =
                                                Some(time_since_start() + SCREEN_SUCCESS_DELAY);
                                            row.needs_clear = false;
                                        }
                                        _ => {
                                            delay_target =
                                                Some(time_since_start() + SCREEN_FAILURE_DELAY);
                                            Self::print_error()
                                        }
                                    }
                                } else if row.dirty {
                                    match unsafe {
                                        bindings::controller_set_text(
                                            id,
                                            index as u8,
                                            0,
                                            row.chars.as_ptr(),
                                        )
                                    } {
                                        1 => {
                                            delay_target =
                                                Some(time_since_start() + SCREEN_SUCCESS_DELAY);
                                            row.dirty = false;
                                        }
                                        _ => {
                                            delay_target =
                                                Some(time_since_start() + SCREEN_FAILURE_DELAY);
                                            Self::print_error()
                                        }
                                    }
                                } else {
                                    continue;
                                }
                                offset = i + 1;
                                continue 'main;
                            }
                            // No updates were made; delay indefinitely until next command.
                            delay_target = None;
                        }
                    }
                },
            )
            .unwrap();
            send
        })
    }

    fn print_error() {
        if get_errno() != libc::EAGAIN {
            eprintln!("{:?}", ControllerError::from_errno());
        }
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        if self.queue.is_some() {
            self.command(ScreenCommand::Stop);
        }
    }
}

#[derive(Copy, Clone)]
struct ScreenRow {
    chars: [libc::c_char; 20],
    dirty: bool,
    needs_clear: bool,
}

impl Default for ScreenRow {
    fn default() -> Self {
        // All spaces except last.
        let mut chars = [0x20; 20];
        chars[19] = 0;
        Self {
            chars,
            dirty: Default::default(),
            needs_clear: Default::default(),
        }
    }
}

#[derive(Debug)]
enum ScreenCommand {
    Clear,
    ClearLine(u8),
    Print {
        chars: [libc::c_char; 19],
        line: u8,
        column: u8,
        length: u8,
    },
    Rumble([libc::c_char; 8]),
    Stop,
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
