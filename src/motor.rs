//! # Motor API.

use core::{
    convert::identity,
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

use uom::si::{
    angle::revolution,
    angular_velocity::revolution_per_minute,
    electric_current::milliampere,
    electric_potential::{millivolt, volt},
    f64::{
        Angle, AngularVelocity, ElectricCurrent, Power, Ratio, ThermodynamicTemperature, Torque,
    },
    power::watt,
    quantities::ElectricPotential,
    ratio::percent,
    thermodynamic_temperature::degree_celsius,
    torque::newton_meter,
};

use crate::{
    bindings,
    error::{get_errno, Error},
    rtos::DataSource,
};

#[repr(transparent)]
/// A struct which represents a V5 smart port configured as a motor.
pub struct Motor {
    port: u8,
}

impl Motor {
    /// Constructs a new motor.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same motor. You likely want to implement
    /// [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8, gearset: Gearset, reverse: bool) -> Result<Self, MotorError> {
        let mut motor = Self { port };
        motor.set_reversed(reverse)?;
        motor.set_gearing(gearset)?;
        match bindings::motor_set_encoder_units(
            port,
            bindings::motor_encoder_units_e_E_MOTOR_ENCODER_ROTATIONS,
        ) {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            _ => Ok(motor),
        }
    }

    /// Sets the voltage for the motor from -127 to 127.
    ///
    /// This is designed to map easily to the input from the controller's analog
    /// stick for simple opcontrol use. The actual behavior of the motor is
    /// analogous to use of [`Motor::move_voltage()`].
    pub fn move_i8(&mut self, voltage: i8) -> Result<(), MotorError> {
        match unsafe { bindings::motor_move(self.port, voltage as i32) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
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
    pub fn move_absolute(
        &mut self,
        position: Angle,
        velocity: AngularVelocity,
    ) -> Result<(), MotorError> {
        match unsafe {
            bindings::motor_move_absolute(
                self.port,
                position.get::<revolution>(),
                velocity.get::<revolution_per_minute>() as i32,
            )
        } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the relative target position for the motor to move to.
    ///
    /// This movement is relative to the current position of the motor as given
    /// in [`Motor::get_position()`]. Providing 10 degrees as the position
    /// parameter would result in the motor moving clockwise by 10 degrees,
    /// no matter what the current position is.
    ///
    /// **Note:** This function simply sets the target for the motor, it does
    /// not block program execution until the movement finishes.
    pub fn move_relative(
        &mut self,
        position: Angle,
        velocity: AngularVelocity,
    ) -> Result<(), MotorError> {
        match unsafe {
            bindings::motor_move_relative(
                self.port,
                position.get::<revolution>(),
                velocity.get::<revolution_per_minute>() as i32,
            )
        } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the velocity for the motor.
    ///
    /// This velocity corresponds to different actual speeds depending on the
    /// gearset used for the motor. This results in a range of ±100 RPM for
    /// [`Gearset::ThirtySixToOne`] ±200 RPM for [`Gearset::EighteenToOne`] and
    /// ±600 RPM for [`Gearset::SixToOne`]. The velocity is held with PID to
    /// ensure consistent speed.
    pub fn move_velocity(&mut self, velocity: AngularVelocity) -> Result<(), MotorError> {
        match unsafe {
            bindings::motor_move_velocity(self.port, velocity.get::<revolution_per_minute>() as i32)
        } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the output voltage for the motor from -12 V to 12 V.
    pub fn move_voltage(&mut self, voltage: ElectricPotential<f64>) -> Result<(), MotorError> {
        match unsafe { bindings::motor_move_voltage(self.port, voltage.get::<millivolt>() as i32) }
        {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Changes the output velocity for a profiled movement
    /// ([`Motor::move_absolute()`] or [`Motor::move_relative()`]). This
    /// will have no effect if the motor is not following a profiled movement.
    pub fn modify_profiled_velocity(
        &mut self,
        velocity: AngularVelocity,
    ) -> Result<(), MotorError> {
        match unsafe {
            bindings::motor_modify_profiled_velocity(
                self.port,
                velocity.get::<revolution_per_minute>() as i32,
            )
        } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Gets the target position set for the motor by the user.
    pub fn get_target_position(&self) -> Result<Angle, MotorError> {
        match unsafe { bindings::motor_get_target_position(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(MotorError::from_errno()),
            x => Ok(Angle::new::<revolution>(x)),
        }
    }

    /// Gets the velocity commanded to the motor by the user.
    pub fn get_target_velocity(&self) -> Result<AngularVelocity, MotorError> {
        match unsafe { bindings::motor_get_target_velocity(self.port) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            x => Ok(AngularVelocity::new::<revolution_per_minute>(x as f64)),
        }
    }

    /// Gets the actual velocity of the motor.
    pub fn get_actual_velocity(&self) -> Result<AngularVelocity, MotorError> {
        match unsafe { bindings::motor_get_actual_velocity(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(MotorError::from_errno()),
            x => Ok(AngularVelocity::new::<revolution_per_minute>(x)),
        }
    }

    /// Gets the current drawn by the motor.
    pub fn get_current_draw(&self) -> Result<ElectricCurrent, MotorError> {
        match unsafe { bindings::motor_get_current_draw(self.port) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            x => Ok(ElectricCurrent::new::<milliampere>(x as f64)),
        }
    }

    /// Gets the direction of movement for the motor.
    pub fn get_direction(&self) -> Result<Direction, MotorError> {
        match unsafe { bindings::motor_get_direction(self.port) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            1 => Ok(Direction::Positive),
            -1 => Ok(Direction::Negative),
            x => panic!(
                "bindings::motor_get_direction returned unexpected value: {}",
                x
            ),
        }
    }

    /// Gets the efficiency of the motor.
    pub fn get_efficiency(&self) -> Result<Ratio, MotorError> {
        match unsafe { bindings::motor_get_efficiency(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(MotorError::from_errno()),
            x => Ok(Ratio::new::<percent>(x)),
        }
    }

    /// Gets the absolute position of the motor.
    pub fn get_position(&self) -> Result<Angle, MotorError> {
        match unsafe { bindings::motor_get_position(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(MotorError::from_errno()),
            x => Ok(Angle::new::<revolution>(x)),
        }
    }

    /// Gets the power drawn by the motor.
    pub fn get_power(&self) -> Result<Power, MotorError> {
        match unsafe { bindings::motor_get_power(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(MotorError::from_errno()),
            x => Ok(Power::new::<watt>(x)),
        }
    }

    /// Gets the temperature of the motor.
    pub fn get_temperature(&self) -> Result<ThermodynamicTemperature, MotorError> {
        match unsafe { bindings::motor_get_temperature(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(MotorError::from_errno()),
            x => Ok(ThermodynamicTemperature::new::<degree_celsius>(x)),
        }
    }

    /// Gets the torque of the motor.
    pub fn get_torque(&self) -> Result<Torque, MotorError> {
        match unsafe { bindings::motor_get_torque(self.port) } {
            x if x == bindings::PROS_ERR_F_ => Err(MotorError::from_errno()),
            x => Ok(Torque::new::<newton_meter>(x)),
        }
    }

    /// Gets the voltage delivered to the motor.
    pub fn get_voltage(&self) -> Result<ElectricPotential<f64>, MotorError> {
        match unsafe { bindings::motor_get_voltage(self.port) } {
            x if x == bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            x => Ok(ElectricPotential::new::<millivolt>(x as f64)),
        }
    }

    /// Checks if the motor is drawing over its current limit.
    pub fn is_over_current(&self) -> Result<bool, MotorError> {
        match unsafe { bindings::motor_is_over_current(self.port) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    /// Checks if the motor's temperature is above its limit.
    pub fn is_over_temp(&self) -> Result<bool, MotorError> {
        match unsafe { bindings::motor_is_over_temp(self.port) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    /// Gets the brake mode that was set for the motor.
    pub fn get_brake_mode(&self) -> Result<BrakeMode, MotorError> {
        match unsafe { bindings::motor_get_brake_mode(self.port) } {
            bindings::motor_brake_mode_e_E_MOTOR_BRAKE_BRAKE => Ok(BrakeMode::Brake),
            bindings::motor_brake_mode_e_E_MOTOR_BRAKE_COAST => Ok(BrakeMode::Coast),
            bindings::motor_brake_mode_e_E_MOTOR_BRAKE_HOLD => Ok(BrakeMode::Hold),
            bindings::motor_brake_mode_e_E_MOTOR_BRAKE_INVALID => Err(MotorError::from_errno()),
            x => panic!(
                "bindings::motor_get_brake_mode returned unexpected value: {}.",
                x
            ),
        }
    }

    /// Gets the current limit for the motor.
    ///
    /// The default value is 2.5 A, however the effective limit may be lower if
    /// more then 8 motors are competing for power.
    pub fn get_current_limit(&self) -> Result<ElectricCurrent, MotorError> {
        match unsafe { bindings::motor_get_current_limit(self.port) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            x => Ok(ElectricCurrent::new::<milliampere>(x as f64)),
        }
    }

    /// Gets the gearset that was set for the motor.
    pub fn get_gearing(&self) -> Result<Gearset, MotorError> {
        match unsafe { bindings::motor_get_gearing(self.port) } {
            bindings::motor_gearset_e_E_MOTOR_GEARSET_36 => Ok(Gearset::SixToOne),
            bindings::motor_gearset_e_E_MOTOR_GEARSET_18 => Ok(Gearset::EighteenToOne),
            bindings::motor_gearset_e_E_MOTOR_GEARSET_06 => Ok(Gearset::ThirtySixToOne),
            bindings::motor_gearset_e_E_MOTOR_GEARSET_INVALID => Err(MotorError::from_errno()),
            x => panic!(
                "bindings::motor_get_gearing returned unexpected value: {}.",
                x
            ),
        }
    }

    /// Gets the voltage limit set by the user.
    ///
    /// Default value is 0V, which means that there is no software limitation
    /// imposed on the voltage.
    pub fn get_voltage_limit(&self) -> Result<ElectricPotential<i32>, MotorError> {
        match unsafe { bindings::motor_get_voltage_limit(self.port) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            x => Ok(ElectricPotential::new::<volt>(x)),
        }
    }

    /// Gets the operation direction of the motor as set by the user.
    ///
    /// Returns 1 if the motor has been reversed and 0 if the motor was not.
    pub fn is_reversed(&self) -> Result<bool, MotorError> {
        match unsafe { bindings::motor_is_reversed(self.port) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    /// Gets the brake mode that was set for the motor.
    pub fn set_brake_mode(&mut self, mode: BrakeMode) -> Result<(), MotorError> {
        match unsafe { bindings::motor_set_brake_mode(self.port, mode.into()) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the current limit for the motor.
    pub fn set_current_limit(&mut self, limit: ElectricCurrent) -> Result<(), MotorError> {
        match unsafe {
            bindings::motor_set_current_limit(self.port, limit.get::<milliampere>() as i32)
        } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets one of [`Gearset`] for the motor.
    pub fn set_gearing(&mut self, gearset: Gearset) -> Result<(), MotorError> {
        match unsafe { bindings::motor_set_gearing(self.port, gearset.into()) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the reverse flag for the motor.
    ///
    /// This will invert its movements and the values returned for its position.
    pub fn set_reversed(&mut self, reverse: bool) -> Result<(), MotorError> {
        match unsafe { bindings::motor_set_reversed(self.port, reverse) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the voltage limit for the motor.
    pub fn set_voltage_limit(&mut self, limit: ElectricPotential<i32>) -> Result<(), MotorError> {
        match unsafe { bindings::motor_set_voltage_limit(self.port, limit.get::<volt>()) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the "absolute" zero position of the motor.
    pub fn set_zero_position(&mut self, position: Angle) -> Result<(), MotorError> {
        match unsafe { bindings::motor_set_zero_position(self.port, position.get::<revolution>()) }
        {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }

    /// Sets the "absolute" zero position of the motor to its current position.
    pub fn tare_position(&mut self) -> Result<(), MotorError> {
        match unsafe { bindings::motor_tare_position(self.port) } {
            bindings::PROS_ERR_ => Err(MotorError::from_errno()),
            _ => Ok(()),
        }
    }
}

impl DataSource for Motor {
    type Data = MotorData;

    type Error = MotorError;

    fn read(&self) -> Result<Self::Data, Self::Error> {
        Ok(MotorData {
            target_position: self.get_target_position()?,
            target_velocity: self.get_target_velocity()?,
            actual_velocity: self.get_actual_velocity()?,
            current_draw: self.get_current_draw()?,
            direction: self.get_direction()?,
            efficiency: self.get_efficiency()?,
            position: self.get_position()?,
            power: self.get_power()?,
            temperature: self.get_temperature()?,
            torque: self.get_torque()?,
            voltage: self.get_voltage()?,
            over_current: self.is_over_current()?,
            over_temp: self.is_over_temp()?,
            brake_mode: self.get_brake_mode()?,
            current_limit: self.get_current_limit()?,
            voltage_limit: self.get_voltage_limit()?,
        })
    }
}

/// Represents the data that can be read from a motor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotorData {
    /// The target position set for the motor by the user.
    pub target_position: Angle,
    /// The velocity commanded to the motor by the user.
    pub target_velocity: AngularVelocity,
    /// The actual velocity of the motor.
    pub actual_velocity: AngularVelocity,
    /// The current drawn by the motor in milliamperes.
    pub current_draw: ElectricCurrent,
    /// The direction of movement for the motor.
    pub direction: Direction,
    /// The efficiency of the motor in percent.
    pub efficiency: Ratio,
    /// The absolute position of the motor in encoder ticks.
    pub position: Angle,
    /// The power drawn by the motor in watts.
    pub power: Power,
    /// The temperature of the motor in degrees Celsius.
    pub temperature: ThermodynamicTemperature,
    /// The torque of the motor in newton-metres.
    pub torque: Torque,
    /// The voltage delivered to the motor in millivolts.
    pub voltage: ElectricPotential<f64>,
    /// Whether the motor is drawing over its current limit.
    pub over_current: bool,
    /// Whether the motor's temperature is above its limit.
    pub over_temp: bool,
    /// The brake mode that was set for the motor.
    pub brake_mode: BrakeMode,
    /// The current limit for the motor in milliamperes.
    pub current_limit: ElectricCurrent,
    /// The voltage limit set by the user in volts.
    pub voltage_limit: ElectricPotential<i32>,
}

#[repr(transparent)]
/// Represents a group of motors.
pub struct MotorGroup<const N: usize> {
    motors: [Motor; N],
}

impl<const N: usize> MotorGroup<N> {
    /// Construct a new motor group from a vector of motors.
    pub fn new(motors: [Motor; N]) -> Self {
        Self { motors }
    }

    /// Sets the voltage of all motors in the group from -127 to 127; see
    /// [`Motor::move_i8()`].
    pub fn move_i8(&mut self, voltage: i8) -> Result<(), MotorError> {
        for motor in self.motors.iter_mut() {
            motor.move_i8(voltage)?;
        }
        Ok(())
    }

    /// Sets the target absolute position for all motors in the group; see
    /// [`Motor::move_absolute()`].
    pub fn move_absolute(
        &mut self,
        position: Angle,
        velocity: AngularVelocity,
    ) -> Result<(), MotorError> {
        for motor in self.motors.iter_mut() {
            motor.move_absolute(position, velocity)?;
        }
        Ok(())
    }

    /// Sets the target relative position for all motors in the group; see
    /// [`Motor::move_relative()`].
    pub fn move_relative(
        &mut self,
        position: Angle,
        velocity: AngularVelocity,
    ) -> Result<(), MotorError> {
        for motor in self.motors.iter_mut() {
            motor.move_relative(position, velocity)?;
        }
        Ok(())
    }

    /// Sets the velocity for the motor; see [`Motor::move_velocity()`].
    pub fn move_velocity(&mut self, velocity: AngularVelocity) -> Result<(), MotorError> {
        for motor in self.motors.iter_mut() {
            motor.move_velocity(velocity)?;
        }
        Ok(())
    }

    /// Sets the output voltage for the motor from -12 V to 12 V; see
    /// [`Motor::move_voltage()`].
    pub fn move_voltage(&mut self, voltage: ElectricPotential<f64>) -> Result<(), MotorError> {
        for motor in self.motors.iter_mut() {
            motor.move_voltage(voltage)?;
        }
        Ok(())
    }

    /// Changes the output velocity for a profiled movement; see
    /// [`Motor::modify_profiled_velocity()`].
    pub fn modify_profiled_velocity(
        &mut self,
        velocity: AngularVelocity,
    ) -> Result<(), MotorError> {
        for motor in self.motors.iter_mut() {
            motor.modify_profiled_velocity(velocity)?;
        }
        Ok(())
    }

    /// Gets the actual velocity of each motor; see
    /// [`Motor::get_actual_velocity()`].
    pub fn get_actual_velocity(&self) -> Result<[AngularVelocity; N], MotorError> {
        self.motors.each_ref().try_map(Motor::get_actual_velocity)
    }

    /// Gets the average actual velocity of the motors.
    pub fn get_average_actual_velocity(&self) -> Result<AngularVelocity, MotorError> {
        let mut value = self.get_actual_velocity()?.into_iter().sum();
        value *= (N as f64).recip();
        Ok(value)
    }

    /// Gets the current draw of each motor; see [`Motor::get_current_draw()`].
    pub fn get_current_draw(&self) -> Result<[ElectricCurrent; N], MotorError> {
        self.motors.each_ref().try_map(Motor::get_current_draw)
    }

    /// Gets the total current draw of the motors.
    pub fn get_total_current_draw(&self) -> Result<ElectricCurrent, MotorError> {
        Ok(self.get_current_draw()?.into_iter().sum())
    }

    /// Gets the efficiency of each motor; see [`Motor::get_efficiency()`].
    pub fn get_efficiency(&self) -> Result<[Ratio; N], MotorError> {
        self.motors.each_ref().try_map(Motor::get_efficiency)
    }

    /// Gets the average efficiency of the motors.
    pub fn get_average_efficiency(&self) -> Result<Ratio, MotorError> {
        let mut value = self.get_efficiency()?.into_iter().sum();
        value *= (N as f64).recip();
        Ok(value)
    }

    /// Gets the position of each motor; see [`Motor::get_position`].
    pub fn get_position(&self) -> Result<[Angle; N], MotorError> {
        self.motors.each_ref().try_map(Motor::get_position)
    }

    /// Gets the average position of the motors.
    pub fn get_average_position(&self) -> Result<Angle, MotorError> {
        let mut value = self.get_position()?.into_iter().sum();
        value *= (N as f64).recip();
        Ok(value)
    }

    /// Gets the power drawn by each motor; see [`Motor::get_power()`].
    pub fn get_power(&self) -> Result<[Power; N], MotorError> {
        self.motors.each_ref().try_map(Motor::get_power)
    }

    /// Gets the total power drawn by the motors.
    pub fn get_total_power(&self) -> Result<Power, MotorError> {
        Ok(self.get_power()?.into_iter().sum())
    }

    /// Gets the temperate of each motor; see [`Motor::get_temperature()`].
    pub fn get_temperature(&self) -> Result<[ThermodynamicTemperature; N], MotorError> {
        self.motors.each_ref().try_map(Motor::get_temperature)
    }

    /*
    pub fn get_average_temperature(&self) -> Result<ThermodynamicTemperature, MotorError> {
        let mut value = self.get_temperature()?.into_iter().sum();
        value *= (N as f64).recip();
        Ok(value)
    } // */

    /// Gets the torque applied by each motor; see [`Motor::get_torque()`].
    pub fn get_torque(&self) -> Result<[Torque; N], MotorError> {
        self.motors.each_ref().try_map(Motor::get_torque)
    }

    /// Gets the total torque applied by the motors.
    pub fn get_total_torque(&self) -> Result<Torque, MotorError> {
        Ok(self.get_torque()?.into_iter().sum())
    }

    /// Gets the voltage delivered to each motor; see [`Motor::get_voltage()`].
    pub fn get_voltage(&self) -> Result<[ElectricPotential<f64>; N], MotorError> {
        self.motors.each_ref().try_map(Motor::get_voltage)
    }

    /// Gets the average voltage delivered to the motors.
    pub fn get_average_voltage(&self) -> Result<ElectricPotential<f64>, MotorError> {
        let mut value = self.get_voltage()?.into_iter().sum();
        value *= (N as f64).recip();
        Ok(value)
    }

    /// Checks if each motor is drawing over its current limit; see
    /// [`Motor::is_over_current()`].
    pub fn is_over_current(&self) -> Result<[bool; N], MotorError> {
        self.motors.each_ref().try_map(Motor::is_over_current)
    }

    /// Checks whether any of the motors are drawing over their current limit.
    pub fn is_any_over_current(&self) -> Result<bool, MotorError> {
        Ok(self.is_over_current()?.into_iter().any(identity))
    }

    /// Checks if each motor is over its temperature limit; see
    /// [`Motor::is_over_temp()`].
    pub fn is_over_temp(&self) -> Result<[bool; N], MotorError> {
        self.motors.each_ref().try_map(Motor::is_over_temp)
    }

    /// Checks whether any of the motors are over their temperature limit.
    pub fn is_any_over_temp(&self) -> Result<bool, MotorError> {
        Ok(self.is_over_temp()?.into_iter().any(identity))
    }

    /// Sets the brake mode of all motors in the group; see
    /// [`Motor::set_brake_mode()`].
    pub fn set_brake_mode(&mut self, mode: BrakeMode) -> Result<(), MotorError> {
        for motor in self.motors.iter_mut() {
            motor.set_brake_mode(mode)?;
        }
        Ok(())
    }

    /// Sets the current limit of all motors in the group; see
    /// [`Motor::set_current_limit()`].
    pub fn set_current_limit(&mut self, limit: ElectricCurrent) -> Result<(), MotorError> {
        for motor in self.motors.iter_mut() {
            motor.set_current_limit(limit)?;
        }
        Ok(())
    }

    /// Sets the voltage limit of all motors in the group; see
    /// [`Motor::set_voltage_limit()`].
    pub fn set_voltage_limit(&mut self, limit: ElectricPotential<i32>) -> Result<(), MotorError> {
        for motor in self.motors.iter_mut() {
            motor.set_voltage_limit(limit)?;
        }
        Ok(())
    }

    /// Sets the "absolute" zero position of each motor to its current position.
    pub fn tare_position(&mut self) -> Result<(), MotorError> {
        for motor in self.motors.iter_mut() {
            motor.tare_position()?;
        }
        Ok(())
    }

    /// Returns an iterator over the motors in the group.
    pub fn iter(&self) -> Iter<'_, Motor> {
        self.motors.iter()
    }

    /// Returns a mutable iterator over the motors in the group.
    pub fn iter_mut(&mut self) -> IterMut<'_, Motor> {
        self.motors.iter_mut()
    }
}

impl<Idx, const N: usize> Index<Idx> for MotorGroup<N>
where
    [Motor; N]: Index<Idx>,
{
    type Output = <[Motor; N] as Index<Idx>>::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.motors[index]
    }
}

impl<Idx, const N: usize> IndexMut<Idx> for MotorGroup<N>
where
    [Motor; N]: IndexMut<Idx>,
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.motors[index]
    }
}

impl<const N: usize> DataSource for MotorGroup<N> {
    type Data = [MotorData; N];

    type Error = MotorError;

    fn read(&self) -> Result<Self::Data, Self::Error> {
        self.motors.each_ref().try_map(DataSource::read)
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
            libc::ENXIO => Self::PortOutOfRange,
            libc::ENODEV => Self::PortNotMotor,
            x => Self::Unknown(x),
        }
    }
}

impl From<MotorError> for Error {
    fn from(err: MotorError) -> Self {
        match err {
            MotorError::PortOutOfRange => Error::Custom("port out of range".into()),
            MotorError::PortNotMotor => Error::Custom("port not a motor".into()),
            MotorError::Unknown(n) => Error::System(n),
        }
    }
}

/// Represents possible brake modes for a motor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrakeMode {
    /// Motor coasts when stopped.
    Coast,
    /// Motor brakes when stopped.
    Brake,
    /// Motor holds position when stopped.
    Hold,
}

impl From<BrakeMode> for bindings::motor_brake_mode_e {
    fn from(mode: BrakeMode) -> Self {
        match mode {
            BrakeMode::Coast => bindings::motor_brake_mode_e_E_MOTOR_BRAKE_COAST,
            BrakeMode::Brake => bindings::motor_brake_mode_e_E_MOTOR_BRAKE_BRAKE,
            BrakeMode::Hold => bindings::motor_brake_mode_e_E_MOTOR_BRAKE_HOLD,
        }
    }
}

/// Represents possible gear cartridges for a motor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gearset {
    /// Blue 6:1 Gearset (600RPM).
    SixToOne,
    /// Green 18:1 Gearset (200RPM).
    EighteenToOne,
    /// Red 36:1 Gearset (100RPM).
    ThirtySixToOne,
}

impl From<Gearset> for bindings::motor_gearset_e {
    fn from(gearset: Gearset) -> Self {
        match gearset {
            Gearset::SixToOne => bindings::motor_gearset_e_E_MOTOR_GEARSET_06,
            Gearset::EighteenToOne => bindings::motor_gearset_e_E_MOTOR_GEARSET_18,
            Gearset::ThirtySixToOne => bindings::motor_gearset_e_E_MOTOR_GEARSET_36,
        }
    }
}

/// Represents two possible directions of movement for a robot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// The positive direction.
    Positive,
    /// The negative direction.
    Negative,
}
