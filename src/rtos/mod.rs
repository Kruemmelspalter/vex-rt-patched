//! Multitasking primitives.

use alloc::{boxed::Box, format, string::String};
use core::{
    convert::TryInto,
    fmt::{self, Debug, Display, Formatter},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
    time::Duration,
};
use cstring_interop::{from_cstring_raw, with_cstring};
use libc::c_void;

use crate::{
    bindings,
    error::{Error, SentinelError},
};

const TIMEOUT_MAX: u32 = 0xffffffff;

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
    pub fn as_millis(&self) -> u64 {
        self.0 / 1000
    }

    #[inline]
    /// Returns the number of *whole* seconds since program start contained by
    /// this `Instant`.
    pub fn as_secs(&self) -> u64 {
        self.0 / 1000000
    }

    #[inline]
    /// Returns the number of whole microseconds since program start contained
    /// by this `Instant`.
    pub fn as_micros(&self) -> u64 {
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
/// Represents a FreeRTOS task.
pub struct Task(bindings::task_t);

impl Task {
    /// The default priority for new tasks.
    pub const DEFAULT_PRIORITY: u32 = bindings::TASK_PRIORITY_DEFAULT;

    /// The default stack depth for new tasks.
    pub const DEFAULT_STACK_DEPTH: u16 = bindings::TASK_STACK_DEPTH_DEFAULT as u16;

    #[inline]
    /// Delays the current task by the specified duration.
    pub fn delay(dur: Duration) {
        unsafe {
            bindings::task_delay(dur.as_millis() as u32);
        }
    }

    #[inline]
    /// Gets the current task.
    pub fn current() -> Task {
        Task(unsafe { bindings::task_get_current() })
    }

    /// Finds a task by its name.
    pub fn find_by_name(name: &str) -> Result<Task, Error> {
        let ptr = (with_cstring(name.into(), |name| unsafe {
            bindings::task_get_by_name(name.into_raw()).check()
        }) as Result<*mut c_void, Error>)?;
        if ptr.is_null() {
            Err(Error::Custom(format!("task not found: {}", name)))
        } else {
            Ok(Task(ptr))
        }
    }

    #[inline]
    /// Spawns a new task with no name and the default priority and stack depth.
    pub fn spawn<F>(f: F) -> Result<Task, Error>
    where
        F: FnOnce() + Send + 'static,
    {
        Task::spawn_ext("", Self::DEFAULT_PRIORITY, Self::DEFAULT_STACK_DEPTH, f)
    }

    /// Spawns a new task with the specified name, priority and stack depth.
    pub fn spawn_ext<F>(name: &str, priority: u32, stack_depth: u16, f: F) -> Result<Task, Error>
    where
        F: FnOnce() + Send + 'static,
    {
        extern "C" fn run<F: FnOnce()>(arg: *mut libc::c_void) {
            let cb_box: Box<F> = unsafe { Box::from_raw(arg as *mut F) };
            cb_box()
        }

        let cb = Box::new(f);
        unsafe {
            let arg = Box::into_raw(cb);
            let r = Task::spawn_raw(name, priority, stack_depth, run::<F>, arg as *mut _);
            if r.is_err() {
                // We need to re-box the pointer if the task could not be created, to avoid a
                // memory leak.
                Box::from_raw(arg);
            }
            r
        }
    }

    #[inline]
    /// Spawns a new task from a C function pointer and an arbitrary data
    /// pointer.
    pub fn spawn_raw(
        name: &str,
        priority: u32,
        stack_depth: u16,
        f: unsafe extern "C" fn(arg1: *mut libc::c_void),
        arg: *mut libc::c_void,
    ) -> Result<Task, Error> {
        with_cstring(name.into(), |cname| {
            Ok(Task(
                unsafe {
                    bindings::task_create(Some(f), arg, priority, stack_depth, cname.into_raw())
                }
                .check()?,
            ))
        })
    }

    #[inline]
    /// Gets the name of the task.
    pub fn name(&self) -> String {
        unsafe { from_cstring_raw(bindings::task_get_name(self.0)) }
    }

    #[inline]
    /// Gets the priority of the task.
    pub fn priority(&self) -> u32 {
        unsafe { bindings::task_get_priority(self.0) }
    }

    #[inline]
    /// Gets the state of the task.
    pub fn state(&self) -> TaskState {
        match unsafe { bindings::task_get_state(self.0) } {
            bindings::task_state_e_t_E_TASK_STATE_RUNNING => TaskState::Running,
            bindings::task_state_e_t_E_TASK_STATE_READY => TaskState::Ready,
            bindings::task_state_e_t_E_TASK_STATE_BLOCKED => TaskState::Blocked,
            bindings::task_state_e_t_E_TASK_STATE_SUSPENDED => TaskState::Suspended,
            bindings::task_state_e_t_E_TASK_STATE_DELETED => TaskState::Deleted,
            bindings::task_state_e_t_E_TASK_STATE_INVALID => {
                panic!("invalid task handle: {:#010x}", self.0 as usize)
            }
            x => panic!("bindings::task_get_state returned unexpected value: {}", x),
        }
    }

    #[inline]
    /// Unsafely deletes the task.
    ///
    /// # Safety
    /// This is unsafe because it does not guarantee that the task's code safely
    /// unwinds (i.e., that destructors are called, memory is freed and other
    /// resources are released).
    pub unsafe fn delete(&self) {
        bindings::task_delete(self.0)
    }
}

impl Debug for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task")
            .field("name", &self.name())
            .field("priority", &self.priority())
            .finish()
    }
}

