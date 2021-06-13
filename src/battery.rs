//! # V5 Battery API.

use crate::{
    bindings,
    error::{get_errno, Error},
};
use uom::si::electric_current::milliampere;
use uom::si::electric_potential::millivolt;
use uom::si::f64::{ElectricCurrent, ElectricPotential, Ratio, ThermodynamicTemperature};
use uom::si::ratio::percent;
use uom::si::thermodynamic_temperature::degree_celsius;

/// A struct which represents a V5 Battery
#[derive(Debug)]
pub struct Battery;

impl Battery {
    /// Gets the current capacity of the battery, as reported by VEXos
    pub fn get_capacity() -> Result<Ratio, BatteryError> {
        unsafe {
            let x = bindings::battery_get_capacity();
            if x == bindings::PROS_ERR_F_ {
                Err(BatteryError::from_errno())
            } else {
                Ok(Ratio::new::<percent>(x))
            }
        }
    }

    /// Gets the current current of the battery, as reported by VEXos
    pub fn get_current() -> Result<ElectricCurrent, BatteryError> {
        match unsafe { bindings::battery_get_current() } {
            bindings::PROS_ERR_ => Err(BatteryError::from_errno()),
            x => Ok(ElectricCurrent::new::<milliampere>(x as f64)),
        }
    }

    /// Gets the current temperature of the battery, as reported by VEXos
    pub fn get_temperature() -> Result<ThermodynamicTemperature, BatteryError> {
        unsafe {
            let x = bindings::battery_get_capacity();
            if x == bindings::PROS_ERR_F_ {
                Err(BatteryError::from_errno())
            } else {
                Ok(ThermodynamicTemperature::new::<degree_celsius>(x))
            }
        }
    }

    /// Gets the current voltage of the battery, as reported by VEXos
    pub fn get_voltage() -> Result<ElectricPotential, BatteryError> {
        match unsafe { bindings::battery_get_voltage() } {
            bindings::PROS_ERR_ => Err(BatteryError::from_errno()),
            x => Ok(ElectricPotential::new::<millivolt>(x as f64)),
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
