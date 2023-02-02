use core::time::Duration;

use super::{time_since_start, GenericSleep, Instant, Selectable, Task};

/// Provides a constant-period looping construct.
pub struct Loop {
    delta: Duration,
    next: Instant,
    cycle: usize,
}

impl Loop {
    #[inline]
    /// Creates a new loop object with a given period.
    pub fn new(delta: Duration) -> Self {
        Loop {
            delta,
            next: time_since_start() + delta,
            cycle: 0,
        }
    }

    /// Delays until the next loop cycle.
    pub fn delay(&mut self) {
        if let Some(d) = self.next.checked_sub_instant(time_since_start()) {
            Task::delay(d);
        }
        self.next += self.delta;
        self.cycle += 1;
    }

    #[inline]
    /// Returns the current cycle index.
    ///
    /// This is initially 0 and increments each cycle (i.e., when
    /// [`delay()`](Self::delay()) returns or `select` successfully
    /// completes).
    pub fn cycle(&self) -> usize {
        self.cycle
    }

    #[inline]
    /// Helper function to check whether the cycle index (see
    /// [`cycle()`](Self::cycle())) is a multiple of `modulus`.
    pub fn is_mod(&self, modulus: usize) -> bool {
        self.cycle % modulus == 0
    }

    #[inline]
    /// A [`Selectable`] event which occurs at the next loop cycle.
    pub fn select(&'_ mut self) -> impl Selectable<Output = ()> + '_ {
        #[repr(transparent)]
        struct LoopSelect<'a>(&'a mut Loop);

        impl<'a> Selectable for LoopSelect<'a> {
            type Output = ();

            fn poll(self) -> Result<Self::Output, Self> {
                if time_since_start() >= self.0.next {
                    self.0.next += self.0.delta;
                    self.0.cycle += 1;
                    Ok(())
                } else {
                    Err(self)
                }
            }

            fn sleep(&self) -> GenericSleep {
                GenericSleep::Timestamp(self.0.next)
            }
        }

        LoopSelect(self)
    }
}
