#![no_std]
#![no_main]

use core::convert::TryInto;
use core::time::Duration;
use vex_rt::prelude::*;

struct DriveTrain {
    encoder: AdiEncoder,
}

impl DriveTrain {
    fn new(encoder_port_left: AdiPort, encoder_port_right: AdiPort) -> Self {
        Self {
            encoder: (encoder_port_left, encoder_port_right).try_into().unwrap(),
        }
    }
}

struct ClawBot {
    drive_train: VexAsyncMutex<DriveTrain>,
}

#[async_trait(?Send)]
impl Robot for ClawBot {
    async fn new(peripherals: Peripherals) -> Self {
        Self {
            drive_train: VexAsyncMutex::new(DriveTrain::new(
                peripherals.port_a,
                peripherals.port_b,
            )),
        }
    }

    async fn autonomous(&'static self, robot_args: RobotArgs) {
        println!("autonomous");
        let drive_train = self.drive_train.lock_async().await;
        async_loop!(robot_args: (Duration::from_millis(20)){
            println!("{}", drive_train.encoder.get().unwrap());
        });
    }
}

entry!(ClawBot);
