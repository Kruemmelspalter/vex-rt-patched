//! ADIPort.

use super::{
    AdiAnalog, AdiAnalogError, AdiDigitalInput, AdiDigitalInputError, AdiDigitalOutput,
    AdiDigitalOutputError, AdiEncoder, AdiEncoderError, AdiGyro, AdiGyroError, AdiUltrasonic,
    AdiUltrasonicError,
};

use crate::bindings;
use core::cmp::Ordering;
use core::convert::{TryFrom, TryInto};

/// A struct which represents an unconfigured ADI port.
pub struct AdiPort {
    port: u8,
    expander_port: u8,
}

impl AdiPort {
    /// Constructs a new ADI port.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to an V5 ADI port. You likely want to implement
    /// [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8, expander_port: u8) -> Self {
        assert!(
            (1..9).contains(&port),
            "Cannot construct an ADI port on port {}",
            port
        );
        assert!(
            (1..22).contains(&expander_port) || expander_port == bindings::INTERNAL_ADI_PORT as u8,
            "Cannot construct an ADI port with ADI expander on smart port {}",
            expander_port
        );
        Self {
            port,
            expander_port,
        }
    }

    /// Turns this port into an ADI analog input.
    #[inline]
    pub fn into_adi_analog(self) -> Result<AdiAnalog, AdiAnalogError> {
        self.try_into()
    }

    /// Turns this port into an ADI digital input.
    #[inline]
    pub fn into_adi_digital_input(self) -> Result<AdiDigitalInput, AdiDigitalInputError> {
        self.try_into()
    }
    /// Turns this port into an ADI digital output.
    #[inline]
    pub fn into_adi_digital_output(self) -> Result<AdiDigitalOutput, AdiDigitalOutputError> {
        self.try_into()
    }

    /// Turns this and another port into an ADI encoder.
    #[inline]
    pub fn into_adi_encoder(self, bottom: Self) -> Result<AdiEncoder, AdiEncoderError> {
        (self, bottom).try_into()
    }

    /// Turns this port into an ADI gyro.
    #[inline]
    pub fn into_adi_gyro(self, multiplier: f64) -> Result<AdiGyro, AdiGyroError> {
        (self, multiplier).try_into()
    }

    /// Turns this and another port into an ADI ultrasonic sensor.
    #[inline]
    pub fn into_adi_ultrasonic(self, bottom: Self) -> Result<AdiUltrasonic, AdiUltrasonicError> {
        (self, bottom).try_into()
    }
}

impl TryFrom<AdiPort> for AdiAnalog {
    type Error = AdiAnalogError;

    /// Converts a `AdiPort` into a [`AdiAnalog`].
    fn try_from(port: AdiPort) -> Result<Self, Self::Error> {
        unsafe { AdiAnalog::new(port.port, port.expander_port) }
    }
}

impl TryFrom<AdiPort> for AdiDigitalInput {
    type Error = AdiDigitalInputError;

    /// Converts a `AdiPort` into a [`AdiDigitalInput`].
    fn try_from(port: AdiPort) -> Result<Self, Self::Error> {
        unsafe { AdiDigitalInput::new(port.port, port.expander_port) }
    }
}

impl TryFrom<AdiPort> for AdiDigitalOutput {
    type Error = AdiDigitalOutputError;

    /// Converts a `AdiPort` into a [`AdiDigitalOutput`].
    fn try_from(port: AdiPort) -> Result<Self, Self::Error> {
        unsafe { AdiDigitalOutput::new(port.port, port.expander_port) }
    }
}

impl TryFrom<(AdiPort, AdiPort)> for AdiEncoder {
    type Error = AdiEncoderError;

    /// Converts an `(AdiPort, AdiPort)` into an
    /// [`AdiEncoder`](crate::adi::AdiEncoder).
    fn try_from(ports: (AdiPort, AdiPort)) -> Result<Self, Self::Error> {
        if ports.0.expander_port != ports.1.expander_port {
            return Err(AdiEncoderError::PortNonMatchingExtenders);
        }

        let top_port;
        let bottom_port;
        let reversed;

        match ports.0.port.cmp(&ports.1.port) {
            Ordering::Less => {
                top_port = ports.0;
                bottom_port = ports.1;
                reversed = false;
            }
            Ordering::Greater => {
                top_port = ports.1;
                bottom_port = ports.0;
                reversed = true;
            }
            Ordering::Equal => return Err(AdiEncoderError::PortsOutOfRange),
        }

        if bottom_port.port - top_port.port != 1 || bottom_port.port % 2 != 0 {
            return Err(AdiEncoderError::PortsOutOfRange);
        }

        unsafe {
            AdiEncoder::new(
                top_port.port,
                bottom_port.port,
                reversed,
                top_port.expander_port,
            )
        }
    }
}

impl TryFrom<(AdiPort, f64)> for AdiGyro {
    type Error = AdiGyroError;

    #[inline]
    fn try_from(port_multiplier: (AdiPort, f64)) -> Result<Self, Self::Error> {
        unsafe {
            AdiGyro::new(
                port_multiplier.0.port,
                port_multiplier.1,
                port_multiplier.0.expander_port,
            )
        }
    }
}

impl TryFrom<(AdiPort, AdiPort)> for AdiUltrasonic {
    type Error = AdiUltrasonicError;

    fn try_from(ports: (AdiPort, AdiPort)) -> Result<Self, Self::Error> {
        if ports.0.expander_port != ports.1.expander_port {
            Err(AdiUltrasonicError::PortNonMatchingExtenders)
        } else {
            unsafe { AdiUltrasonic::new(ports.0.port, ports.1.port, ports.0.expander_port) }
        }
    }
}
