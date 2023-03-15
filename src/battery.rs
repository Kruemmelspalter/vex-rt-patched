//! # V5 Battery API.

use qunit::{
    current::{Current, CurrentExt},
    ratio::{Ratio, RatioExt},
    voltage::{Voltage, VoltageExt},
};

use crate::{
    bindings,
    error::{get_errno, Error},
};
// use uom::si::electric_current::milliampere;
// use uom::si::electric_potential::millivolt;
// use uom::si::f64::{ElectricCurrent, ElectricPotential, Ratio,
// ThermodynamicTemperature}; use uom::si::ratio::percent;
// use uom::si::thermodynamic_temperature::degree_celsius;

/// A struct which represents a V5 Battery
#[derive(Debug)]
pub struct Battery;

impl Battery {
    /// Gets the capacity of the battery.
    pub fn get_capacity() -> Result<Ratio, BatteryError> {
        unsafe {
            let x = bindings::battery_get_capacity();
            if x == bindings::PROS_ERR_F_ {
                Err(BatteryError::from_errno())
            } else {
                Ok(x.percent())
            }
        }
    }

    /// Gets the current draw of the battery.
    pub fn get_current() -> Result<Current, BatteryError> {
        match unsafe { bindings::battery_get_current() } {
            bindings::PROS_ERR_ => Err(BatteryError::from_errno()),
            x => Ok((x as f64).mA()),
        }
    }

    /// Gets the current temperature of the battery, in degrees Celsius.
    pub fn get_temperature() -> Result<f64, BatteryError> {
        unsafe {
            let x = bindings::battery_get_temperature();
            if x == bindings::PROS_ERR_F_ {
                Err(BatteryError::from_errno())
            } else {
                Ok(x)
            }
        }
    }

    /// Gets the current voltage of the battery.
    pub fn get_voltage() -> Result<Voltage, BatteryError> {
        match unsafe { bindings::battery_get_voltage() } {
            bindings::PROS_ERR_ => Err(BatteryError::from_errno()),
            x => Ok((x as f64).mV()),
        }
    }
}

/// Represents possible errors for battery operations.
#[derive(Debug)]
pub enum BatteryError {
    /// Another resource is currently trying to access the battery.
    BatteryBusy,
    /// Unknown error.
    Unknown(i32),
}

impl BatteryError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::EACCES => Self::BatteryBusy,
            x => Self::Unknown(x),
        }
    }
}

impl From<BatteryError> for Error {
    fn from(err: BatteryError) -> Self {
        match err {
            BatteryError::BatteryBusy => Error::Custom("battery is busy".into()),
            BatteryError::Unknown(n) => Error::System(n),
        }
    }
}