unsafe impl Send for Task {}

unsafe impl Sync for Task {}

/// Represents the state of a [`Task`].
pub enum TaskState {
    /// The task is actively executing.
    Running,
    /// The task exists and is available to run, but is not currently running.
    Ready,
    /// The task is delayed or blocked by a mutex, semaphore or I/O operation.
    Blocked,
    /// The task is suspended.
    Suspended,
    /// The task has been deleted.
    Deleted,
}

#[derive(Copy, Clone, Debug)]
/// Represents a future time to sleep until.
pub enum GenericSleep {
    /// Represents a future time when a notification occurs. If a timestamp is
    /// present, then it represents whichever is earlier.
    NotifyTake(Option<(Instant, u32)>),
    /// Represents an explicit future timestamp.
    Timestamp(Instant, u32),
}

impl GenericSleep {
    /// Sleeps until the future time represented by `self`. The result is the
    /// number of notifications which were present, if the sleep ended due to
    /// notification.
    pub fn sleep(self) -> u32 {
        match self {
            GenericSleep::NotifyTake(timeout) => {
                let timeout = timeout.map_or(TIMEOUT_MAX, |(t, k)| {
                    t.checked_sub_instant(time_since_start())
                        .map_or(0, |d| d.as_millis() as u32)
                });
                unsafe { bindings::task_notify_take(true, timeout) }
            }
            GenericSleep::Timestamp(t, k) => {
                if let Some(d) = t.checked_sub_instant(time_since_start()) {
                    Task::delay(d);
                }
                k
            }
        }
    }

    #[inline]
    /// Get the timestamp represented by `self`, if it is present.
    pub fn timeout(self) -> Option<(Instant, u32)> {
        match self {
            GenericSleep::NotifyTake(v) => v,
            GenericSleep::Timestamp(t, k) => Some((t, k)),
        }
    }

    /// Combine two `GenericSleep` objects to one which represents the earliest
    /// possible time of the two.
    pub fn combine(self, other: Self) -> Self {
        fn combine_time(t1: Instant, k1: u32, t2: Instant, k2: u32) -> (Instant, u32) {
            match t1.cmp(&t2) {
                core::cmp::Ordering::Less => (t1, k1),
                core::cmp::Ordering::Equal => (t1, k1 | k2),
                core::cmp::Ordering::Greater => (t2, k2),
            }
        }

        match (self, other) {
            (GenericSleep::Timestamp(t1, k1), GenericSleep::Timestamp(t2, k2)) => {
                let (t, k) = combine_time(t1, k1, t2, k2);
                GenericSleep::Timestamp(t, k)
            }
            (a, b) => GenericSleep::NotifyTake(a.timeout().map_or(b.timeout(), |(t1, k1)| {
                Some(
                    b.timeout()
                        .map_or((t1, k1), |(t2, k2)| combine_time(t1, k1, t2, k2)),
                )
            })),
        }
    }
}

/// Represents a future event which can be used with the [`select!`] macro.
pub trait Selectable: Sized {
    const COUNT: u32;
    type Result;
    type Event;

    fn listen(self, offset: u32) -> Self::Event;

    /// Processes the event if it is ready, consuming the event object;
    /// otherwise, it provides a replacement event object.
    fn poll(event: Self::Event, mask: u32) -> Result<Self::Result, Self::Event>;

    /// Gets the earliest time that the event could be ready.
    fn sleep(event: &Self::Event) -> GenericSleep;
}

