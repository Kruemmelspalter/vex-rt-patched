pub struct Peripherals {
    pub port1: crate::SmartPort,
    pub port2: crate::SmartPort,
    pub port3: crate::SmartPort,
    pub port4: crate::SmartPort,
    pub port5: crate::SmartPort,
    pub port6: crate::SmartPort,
    pub port7: crate::SmartPort,
    pub port8: crate::SmartPort,
    pub port9: crate::SmartPort,
    pub port10: crate::SmartPort,
    pub port11: crate::SmartPort,
    pub port12: crate::SmartPort,
    pub port13: crate::SmartPort,
    pub port14: crate::SmartPort,
    pub port15: crate::SmartPort,
    pub port16: crate::SmartPort,
    pub port17: crate::SmartPort,
    pub port18: crate::SmartPort,
    pub port19: crate::SmartPort,
    pub port20: crate::SmartPort,
    pub port21: crate::SmartPort,
}

static mut PERIPHERALS_TAKEN: bool = false;

impl Peripherals {
    pub fn take() -> Self {
        if unsafe { PERIPHERALS_TAKEN } {
            panic!("Peripherals::take() can be called only once.")
        } else {
            unsafe {
                PERIPHERALS_TAKEN = true;
                Self::steal()
            }
        }
    }

    pub unsafe fn steal() -> Self {
        Peripherals {
            port1: crate::SmartPort::new(1),
            port2: crate::SmartPort::new(2),
            port3: crate::SmartPort::new(3),
            port4: crate::SmartPort::new(4),
            port5: crate::SmartPort::new(5),
            port6: crate::SmartPort::new(6),
            port7: crate::SmartPort::new(7),
            port8: crate::SmartPort::new(8),
            port9: crate::SmartPort::new(9),
            port10: crate::SmartPort::new(10),
            port11: crate::SmartPort::new(11),
            port12: crate::SmartPort::new(12),
            port13: crate::SmartPort::new(13),
            port14: crate::SmartPort::new(14),
            port15: crate::SmartPort::new(15),
            port16: crate::SmartPort::new(16),
            port17: crate::SmartPort::new(17),
            port18: crate::SmartPort::new(18),
            port19: crate::SmartPort::new(19),
            port20: crate::SmartPort::new(20),
            port21: crate::SmartPort::new(21),
        }
    }
}