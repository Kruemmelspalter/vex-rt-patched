//! API for using smart ports as generic serial ports.

use core::convert::TryInto;

use crate::{
    bindings,
    error::{Error, SentinelError},
};

/// Represents the generic serial interface of a smart port.
pub struct Serial(u8);

impl Serial {
    /// Constructs a new generic serial port. Panics on failure; see
    /// [`Serial::try_new()`].
    ///
    /// # Safety
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same smart port interface. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8, baudrate: i32) -> Self {
        Self::try_new(port, baudrate)
            .unwrap_or_else(|err| panic!("failed to create generic serial port: {}", err))
    }

    /// Constructs a new generic serial port.
    ///
    /// # Safety
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to the same smart port interface. You likely want to
    /// implement [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn try_new(port: u8, baudrate: i32) -> Result<Self, Error> {
        bindings::serial_enable(port).check()?;
        bindings::serial_set_baudrate(port, baudrate).check()?;
        Ok(Self(port))
    }

    #[inline]
    /// Changes the baudrate of the serial port.
    pub fn set_baudrate(&mut self, baudrate: i32) -> Result<(), Error> {
        unsafe { bindings::serial_set_baudrate(self.0, baudrate) }.check()?;
        Ok(())
    }

    #[inline]
    /// Gets the number of bytes available to read in the input buffer of the
    /// serial port.
    pub fn get_read_avail(&self) -> Result<usize, Error> {
        Ok(unsafe { bindings::serial_get_read_avail(self.0) }
            .check()?
            .try_into()?)
    }

    #[inline]
    /// Gets the number of bytes free in the output buffer of the serial port.
    pub fn get_write_free(&self) -> Result<usize, Error> {
        Ok(unsafe { bindings::serial_get_write_free(self.0) }
            .check()?
            .try_into()?)
    }

    #[inline]
    /// Reads the next available byte in the input buffer of the serial port
    /// without removing it.
    pub fn peek_byte(&self) -> Result<u8, Error> {
        Ok(unsafe { bindings::serial_peek_byte(self.0) }
            .check()?
            .try_into()?)
    }

    #[inline]
    /// Reads the next available byte in the input buffer of the serial port.
    pub fn read_byte(&mut self) -> Result<u8, Error> {
        Ok(unsafe { bindings::serial_read_byte(self.0) }
            .check()?
            .try_into()?)
    }

    #[inline]
    /// Writes the given byte to the output buffer of the serial port.
    pub fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        unsafe { bindings::serial_write_byte(self.0, byte) }.check()?;
        Ok(())
    }

    #[inline]
    /// Reads as many bytes as possible from the input buffer of the serial port
    /// into the given buffer, returning the number read.
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        Ok(
            unsafe { bindings::serial_read(self.0, buffer.as_mut_ptr(), buffer.len().try_into()?) }
                .check()?
                .try_into()?,
        )
    }

    #[inline]
    /// Writes as many bytes as possible to the output buffer of the serial port
    /// from the given buffer, returning the number written.
    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        Ok(unsafe {
            bindings::serial_write(self.0, buffer.as_ptr() as *mut _, buffer.len().try_into()?)
        }
        .check()?
        .try_into()?)
    }

    #[inline]
    /// Clears the internal input and output buffers of the serial port,
    /// effectively resetting its state.
    pub fn flush(&mut self) -> Result<(), Error> {
        unsafe { bindings::serial_flush(self.0) }.check()?;
        Ok(())
    }
}
