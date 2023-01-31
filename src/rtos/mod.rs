//! Multitasking primitives.

use alloc::{boxed::Box, format, string::String};
use core::{
    cmp::min,
    convert::TryInto,
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
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
/// This type has a precision of 1 microsecond.
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
                .expect("overflow when creating Instant from milliseconds"),
        )
    }

    /// Creates a new `Instant` from the specified number of whole seconds since
    /// program start.
    pub fn from_secs(secs: u64) -> Self {
        Self(
            secs.checked_mul(1000000)
                .expect("overflow when creating Instant from seconds"),
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
/// Represents a FreeRTOS task.
pub struct Task(bindings::task_t);

impl Task {
    /// The default priority for new tasks.
    pub const DEFAULT_PRIORITY: u32 = bindings::TASK_PRIORITY_DEFAULT;

    /// The maximum priority for tasks.
    pub const MAX_PRIORITY: u32 = bindings::TASK_PRIORITY_MAX;

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
    pub fn current() -> Self {
        Self(unsafe { bindings::task_get_current() })
    }

    /// Finds a task by its name.
    pub fn find_by_name(name: &str) -> Result<Self, Error> {
        let ptr = (with_cstring(name.into(), |name| unsafe {
            bindings::task_get_by_name(name.into_raw()).check()
        }) as Result<*mut c_void, Error>)?;
        if ptr.is_null() {
            Err(Error::Custom(format!("task not found: {}", name)))
        } else {
            Ok(Self(ptr))
        }
    }

    #[inline]
    /// Spawns a new task with no name and the default priority and stack depth.
    pub fn spawn<F>(f: F) -> Result<Self, Error>
    where
        F: FnOnce() + Send + 'static,
    {
        Self::spawn_ext("", Self::DEFAULT_PRIORITY, Self::DEFAULT_STACK_DEPTH, f)
    }

    /// Spawns a new task with the specified name, priority and stack depth.
    pub fn spawn_ext<F>(name: &str, priority: u32, stack_depth: u16, f: F) -> Result<Self, Error>
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
            let r = Self::spawn_raw(name, priority, stack_depth, run::<F>, arg as *mut _);
            if r.is_err() {
                // We need to re-box the pointer if the task could not be created, to avoid a
                // memory leak.
                drop(Box::from_raw(arg));
            }
            r
        }
    }

    #[inline]
    /// Spawns a new task from a C function pointer and an arbitrary data
    /// pointer.
    ///
    /// # Safety
    /// This function spawns a task using raw c pointers. These are inherently
    /// unsafe. The arg passed must meet the requirements of the called
    /// function.
    pub unsafe fn spawn_raw(
        name: &str,
        priority: u32,
        stack_depth: u16,
        f: unsafe extern "C" fn(arg1: *mut libc::c_void),
        arg: *mut libc::c_void,
    ) -> Result<Self, Error> {
        with_cstring(name.into(), |cname| {
            Ok(Self(
                bindings::task_create(Some(f), arg, priority, stack_depth, cname.into_raw())
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
    ///
    /// This is unsafe because it does not guarantee that the task's code safely
    /// unwinds (i.e., that destructors are called, memory is freed and other
    /// resources are released).
    pub unsafe fn delete(&self) {
        bindings::task_delete(self.0)
    }

    #[inline]
    /// Suspends execution of the task until [`resume`](Self::resume()) is
    /// called.
    pub fn suspend(&self) {
        unsafe {
            bindings::task_suspend(self.0);
        }
    }

    #[inline]
    /// Resumes execution of the task.
    pub fn resume(&self) {
        unsafe {
            bindings::task_resume(self.0);
        }
    }

    #[allow(dead_code)] // TODO: Remove when used
    #[inline]
    /// Notifies the task, incrementing the notification counter
    pub(crate) fn notify(&self) {
        unsafe {
            bindings::task_notify(self.0);
        }
    }

    #[allow(dead_code)] // TODO: Remove when used
    #[inline]
    /// Waits for notifications on the current task, returning the number before
    /// decrement or clear. If clear is false will decrement notification
    /// number instead of setting to 0;
    pub(crate) fn notify_take(clear: bool, timeout: Option<Duration>) -> u32 {
        unsafe {
            bindings::task_notify_take(
                clear,
                timeout.map_or(TIMEOUT_MAX, |timeout| timeout.as_millis() as u32),
            )
        }
    }

    #[allow(dead_code)] // TODO: Remove when used
    #[inline]
    /// Clears the notifications for this task returning true if there were
    /// notifications
    pub(crate) fn clear_notifications(&self) -> bool {
        unsafe { bindings::task_notify_clear(self.0) }
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

#[derive(Copy, Clone, Debug)]
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
    NotifyTake(Option<Instant>),
    /// Represents an explicit future timestamp.
    Timestamp(Instant),
    /// Represents a time that is now or in the past.
    Ready,
    /// Represents a time at infinity.
    Never,
}

impl GenericSleep {
    /// Sleeps until the future time represented by `self`. The result is the
    /// number of notifications which were present, if the sleep ended due to
    /// notification.
    pub fn sleep(self) -> u32 {
        match self {
            GenericSleep::NotifyTake(timeout) => {
                let timeout = timeout.map_or(TIMEOUT_MAX, |v| {
                    v.checked_sub_instant(time_since_start())
                        .map_or(0, |d| d.as_millis() as u32)
                });
                unsafe { bindings::task_notify_take(true, timeout) }
            }
            GenericSleep::Timestamp(v) => {
                if let Some(d) = v.checked_sub_instant(time_since_start()) {
                    Task::delay(d);
                }
                0
            }
            GenericSleep::Ready => 0,
            GenericSleep::Never => panic!("attempted to sleep forever"),
        }
    }

    #[inline]
    /// Get the timestamp represented by `self`, if it is present.
    pub fn timeout(self) -> Option<Instant> {
        match self {
            GenericSleep::NotifyTake(v) => v,
            GenericSleep::Timestamp(v) => Some(v),
            _ => None,
        }
    }

    /// Combine two `GenericSleep` objects to one which represents the earliest
    /// possible time of the two.
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (_, GenericSleep::Ready) => GenericSleep::Ready,
            (GenericSleep::Ready, _) => GenericSleep::Ready,
            (a, GenericSleep::Never) => a,
            (GenericSleep::Never, b) => b,
            (GenericSleep::Timestamp(a), GenericSleep::Timestamp(b)) => {
                GenericSleep::Timestamp(min(a, b))
            }
            (a, b) => GenericSleep::NotifyTake(
                a.timeout()
                    .map_or(b.timeout(), |a| Some(b.timeout().map_or(a, |b| min(a, b)))),
            ),
        }
    }
}

/// Represents a future event which can be used with the
/// [`select!`](crate::select!) macro.
#[must_use]
pub trait Selectable: Sized {
    /// The type of the event result.
    type Output;

    /// Processes the event if it is ready, consuming the event object;
    /// otherwise, it provides a replacement event object.
    fn poll(self) -> Result<Self::Output, Self>;

    /// Gets the earliest time that the event could be ready.
    fn sleep(&self) -> GenericSleep;
}

/// An extension trait which provides utility functions for [`Selectable`]
/// events.
pub trait SelectableExt: Selectable {
    /// Waits for the event to complete.
    fn wait(self) -> Self::Output {
        select(self)
    }
}

impl<E: Selectable> SelectableExt for E {}

#[inline]
/// Creates a new [`Selectable`] event by mapping the result of a given one.
pub fn select_map<'a, T: 'a, U: 'a>(
    event: impl Selectable<Output = T> + 'a,
    f: impl 'a + FnOnce(T) -> U,
) -> impl Selectable<Output = U> + 'a {
    struct MapSelect<U, E: Selectable, F: FnOnce(E::Output) -> U> {
        event: E,
        f: F,
        _t: PhantomData<E::Output>,
    }

    impl<U, E: Selectable, F: FnOnce(E::Output) -> U> Selectable for MapSelect<U, E, F> {
        type Output = U;

        fn poll(self) -> Result<Self::Output, Self> {
            match self.event.poll() {
                Ok(r) => Ok((self.f)(r)),
                Err(event) => Err(Self { event, ..self }),
            }
        }
        fn sleep(&self) -> GenericSleep {
            self.event.sleep()
        }
    }

    MapSelect {
        event,
        f,
        _t: PhantomData,
    }
}

#[inline]
/// Creates a new [`Selectable`] event which processes exactly one of the given
/// events.
pub fn select_either<'a, T: 'a>(
    fst: impl Selectable<Output = T> + 'a,
    snd: impl Selectable<Output = T> + 'a,
) -> impl Selectable<Output = T> + 'a {
    struct EitherSelect<T, E1: Selectable<Output = T>, E2: Selectable<Output = T>>(
        E1,
        E2,
        PhantomData<T>,
    );

    impl<T, E1: Selectable<Output = T>, E2: Selectable<Output = T>> Selectable
        for EitherSelect<T, E1, E2>
    {
        type Output = T;

        fn poll(self) -> Result<Self::Output, Self> {
            Err(Self(
                match self.0.poll() {
                    Ok(r) => return Ok(r),
                    Err(e) => e,
                },
                match self.1.poll() {
                    Ok(r) => return Ok(r),
                    Err(e) => e,
                },
                PhantomData,
            ))
        }
        fn sleep(&self) -> GenericSleep {
            self.0.sleep().combine(self.1.sleep())
        }
    }

    EitherSelect(fst, snd, PhantomData)
}

#[inline]
/// Creates a new [`Selectable`] event which waits for both of the given events
/// to complete, processing each when it is ready.
pub fn select_both<'a, T: 'a, U: 'a>(
    fst: impl Selectable<Output = T> + 'a,
    snd: impl Selectable<Output = U> + 'a,
) -> impl Selectable<Output = (T, U)> {
    enum BothSelect<E1: Selectable, E2: Selectable> {
        Neither(E1, E2),
        GotFirst(E1::Output, E2),
        GotSecond(E1, E2::Output),
    }

    impl<E1: Selectable, E2: Selectable> Selectable for BothSelect<E1, E2> {
        type Output = (E1::Output, E2::Output);

        fn poll(self) -> Result<Self::Output, Self> {
            match self {
                Self::Neither(fst, snd) => Err(match (fst.poll(), snd.poll()) {
                    (Ok(fst), Ok(snd)) => return Ok((fst, snd)),
                    (Ok(fst), Err(snd)) => Self::GotFirst(fst, snd),
                    (Err(fst), Ok(snd)) => Self::GotSecond(fst, snd),
                    (Err(fst), Err(snd)) => Self::Neither(fst, snd),
                }),
                Self::GotFirst(fst, snd) => match snd.poll() {
                    Ok(snd) => Ok((fst, snd)),
                    Err(snd) => Err(Self::GotFirst(fst, snd)),
                },
                Self::GotSecond(fst, snd) => match fst.poll() {
                    Ok(fst) => Ok((fst, snd)),
                    Err(fst) => Err(Self::GotSecond(fst, snd)),
                },
            }
        }

        fn sleep(&self) -> GenericSleep {
            match self {
                BothSelect::Neither(fst, snd) => fst.sleep().combine(snd.sleep()),
                BothSelect::GotFirst(_, snd) => snd.sleep(),
                BothSelect::GotSecond(fst, _) => fst.sleep(),
            }
        }
    }

    BothSelect::Neither(fst, snd)
}

