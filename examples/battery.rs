#![no_std]
#![no_main]

use vex_rt::prelude::*;

struct BatteryBot {}

impl Robot for BatteryBot {
    fn new(_peripherals: Peripherals) -> Self {
        Self {}
    }

    fn opcontrol(&mut self, _ctx: Context) {
        println!("Battery Capacity: {}", Battery::get_capacity().unwrap());
        println!("Battery Current: {}", Battery::get_current().unwrap());
        println!(
            "Battery Temperature: {}",
            Battery::get_temperature().unwrap()
        );
        println!("Battery Voltage: {}", Battery::get_voltage().unwrap());
    }
}

entry!(BatteryBot);
