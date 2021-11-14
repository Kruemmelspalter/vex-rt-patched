use crate::bindings;
use core::convert::TryInto;
use core::fmt;
use core::fmt::{Debug, Display, Formatter};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use core::time::Duration;

/// Represents a time on a monotonically increasing clock (i.e., time since
/// program start).
///
/// This type has a precision of 1 millisecond.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(u64);

impl Instant {
    #[inline]
    /// Creates a new `Instant` from the specified number of *whole*
    /// microseconds since program start.
    pub fn from_micros(micros: u64) -> Self {
        Self(micros)
    }

    #[inline]
    /// Creates a new `Instant` from the specified number of whole milliseconds
    /// since program start.
    pub fn from_millis(millis: u64) -> Self {
        Self(
            millis
                .checked_mul(1000)
                .expect("overflow when creating instant from seconds"),
        )
    }

    /// Creates a new `Instant` from the specified number of whole seconds since
    /// program start.
    pub fn from_secs(secs: u64) -> Self {
        Self(
            secs.checked_mul(1000000)
                .expect("overflow when creating instant from seconds"),
        )
    }

    #[inline]
    /// Returns the number of *whole* seconds since program start contained by
    /// this `Instant`.
    ///
    /// The returned value does not include the fractional (milliseconds) part
    /// of the time value.
    pub const fn as_millis(&self) -> u64 {
        self.0 / 1000
    }

    #[inline]
    /// Returns the number of *whole* seconds since program start contained by
    /// this `Instant`.
    pub const fn as_secs(&self) -> u64 {
        self.0 / 1000000
    }

    #[inline]
    /// Returns the number of whole microseconds since program start contained
    /// by this `Instant`.
    pub const fn as_micros(&self) -> u64 {
        self.0
    }

    #[inline]
    /// Returns the fractional part of this `Instant`, in *whole* milliseconds.
    ///
    /// This method does **not** return the time value in milliseconds. The
    /// returned number always represents a fractional portion of a second
    /// (i.e., it is less than one thousand).
    pub fn subsec_millis(&self) -> u64 {
        self.0 % 1000000 / 1000
    }

    #[inline]
    /// Returns the fractional part of this `Instant`, in whole microseconds.
    ///
    /// This method does **not** return the time value in microseconds. The
    /// returned number always represents a fractional portion of a second
    /// (i.e., it is less than one million).
    pub fn subsec_micros(&self) -> u64 {
        self.0 % 1000000
    }

    #[inline]
    /// Checked addition of a [`Duration`] to an `Instant`. Computes `self +
    /// rhs`, returning [`None`] if overflow occurred.
    pub fn checked_add(self, rhs: Duration) -> Option<Self> {
        Some(Self(self.0.checked_add(rhs.as_micros().try_into().ok()?)?))
    }

    #[inline]
    /// Checked subtraction of a [`Duration`] from an `Instant`. Computes
    /// `self - rhs`, returning [`None`] if the result would be negative or
    /// overflow occurred.
    pub fn checked_sub(self, rhs: Duration) -> Option<Instant> {
        Some(Self(self.0.checked_sub(rhs.as_micros().try_into().ok()?)?))
    }

    #[inline]
    /// Checked subtraction of two `Instant`s. Computes `self - rhs`, returning
    /// [`None`] if the result would be negative or overflow occurred.
    pub fn checked_sub_instant(self, rhs: Self) -> Option<Duration> {
        Some(Duration::from_micros(self.0.checked_sub(rhs.0)?))
    }

    #[inline]
    /// Checked multiplication of an `Instant` by a scalar. Computes `self *
    /// rhs`, returning [`None`] if an overflow occurred.
    pub fn checked_mul(self, rhs: u64) -> Option<Instant> {
        Some(Self(self.0.checked_mul(rhs)?))
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        self.checked_add(rhs)
            .expect("overflow when adding duration to instant")
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, rhs: Duration) -> Self::Output {
        self.checked_sub(rhs)
            .expect("overflow when subtracting duration from instant")
    }
}

impl Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub_instant(rhs)
            .expect("overflow when subtracting instants")
    }
}

impl Mul<u64> for Instant {
    type Output = Instant;

    fn mul(self, rhs: u64) -> Self::Output {
        self.checked_mul(rhs)
            .expect("overflow when multiplying instant by scalar")
    }
}

impl Div<u64> for Instant {
    type Output = Instant;

    #[inline]
    fn div(self, rhs: u64) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl MulAssign<u64> for Instant {
    fn mul_assign(&mut self, rhs: u64) {
        *self = *self * rhs;
    }
}

impl DivAssign<u64> for Instant {
    fn div_assign(&mut self, rhs: u64) {
        *self = *self / rhs;
    }
}

impl Debug for Instant {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:06}s", self.0 / 1000000, self.0 % 1000000)
    }
}

impl Display for Instant {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:06}s", self.0 / 1000000, self.0 % 1000000)
    }
}

#[inline]
/// Gets the current timestamp (i.e., the time which has passed since program
/// start).
pub fn time_since_start() -> Instant {
    Instant::from_micros(unsafe { bindings::micros() })
}