#[inline]
/// Creates a new [`Selectable`] event which never completes if the given base
/// event is None.
pub fn select_option<'a, T: 'a>(
    base: Option<impl Selectable<Output = T> + 'a>,
) -> impl Selectable<Output = T> + 'a {
    struct OptionSelect<E: Selectable>(Option<E>, PhantomData<E::Output>);

    impl<E: Selectable> Selectable for OptionSelect<E> {
        type Output = E::Output;

        fn poll(self) -> Result<Self::Output, Self> {
            Err(Self(
                if let Some(e) = self.0 {
                    match e.poll() {
                        Ok(r) => return Ok(r),
                        Err(e) => Some(e),
                    }
                } else {
                    None
                },
                PhantomData,
            ))
        }

        fn sleep(&self) -> GenericSleep {
            self.0
                .as_ref()
                .map_or(GenericSleep::NotifyTake(None), Selectable::sleep)
        }
    }

    OptionSelect(base, PhantomData)
}

#[inline]
/// Awaits a [`Selectable`] event.
pub fn select<'a, T: 'a>(mut event: impl Selectable<Output = T> + 'a) -> T {
    loop {
        event.sleep().sleep();
        event = match event.poll() {
            Ok(r) => return r,
            Err(e) => e,
        }
    }
}

#[inline]
/// Creates a new [`Selectable`] event which completes after the given duration
/// of time.
pub fn delay(time: Duration) -> impl Selectable {
    delay_until(time_since_start() + time)
}

#[inline]
/// Creates a new [`Selectable`] event which completes at the given timestamp.
pub fn delay_until(timestamp: Instant) -> impl Selectable {
    struct DelaySelect(Instant);

    impl Selectable for DelaySelect {
        type Output = ();

        fn poll(self) -> Result<Self::Output, Self> {
            if time_since_start() >= self.0 {
                Ok(())
            } else {
                Err(self)
            }
        }

        fn sleep(&self) -> GenericSleep {
            GenericSleep::Timestamp(self.0)
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
mod queue;
mod semaphore;

pub use broadcast::*;
pub use channel::*;
pub use context::*;
pub use event::*;
pub use mutex::*;
pub use promise::*;
pub use queue::*;
pub use r#loop::*;
pub use semaphore::*;
