use core::time::Duration;

use super::{time_since_start, GenericSleep, Instant, Selectable, Task};

/// Provides a constant-period looping construct.
pub struct Loop {
    delta: Duration,
    next: Instant,
}

impl Loop {
    #[inline]
    /// Creates a new loop object with a given period.
    pub fn new(delta: Duration) -> Self {
        Loop {
            delta,
            next: time_since_start() + delta,
        }
    }

    /// Delays until the next loop cycle.
    pub fn delay(&mut self) {
        if let Some(d) = self.next.checked_sub_instant(time_since_start()) {
            Task::delay(d);
        }
        self.next += self.delta;
    }

    #[inline]
    /// A [`Selectable`] event which occurs at the next loop cycle.
    pub fn select(&'_ mut self) -> impl Selectable + '_ {
        struct LoopSelect<'a>(&'a mut Loop);

        impl<'a> Selectable for LoopSelect<'a> {
            fn poll(self) -> Result<(), Self> {
                if time_since_start() >= self.0.next {
                    self.0.next += self.0.delta;
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
