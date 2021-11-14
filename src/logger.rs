//! A simple console logger

use crate::io::println;
use crate::prelude::Level;
use crate::rtos::free_rtos::FreeRtosConcurrency;
use alloc::string::ToString;
use ansi_rgb::*;
use concurrency_traits::TimeFunctions;
use core::time::Duration;
use log::{set_logger, set_max_level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use rgb::RGB8;

static LOGGER: TerminalLogger = TerminalLogger;

struct TerminalLogger;
impl Log for TerminalLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let color = match record.level() {
            Level::Error => red(),
            Level::Warn => orange(),
            Level::Info => cyan(),
            Level::Debug => RGB8::new(255 / 2, 255 / 2, 255 / 2),
            Level::Trace => black(),
        };
        println!(
            "[{}][{}:{}]{:.3}s: {}",
            record.level().as_str().fg(color),
            record.file_static().unwrap_or("?"),
            record
                .line()
                .map(|val| val.to_string())
                .unwrap_or_else(|| "?".to_string()),
            Duration::from_micros(FreeRtosConcurrency::current_time().as_micros()).as_secs_f64()
                * 1_000_000f64,
            record.args(),
        )
    }

    fn flush(&self) {
        assert_eq!(
            unsafe {
                libc::fflush(libc::fdopen(
                    libc::STDOUT_FILENO,
                    &[b'w', b'\0'] as *const u8,
                ))
            },
            0
        );
    }
}

/// Initializes the logger.
pub fn init_logger(max_level: LevelFilter) -> Result<(), SetLoggerError> {
    set_logger(&LOGGER)?;
    set_max_level(max_level);
    Ok(())
}
