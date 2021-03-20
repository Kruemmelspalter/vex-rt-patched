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
    pub fn select(&'_ mut self) -> impl Selectable<Result = ()> + '_ {
        struct LoopSelect<'a>(&'a mut Loop);

        struct LoopEvent<'a> {
            l: &'a mut Loop,
            offset: u32,
        }

        impl<'a> Selectable for LoopSelect<'a> {
            const COUNT: u32 = 1;

            type Result = ();

            type Event = LoopEvent<'a>;

            fn listen(self, offset: u32) -> Self::Event {
                LoopEvent { l: self.0, offset }
            }

            fn poll(event: Self::Event, _mask: u32) -> Result<(), Self::Event> {
                if time_since_start() >= event.l.next {
                    event.l.next += event.l.delta;
                    Ok(())
                } else {
                    Err(event)
                }
            }

            fn sleep(event: &Self::Event) -> GenericSleep {
                GenericSleep::Timestamp(event.l.next, 1u32.rotate_left(event.offset))
            }
        }

        LoopSelect(self)
    }
}
