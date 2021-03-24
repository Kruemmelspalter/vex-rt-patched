//! Peripherals.

use crate::adi::AdiPort;
use crate::{
    bindings,
    controller::{Controller, ControllerId},
    smart_port::SmartPort,
};

/// A struct which represents all the peripherals on the V5 brain.
pub struct Peripherals {
    /// Primary Controller.
    pub master_controller: Controller,
    /// Partner Controller.
    pub partner_controller: Controller,
    /// Smart Port 1.
    pub port01: SmartPort,
    /// Smart Port 2.
    pub port02: SmartPort,
    /// Smart Port 3.
    pub port03: SmartPort,
    /// Smart Port 4.
    pub port04: SmartPort,
    /// Smart Port 5.
    pub port05: SmartPort,
    /// Smart Port 6.
    pub port06: SmartPort,
    /// Smart Port 7.
    pub port07: SmartPort,
    /// Smart Port 8.
    pub port08: SmartPort,
    /// Smart Port 9.
    pub port09: SmartPort,
    /// Smart Port 10.
    pub port10: SmartPort,
    /// Smart Port 11.
    pub port11: SmartPort,
    /// Smart Port 12.
    pub port12: SmartPort,
    /// Smart Port 13.
    pub port13: SmartPort,
    /// Smart Port 14.
    pub port14: SmartPort,
    /// Smart Port 15.
    pub port15: SmartPort,
    /// Smart Port 16.
    pub port16: SmartPort,
    /// Smart Port 17.
    pub port17: SmartPort,
    /// Smart Port 18.
    pub port18: SmartPort,
    /// Smart Port 19.
    pub port19: SmartPort,
    /// Smart Port 20.
    pub port20: SmartPort,
    /// Smart Port 21.
    pub port21: SmartPort,
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

impl Peripherals {
    /// Constructs a [`Peripherals`] struct unsafely.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the V5's peripherals. You likely want to use the
    /// peripherals object passed into
    /// [`Robot::initialize`](crate::robot::Robot::initialize()) instead.
    pub unsafe fn new() -> Self {
        Peripherals {
            master_controller: Controller::new(ControllerId::Master),
            partner_controller: Controller::new(ControllerId::Partner),
            port01: SmartPort::new(1),
            port02: SmartPort::new(2),
            port03: SmartPort::new(3),
            port04: SmartPort::new(4),
            port05: SmartPort::new(5),
            port06: SmartPort::new(6),
            port07: SmartPort::new(7),
            port08: SmartPort::new(8),
            port09: SmartPort::new(9),
            port10: SmartPort::new(10),
            port11: SmartPort::new(11),
            port12: SmartPort::new(12),
            port13: SmartPort::new(13),
            port14: SmartPort::new(14),
            port15: SmartPort::new(15),
            port16: SmartPort::new(16),
            port17: SmartPort::new(17),
            port18: SmartPort::new(18),
            port19: SmartPort::new(19),
            port20: SmartPort::new(20),
            port21: SmartPort::new(21),
            port_a: AdiPort::new(1, bindings::INTERNAL_ADI_PORT as u8),
            port_b: AdiPort::new(2, bindings::INTERNAL_ADI_PORT as u8),
            port_c: AdiPort::new(3, bindings::INTERNAL_ADI_PORT as u8),
            port_d: AdiPort::new(4, bindings::INTERNAL_ADI_PORT as u8),
            port_e: AdiPort::new(5, bindings::INTERNAL_ADI_PORT as u8),
            port_f: AdiPort::new(6, bindings::INTERNAL_ADI_PORT as u8),
            port_g: AdiPort::new(7, bindings::INTERNAL_ADI_PORT as u8),
            port_h: AdiPort::new(8, bindings::INTERNAL_ADI_PORT as u8),
        }
    }
}
