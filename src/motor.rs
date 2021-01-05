//! # Motor API.

use crate::{bindings, error::get_errno};

/// A struct which represents a V5 smart port configured as a motor.
pub struct Motor {
    port: u8,
}

impl Motor {
    /// Constructs a new motor.
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same motor. You likely want to use
    /// [`Peripherals::take()`](crate::peripherals::Peripherals::take())
    /// instead.
    pub unsafe fn new(port: u8, gearset: Gearset, reverse: bool) -> Motor {
        let mut motor = Motor { port };
        motor.set_reversed(reverse).unwrap();
        motor.set_gearing(gearset).unwrap();
        motor.set_encoder_units(EncoderUnits::EncoderTicks).unwrap();
        motor
    }

    /// Sets the voltage for the motor from -127 to 127.
    ///
    /// This is designed to map easily to the input from the controller's analog
    /// stick for simple opcontrol use. The actual behavior of the motor is
    /// analogous to use of [`Motor::move_voltage()`].
    pub fn move_i8(&mut self, voltage: i8) -> Result<(), MotorError> {
        match unsafe { bindings::motor_move(self.port, voltage as i32) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the target absolute position for the motor to move to.
    ///
    /// This movement is relative to the position of the motor when initialized
    /// or the position when it was most recently reset with
    /// [`Motor::set_zero_position()`].
    ///
    /// **Note:** This function simply sets the target for the motor, it does
    /// not block program execution until the movement finishes.
    pub fn move_absolute(&mut self, position: f64, velocity: i32) -> Result<(), MotorError> {
        match unsafe { bindings::motor_move_absolute(self.port, position, velocity) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the relative target position for the motor to move to.
    ///
    /// This movement is relative to the current position of the motor as given
    /// in [`Motor::get_position()`]. Providing 10.0 as the position parameter
    /// would result in the motor moving clockwise 10 ticks, no matter what
    /// the current position is.
    ///
    /// **Note:** This function simply sets the target for the motor, it does
    /// not block program execution until the movement finishes.
    pub fn move_relative(&mut self, position: f64, velocity: i32) -> Result<(), MotorError> {
        match unsafe { bindings::motor_move_relative(self.port, position, velocity) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the velocity for the motor.
    ///
    /// This velocity corresponds to different actual speeds depending on the
    /// gearset used for the motor. This results in a range of ±100 for
    /// [`Gearset::ThirtySixToOne`] ±200 for [`Gearset::EighteenToOne`] and ±600
    /// for [`Gearset::SixToOne`]. The velocity is held with PID to ensure
    /// consistent speed.
    pub fn move_velocity(&mut self, velocity: i32) -> Result<(), MotorError> {
        match unsafe { bindings::motor_move_velocity(self.port, velocity) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the output voltage for the motor from -12000 to 12000 in
    /// millivolts.
    pub fn move_voltage(&mut self, voltage: i32) -> Result<(), MotorError> {
        match unsafe { bindings::motor_move_voltage(self.port, voltage) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Changes the output velocity for a profiled movement
    /// ([`Motor::move_absolute()`] or [`Motor::move_relative()`]). This
    /// will have no effect if the motor is not following a profiled movement.
    pub fn modify_profiled_velocity(&mut self, velocity: i32) -> Result<(), MotorError> {
        match unsafe { bindings::motor_modify_profiled_velocity(self.port, velocity) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Gets the target position set for the motor by the user.
    pub fn get_target_position(&self) -> Result<f64, MotorError> {
        match unsafe { bindings::motor_get_target_position(self.port) } {
            x if x == bindings::VEX_RT_PROS_ERR_F => Err(MotorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the velocity commanded to the motor by the user.
    pub fn get_target_velocity(&self) -> Result<i32, MotorError> {
        match unsafe { bindings::motor_get_target_velocity(self.port) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the actual velocity of the motor.
    pub fn get_actual_velocity(&self) -> Result<f64, MotorError> {
        match unsafe { bindings::motor_get_actual_velocity(self.port) } {
            x if x == bindings::VEX_RT_PROS_ERR_F => Err(MotorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the current drawn by the motor in milliamperes.
    pub fn get_current_draw(&self) -> Result<i32, MotorError> {
        match unsafe { bindings::motor_get_current_draw(self.port) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the direction of movement for the motor.
    pub fn get_direction(&self) -> Result<Direction, MotorError> {
        match unsafe { bindings::motor_get_direction(self.port) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            1 => Ok(Direction::Positive),
            -1 => Ok(Direction::Negative),
            x => panic!(
                "bindings::motor_get_direction returned unexpected value: {}",
                x
            ),
        }
    }

    /// Gets the efficiency of the motor in percent.
    pub fn get_efficiency(&self) -> Result<f64, MotorError> {
        match unsafe { bindings::motor_get_efficiency(self.port) } {
            x if x == bindings::VEX_RT_PROS_ERR_F => Err(MotorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the absolute position of the motor in encoder ticks.
    pub fn get_position(&self) -> Result<f64, MotorError> {
        match unsafe { bindings::motor_get_position(self.port) } {
            x if x == bindings::VEX_RT_PROS_ERR_F => Err(MotorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the power drawn by the motor in Watts.
    pub fn get_power(&self) -> Result<f64, MotorError> {
        match unsafe { bindings::motor_get_power(self.port) } {
            x if x == bindings::VEX_RT_PROS_ERR_F => Err(MotorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the temperature of the motor in degrees Celsius.
    pub fn get_temperature(&self) -> Result<f64, MotorError> {
        match unsafe { bindings::motor_get_temperature(self.port) } {
            x if x == bindings::VEX_RT_PROS_ERR_F => Err(MotorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the torque of the motor in Newton-Meters.
    pub fn get_torque(&self) -> Result<f64, MotorError> {
        match unsafe { bindings::motor_get_torque(self.port) } {
            x if x == bindings::VEX_RT_PROS_ERR_F => Err(MotorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the voltage delivered to the motor in millivolts.
    pub fn get_voltage(&self) -> Result<i32, MotorError> {
        match unsafe { bindings::motor_get_voltage(self.port) } {
            x if x == bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Checks if the motor is drawing over its current limit.
    pub fn is_over_current(&self) -> Result<bool, MotorError> {
        match unsafe { bindings::motor_is_over_current(self.port) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    /// Checks if the motor's temperature is above its limit.
    pub fn is_over_temp(&self) -> Result<bool, MotorError> {
        match unsafe { bindings::motor_is_over_temp(self.port) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    /// Gets the brake mode that was set for the motor.
    pub fn get_brake_mode(&self) -> Result<BrakeMode, MotorError> {
        match unsafe { bindings::motor_get_brake_mode(self.port) } {
            bindings::motor_brake_mode_e::E_MOTOR_BRAKE_BRAKE => Ok(BrakeMode::Brake),
            bindings::motor_brake_mode_e::E_MOTOR_BRAKE_COAST => Ok(BrakeMode::Coast),
            bindings::motor_brake_mode_e::E_MOTOR_BRAKE_HOLD => Ok(BrakeMode::Hold),
            bindings::motor_brake_mode_e::E_MOTOR_BRAKE_INVALID => Err(MotorError::from_errno()),
        }
    }

    /// Gets the current limit for the motor in milliamperes.
    ///
    /// The default value is 2500 milliamperes, however the effective limit may
    /// be lower if more then 8 motors are competing for power.
    pub fn get_current_limit(&self) -> Result<i32, MotorError> {
        match unsafe { bindings::motor_get_current_limit(self.port) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the gearset that was set for the motor.
    pub fn get_gearing(&self) -> Result<Gearset, MotorError> {
        match unsafe { bindings::motor_get_gearing(self.port) } {
            bindings::motor_gearset_e_t::E_MOTOR_GEARSET_36 => Ok(Gearset::SixToOne),
            bindings::motor_gearset_e_t::E_MOTOR_GEARSET_18 => Ok(Gearset::EighteenToOne),
            bindings::motor_gearset_e_t::E_MOTOR_GEARSET_06 => Ok(Gearset::ThirtySixToOne),
            bindings::motor_gearset_e_t::E_MOTOR_GEARSET_INVALID => Err(MotorError::from_errno()),
        }
    }

    /// Gets the voltage limit set by the user in volts.
    ///
    /// Default value is 0V, which means that there is no software limitation
    /// imposed on the voltage.
    pub fn get_voltage_limit(&self) -> Result<i32, MotorError> {
        match unsafe { bindings::motor_get_voltage_limit(self.port) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            x => Ok(x),
        }
    }

    /// Gets the operation direction of the motor as set by the user.
    ///
    /// Returns 1 if the motor has been reversed and 0 if the motor was not.
    pub fn is_reversed(&self) -> Result<bool, MotorError> {
        match unsafe { bindings::motor_is_reversed(self.port) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    /// Gets the brake mode that was set for the motor.
    pub fn set_brake_mode(&mut self, mode: BrakeMode) -> Result<(), MotorError> {
        match unsafe { bindings::motor_set_brake_mode(self.port, mode.into()) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the current limit for the motor in milliamperes.
    pub fn set_current_limit(&mut self, limit: i32) -> Result<(), MotorError> {
        match unsafe { bindings::motor_set_current_limit(self.port, limit) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets one of [`Gearset`] for the motor.
    pub fn set_gearing(&mut self, gearset: Gearset) -> Result<(), MotorError> {
        match unsafe { bindings::motor_set_gearing(self.port, gearset.into()) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the reverse flag for the motor.
    ///
    /// This will invert its movements and the values returned for its position.
    pub fn set_reversed(&mut self, reverse: bool) -> Result<(), MotorError> {
        match unsafe { bindings::motor_set_reversed(self.port, reverse) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the voltage limit for the motor in Volts.
    pub fn set_voltage_limit(&mut self, limit: i32) -> Result<(), MotorError> {
        match unsafe { bindings::motor_set_voltage_limit(self.port, limit) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the "absolute" zero position of the motor.
    pub fn set_zero_position(&mut self, position: f64) -> Result<(), MotorError> {
        match unsafe { bindings::motor_set_zero_position(self.port, position) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the "absolute" zero position of the motor to its current position.
    pub fn tare_position(&mut self) -> Result<(), MotorError> {
        match unsafe { bindings::motor_tare_position(self.port) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    fn set_encoder_units(&mut self, units: EncoderUnits) -> Result<(), MotorError> {
        match unsafe { bindings::motor_set_encoder_units(self.port, units.into()) } {
            bindings::VEX_RT_PROS_ERR => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }
}

/// Represents possible errors for motor operations.
#[derive(Debug)]
pub enum MotorError {
    /// Port is out of range (1-21).
    PortOutOfRange,
    /// Port cannot be configured as a motor.
    PortNotMotor,
    /// Unknown error.
    Unknown(i32),
}

impl MotorError {
    fn from_errno() -> Self {
        match get_errno() {
            bindings::VEX_RT_ENXIO => Self::PortOutOfRange,
            bindings::VEX_RT_ENODEV => Self::PortNotMotor,
            x => Self::Unknown(x),
        }
    }
}

/// Represents possible brake modes for a motor.
pub enum BrakeMode {
    /// Motor coasts when stopped.
    Coast,
    /// Motor brakes when stopped.
    Brake,
    /// Motor holds position when stopped.
    Hold,
}

impl Into<bindings::motor_brake_mode_e> for BrakeMode {
    fn into(self) -> bindings::motor_brake_mode_e {
        match self {
            Self::Coast => bindings::motor_brake_mode_e::E_MOTOR_BRAKE_COAST,
            Self::Brake => bindings::motor_brake_mode_e::E_MOTOR_BRAKE_BRAKE,
            Self::Hold => bindings::motor_brake_mode_e::E_MOTOR_BRAKE_HOLD,
        }
    }
}

/// Represents possible gear cartridges for a motor.
pub enum Gearset {
    /// Blue 6:1 Gearset (600RPM).
    SixToOne,
    /// Green 18:1 Gearset (200RPM).
    EighteenToOne,
    /// Red 36:1 Gearset (100RPM).
    ThirtySixToOne,
}

impl Into<bindings::motor_gearset_e> for Gearset {
    fn into(self) -> bindings::motor_gearset_e {
        match self {
            Self::SixToOne => bindings::motor_gearset_e::E_MOTOR_GEARSET_06,
            Self::EighteenToOne => bindings::motor_gearset_e::E_MOTOR_GEARSET_18,
            Self::ThirtySixToOne => bindings::motor_gearset_e::E_MOTOR_GEARSET_36,
        }
    }
}

/// Represents two possible directions of movement for a robot.
pub enum Direction {
    /// The positive direction.
    Positive,
    /// The negative direction.
    Negative,
}

/// Represets the possible encoder units
enum EncoderUnits {
    EncoderTicks,
    Degrees,
    Rotations,
}

impl Into<bindings::motor_encoder_units_e> for EncoderUnits {
    fn into(self) -> bindings::motor_encoder_units_e {
        match self {
            Self::EncoderTicks => bindings::motor_encoder_units_e::E_MOTOR_ENCODER_COUNTS,
            Self::Degrees => bindings::motor_encoder_units_e::E_MOTOR_ENCODER_DEGREES,
            Self::Rotations => bindings::motor_encoder_units_e::E_MOTOR_ENCODER_ROTATIONS,
        }
    }
}
