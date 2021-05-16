//! ADIExpander.

use super::port::AdiPort;
use crate::bindings;

/// A struct which represents a V5 ADI expander.
#[derive(Debug)]
pub struct AdiExpander {
    /// ADI Port 1 / A.
    pub port_a: AdiPort,
    /// ADI Port 2 / B.
    pub port_b: AdiPort,
    /// ADI Port 3 / C.
    pub port_c: AdiPort,
    /// ADI Port 4 / D.
    pub port_d: AdiPort,
    /// ADI Port 5 / E.
    pub port_e: AdiPort,
    /// ADI Port 6 / F.
    pub port_f: AdiPort,
    /// ADI Port 7 / G.
    pub port_g: AdiPort,
    /// ADI Port 8 / H.
    pub port_h: AdiPort,
}

impl AdiExpander {
    /// Initializes an ADI expander on a V5 Smart Port
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same ADI expander. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(smart_port: u8) -> Self {
        assert!(
            (1..22).contains(&smart_port) || smart_port == bindings::INTERNAL_ADI_PORT as u8,
            "Cannot construct an ADI port with ADI extender on smart port {}",
            smart_port
        );
        Self {
            port_a: AdiPort::new(1, smart_port),
            port_b: AdiPort::new(2, smart_port),
            port_c: AdiPort::new(3, smart_port),
            port_d: AdiPort::new(4, smart_port),
            port_e: AdiPort::new(5, smart_port),
            port_f: AdiPort::new(6, smart_port),
            port_g: AdiPort::new(7, smart_port),
            port_h: AdiPort::new(8, smart_port),
        }
    }
}
