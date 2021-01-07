#![no_std]
#![no_main]

use vex_rt::{
    self, entry,
    io::println,
    motor::{Gearset, Motor},
    peripherals::Peripherals,
    robot::Robot,
    rtos::{Context, Mutex},
    smart_port::SmartPort,
};

struct DriveTrain {
    left_drive: Motor,
    right_drive: Motor,
}

impl DriveTrain {
    fn new(left_drive_port: SmartPort, right_drive_port: SmartPort) -> Self {
        Self {
            left_drive: left_drive_port.into_motor(Gearset::EighteenToOne, true),
            right_drive: right_drive_port.into_motor(Gearset::EighteenToOne, false),
        }
    }

    fn spin(&mut self) {
        self.left_drive.move_velocity(30).unwrap();
        self.right_drive.move_velocity(-30).unwrap();
    }

    fn stop(&mut self) {
        self.left_drive.move_velocity(0).unwrap();
        self.right_drive.move_velocity(0).unwrap();
    }
}

struct ClawBot {
    drive_train: Mutex<DriveTrain>,
}

impl ClawBot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            drive_train: Mutex::new(DriveTrain::new(peripherals.port01, peripherals.port02)),
        }
    }
}

impl Robot for ClawBot {
    fn initialize() -> Self {
        println!("initialize");
        let p = Peripherals::take().unwrap();
        ClawBot::new(p)
    }

    fn autonomous(&self, _ctx: Context) {
        println!("autonomous");
        let mut drive_train = self.drive_train.lock();
        drive_train.spin();
    }

    fn opcontrol(&self, _ctx: Context) {
        println!("opcontrol");
        let mut drive_train = self.drive_train.lock();
        drive_train.stop();
    }

    fn disabled(&self, _ctx: Context) {
        println!("disabled");
        let mut drive_train = self.drive_train.lock();
        drive_train.stop();
    }
}

entry!(ClawBot);
