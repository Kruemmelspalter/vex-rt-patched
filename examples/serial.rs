#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use array_init::array_init;
use core::time::Duration;
use itertools::Itertools;
use vex_rt::{prelude::*, serial::Serial};

const SINGLE_COUNT: u32 = 100;
const BLOCK_COUNT: u32 = 10;
const BAUDRATES: [i32; 14] = [
    9600, 10800, 12000, 13200, 14400, 15600, 17800, 19200, 38400, 57600, 115200, 230400, 460800,
    921600,
];

struct Interface {
    out_port: Serial,
    in_port: Serial,
}

struct SerialBot {
    interface: Interface,
}

impl Robot for SerialBot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            interface: Interface {
                out_port: peripherals.port01.into_serial(BAUDRATES[0]).unwrap(),
                in_port: peripherals.port02.into_serial(BAUDRATES[0]).unwrap(),
            },
        }
    }

    fn opcontrol(&mut self, _ctx: Context) {
        // Initialize block to send. Bytes are 00 through FF.
        let block: [u8; 256] = array_init(|i| i as u8);

        // Map baudrates to pair of averages.
        let averages = BAUDRATES.map(|rate| {
            println!("----------");
            println!("Baudrate: {}", rate);

            // Set both ports to the current baudrate.
            self.interface.out_port.set_baudrate(rate).unwrap();
            self.interface.in_port.set_baudrate(rate).unwrap();

            // Time one byte SINGLE_COUNT times.
            let mut total = Duration::from_millis(0);
            for j in 0..SINGLE_COUNT {
                // Write byte.
                self.interface
                    .out_port
                    .write_byte((j & 0xFF) as u8)
                    .unwrap();

                // Record current time.
                let start = time_since_start();

                // Read byte.
                while self.interface.in_port.get_read_avail().unwrap() == 0 {}
                let read = self.interface.in_port.read_byte().unwrap();

                // Compute time delta.
                let diff = time_since_start() - start;
                println!("[{:6}, {:2}] = {:2x}, {:?}", rate, j, read, diff);
                total += diff;
            }

            // Compute statistics.
            let single_average = total / SINGLE_COUNT;
            println!("----------");
            println!("Total: {:?}", total);
            println!("Average: {:?}", single_average);

            println!("----------");

            // Time 256 bytes BLOCK_COUNT times.
            let mut total = Duration::from_millis(0);
            for j in 0..BLOCK_COUNT {
                // Write block.
                self.interface.out_port.write(&block).unwrap();

                // Record current time.
                let start = time_since_start();

                // Read block.
                let mut buffer = [0u8; 256];
                let mut k = 0usize;
                while k < 256 {
                    k += self.interface.in_port.read(&mut buffer[k..256]).unwrap();
                }

                // Compute time delta.
                let diff = time_since_start() - start;
                println!(
                    "[{:6}, {:2}] = {:02x}, {:?}",
                    rate,
                    j,
                    buffer.iter().format(""),
                    diff
                );
                total += diff;
            }

            // Compute statistics.
            let block_average = total / BLOCK_COUNT;
            println!("----------");
            println!("Total: {:?}", total);
            println!("Average: {:?}", block_average);

            (rate, single_average, block_average)
        });

        // Print total table.
        println!("---------+-----------------------+-------------------------");
        println!("Baudrate | Average Time (1 byte) | Average Time (256 bytes)");
        println!("---------+-----------------------+-------------------------");
        for (rate, single_average, block_average) in averages.iter() {
            println!(
                "{:8} | {:<21} | {:?}",
                rate,
                format!("{:?}", single_average),
                block_average
            );
        }
    }
}

entry!(SerialBot);
