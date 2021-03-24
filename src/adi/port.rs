//! ADIPort.

use super::encoder::{AdiEncoder, AdiEncoderError};

use crate::bindings;
use core::cmp::Ordering;
use core::convert::TryFrom;

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
