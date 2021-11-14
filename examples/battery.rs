#![no_std]
#![no_main]

use vex_rt::prelude::*;

struct BatteryBot;

#[async_trait(?Send)]
impl Robot for BatteryBot {
    async fn new(_peripherals: Peripherals) -> Self {
        Self
    }

    async fn opcontrol(&'static self, _robot_args: RobotArgs) {
        println!("Battery Capacity: {:?}", Battery::get_current().unwrap());
        println!("Battery Current: {:?}", Battery::get_capacity().unwrap());
        println!(
            "Battery Temperature: {:?}",
            Battery::get_temperature().unwrap()
        );
        println!("Battery Voltage: {:?}", Battery::get_voltage().unwrap());
    }
}

entry!(BatteryBot);