#[inline]
/// Creates a new [`Selectable`] event by mapping the result of a given one.
pub fn select_map<'a, T: 'a, U: 'a>(
    upstream: impl Selectable<Result = T> + 'a,
    f: impl 'a + FnOnce(T) -> U,
) -> impl Selectable<Result = U> + 'a {
    struct MapSelect<U, E: Selectable, F: FnOnce(E::Result) -> U> {
        upstream: E,
        f: F,
    }

    struct MapEvent<U, E: Selectable, F: FnOnce(E::Result) -> U> {
        upstream: E::Event,
        f: F,
    }

    impl<U, E: Selectable, F: FnOnce(E::Result) -> U> Selectable for MapSelect<U, E, F> {
        const COUNT: u32 = E::COUNT;

        type Result = U;

        type Event = MapEvent<U, E, F>;

        fn listen(self, offset: u32) -> Self::Event {
            Self::Event {
                upstream: self.upstream.listen(offset),
                f: self.f,
            }
        }

        fn poll(event: Self::Event, mask: u32) -> Result<U, Self::Event> {
            match E::poll(event.upstream, mask) {
                Ok(r) => Ok((event.f)(r)),
                Err(upstream) => Err(Self::Event { upstream, ..event }),
            }
        }

        fn sleep(event: &Self::Event) -> GenericSleep {
            E::sleep(&event.upstream)
        }
    }

    MapSelect { upstream, f }
}

#[inline]
/// Creates a new [`Selectable`] event which processes exactly one of the given
/// events.
pub fn select_either<'a, T: 'a>(
    fst: impl Selectable<Result = T> + 'a,
    snd: impl Selectable<Result = T> + 'a,
) -> impl Selectable<Result = T> + 'a {
    struct EitherSelect<E1, E2>(E1, E2);

    struct EitherEvent<E1: Selectable, E2: Selectable>(E1::Event, E2::Event);

    const fn generate_mask(count: u32) -> u32 {
        if count < 32 {
            (1u32 << count) - 1
        } else {
            u32::MAX
        }
    }

    impl<T, E1: Selectable<Result = T>, E2: Selectable<Result = T>> EitherSelect<E1, E2> {
        const MASK1: u32 = generate_mask(E1::COUNT);
        const MASK2: u32 = generate_mask(E2::COUNT).rotate_left(E1::COUNT);
    }

    impl<T, E1: Selectable<Result = T>, E2: Selectable<Result = T>> Selectable
        for EitherSelect<E1, E2>
    {
        const COUNT: u32 = E1::COUNT + E2::COUNT;

        type Result = T;

        type Event = EitherEvent<E1, E2>;

        fn listen(self, offset: u32) -> Self::Event {
            EitherEvent(
                E1::listen(self.0, offset),
                E2::listen(self.1, offset + E1::COUNT),
            )
        }

        fn poll(event: Self::Event, mask: u32) -> Result<T, Self::Event> {
            Err(EitherEvent(
                if mask & Self::MASK1 == 0 {
                    event.0
                } else {
                    match E1::poll(event.0, mask) {
                        Ok(r) => return Ok(r),
                        Err(e) => e,
                    }
                },
                if mask & Self::MASK2 == 0 {
                    event.1
                } else {
                    match E2::poll(event.1, mask.rotate_right(E1::COUNT)) {
                        Ok(r) => return Ok(r),
                        Err(e) => e,
                    }
                },
            ))
        }

        fn sleep(event: &Self::Event) -> GenericSleep {
            E1::sleep(&event.0).combine(E2::sleep(&event.1))
        }
    }

    EitherSelect(fst, snd)
}

#[inline]
/// Awaits a [`Selectable`] event.
pub fn select<E: Selectable>(select: E) -> E::Result {
    let mut event = select.listen(0);
    loop {
        let mask = E::sleep(&event).sleep();
        event = match E::poll(event, mask) {
            Ok(r) => return r,
            Err(e) => e,
        }
    }
}

#[inline]
/// Creates a new [`Selectable`] event which completes after the given duration
/// of time.
pub fn delay(time: Duration) -> impl Selectable<Result = ()> {
    delay_until(time_since_start() + time)
}

#[inline]
/// Creates a new [`Selectable`] event which completes at the given timestamp.
pub fn delay_until(timestamp: Instant) -> impl Selectable<Result = ()> {
    struct DelaySelect(Instant);

    struct DelayEvent {
        timestamp: Instant,
        offset: u32,
    }

    impl Selectable for DelaySelect {
        const COUNT: u32 = 1;

        type Result = ();

        type Event = DelayEvent;

        fn listen(self, offset: u32) -> Self::Event {
            DelayEvent {
                timestamp: self.0,
                offset,
            }
        }

        fn poll(event: Self::Event, _mask: u32) -> Result<(), Self::Event> {
            if time_since_start() >= event.timestamp {
                Ok(())
            } else {
                Err(event)
            }
        }

        fn sleep(event: &Self::Event) -> GenericSleep {
            GenericSleep::Timestamp(event.timestamp, 1u32.rotate_left(event.offset))
        }
    }

    DelaySelect(timestamp)
}

mod broadcast;
mod channel;
mod context;
mod event;
mod r#loop;
mod mutex;
mod promise;
mod semaphore;

pub use broadcast::*;
pub use channel::*;
pub use context::*;
pub use event::*;
pub use mutex::*;
pub use promise::*;
pub use r#loop::*;
pub use semaphore::*;
