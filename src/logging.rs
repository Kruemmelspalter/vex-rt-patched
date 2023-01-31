//! Support for logging via the [log](https://docs.rs/log/*/log/) crate.

#![cfg(feature = "logging")]
#![cfg_attr(docsrs, doc(cfg(feature = "logging")))]

use alloc::format;
use libc_print::libc_ewrite;
use log::{info, set_logger, set_max_level, LevelFilter, Log, SetLoggerError};
use spin::Once;

use crate::rtos::{time_since_start, Mutex};

static LOGGER: Once<StderrLogger> = Once::INIT;

pub(crate) struct StderrLogger {
    level: LevelFilter,
    mtx: Mutex<()>,
}

impl StderrLogger {
    pub(crate) fn init_stderr(level: LevelFilter) -> Result<(), SetLoggerError> {
        set_logger(LOGGER.call_once(|| Self {
            level,
            mtx: Mutex::new(()),
        }))?;
        set_max_level(level);
        info!("Initialized logging at level {}", level);
        Ok(())
    }
}

impl Log for StderrLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let line = format!(
                "{} {} [{}] {}\n",
                time_since_start(),
                record.level(),
                record.target(),
                record.args(),
            );
            let _lock = self.mtx.lock();
            libc_ewrite!(line.as_str());
        }
    }

    fn flush(&self) {}
}
