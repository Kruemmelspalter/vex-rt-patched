//! SmartPort.

use crate::motor::{EncoderUnits, Gearset, Motor};
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
    /// mutable references to a V5 smart port. You likely want to use
    /// [`Peripherals::take()`](crate::peripherals::Peripherals::take())
    /// instead.
    pub unsafe fn new(port: u8) -> Self {
        assert!(
            (1..22).contains(&port),
            "Cannot construct a smart port on port {}",
            port
        );
        Self { port }
    }

    /// Converts a `SmartPort` into a [`Motor`](crate::motor::Motor).
    ///
    /// # Examples
    ///
    /// ```
    /// use vex_rt as rt;
    /// let peripherals = rt::Peripherals::take();
    /// let gearset = rt::Gearset::ThirtySixToOne;
    /// let is_reversed = false;
    /// let motor01 = peripherals.port01.as_motor(gearset, is_reversed);
    /// ```
    pub fn into_motor(self, gearset: Gearset, encoder_units: EncoderUnits, reverse: bool) -> Motor {
        unsafe { Motor::new(self.port, gearset, encoder_units, reverse) }
    }
}
