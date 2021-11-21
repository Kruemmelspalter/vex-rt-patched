use alloc::format;
use alloc::prelude::v1::String;
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::time::Duration;

use cstring_interop::{from_cstring_raw, with_cstring};

use crate::bindings;
use crate::error::Error;
use crate::error::SentinelError;
use crate::prelude::Box;
use crate::rtos::TIMEOUT_MAX;
use core::ffi::c_void;

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
    ///
    /// This is unsafe because it does not guarantee that the task's code safely
    /// unwinds (i.e., that destructors are called, memory is freed and other
    /// resources are released).
    pub unsafe fn delete(&self) {
        bindings::task_delete(self.0)
    }

    #[allow(dead_code)] // TODO: Remove when used
    /// Notifies the task, incrementing the notification counter
    pub(crate) fn notify(&self) {
        unsafe {
            bindings::task_notify(self.0);
        }
    }

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

    /// Clears the notifications for this task returning true if there were
    /// notifications
    pub fn clear_notifications(&self) -> bool {
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

/// Represents the state of a [`Task`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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