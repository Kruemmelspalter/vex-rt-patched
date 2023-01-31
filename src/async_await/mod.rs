//! RTOS-based async executor.

#![cfg(feature = "async-await")]
#![cfg_attr(docsrs, doc(cfg(feature = "async-await")))]

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BinaryHeap},
    sync::Arc,
};
use by_address::ByAddress;
use core::{
    cell::Cell,
    fmt::{self, Debug, Formatter},
    pin::Pin,
    ptr::NonNull,
    task::{Context, Poll},
};
use futures::{
    future::LocalBoxFuture,
    task::{waker, ArcWake},
    Future, FutureExt,
};
use libc_print::libc_eprintln;

use crate::{
    rtos::{self, queue, Selectable, SendQueue, Task},
    select,
};

/// Launches an executor on a new task and returns its dispatcher.
pub fn launch(ctx: rtos::Context) -> Dispatcher {
    let (send, recv) = queue(BinaryHeap::new());
    let sender = send.clone();

    Task::spawn_ext(
        "executor",
        Task::MAX_PRIORITY,
        Task::DEFAULT_STACK_DEPTH,
        move || {
            let repo = Repository::new();
            let handle_cell = Cell::default();
            let priority_cell = Cell::default();
            let mut tasks = BTreeMap::new();
            let ec = ExecutionContext {
                repo: &repo,
                sender: &sender,
                handle: &handle_cell,
                priority: &priority_cell,
                tasks: &mut tasks,
            };

            loop {
                // NOTE: the order here is important. In particular, repo.select() must be
                // checked before recv.select(), because the Selectable ecosystem assumes that
                // events will be polled or dropped in a short amount of time after a task
                // notification is sent. Because events are long-lived in repo, to uphold this
                // expectation we must process all ready events before moving on to executing
                // dispatched tasks.
                let (priority, dispatch) = select! {
                    _ = ctx.done() => break,
                    _ = repo.select() => continue,
                    msg = recv.select() => msg,
                };

                priority_cell.set(priority);

                let handle = match dispatch {
                    Dispatch::New(ByAddress(mut f)) => {
                        let future = f(ec);
                        let handle: WakeRef = WakeRef::from(&future);
                        assert!(tasks.insert(handle, future).is_none());
                        handle
                    }
                    Dispatch::Wake(handle) => handle,
                };

                handle_cell.set(Some(handle));

                if let Some(future) = tasks.get_mut(&handle) {
                    let task = AsyncTask {
                        priority,
                        handle,
                        sender: sender.clone(),
                    };
                    let waker = waker(Arc::new(task));
                    let context = &mut Context::from_waker(&waker);

                    if future.as_mut().poll(context).is_ready() {
                        tasks.remove(&handle);
                    }
                } else {
                    libc_eprintln!("task {:?} not found; tasks = {:?}", handle, tasks.keys());
                }
            }
        },
    )
    .expect("failed to start executor task");

    Dispatcher(send)
}

struct AsyncTask {
    priority: u16,
    handle: WakeRef,
    sender: SendQueue<(u16, Dispatch)>,
}

impl ArcWake for AsyncTask {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        assert!(
            arc_self
                .sender
                .send((arc_self.priority, Dispatch::Wake(arc_self.handle))),
            "unable to wake task: {:?}",
            arc_self.handle
        );
    }
}

type TaskFn = Box<dyn for<'a> FnMut(ExecutionContext<'a>) -> LocalBoxFuture<'a, ()> + Send + Sync>;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Dispatch {
    New(ByAddress<TaskFn>),
    Wake(WakeRef),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(transparent)]
struct WakeRef(NonNull<()>);

impl Debug for WakeRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.as_ptr().fmt(f)
    }
}

impl<'a> From<&LocalBoxFuture<'a, ()>> for WakeRef {
    fn from(value: &LocalBoxFuture<'a, ()>) -> Self {
        let ptr: *const _ = &**value;
        WakeRef(NonNull::new(ptr as _).unwrap())
    }
}

unsafe impl Send for WakeRef {}
unsafe impl Sync for WakeRef {}

/// Provides a means to dispatch tasks to an executor.
pub struct Dispatcher(SendQueue<(u16, Dispatch)>);

impl Dispatcher {
    #[inline]
    /// Dispatches the given task to the executor at the given priority.
    pub fn dispatch(
        &self,
        priority: u16,
        f: impl for<'a> FnOnce(ExecutionContext<'a>) -> LocalBoxFuture<'a, ()> + Send + Sync + 'static,
    ) {
        let mut f = Some(f);
        self.dispatch_boxed(priority, Box::new(move |cx| f.take().unwrap()(cx)));
    }

    /// Dispatches the given task to the executor at the given priority.
    pub fn dispatch_boxed(&self, priority: u16, f: TaskFn) {
        assert!(
            self.0.send((priority, Dispatch::New(ByAddress(f)))),
            "unable to dispatch task"
        );
    }
}

#[derive(Clone, Copy)]
/// Provides facilities for interacting with the executor from an async task.
pub struct ExecutionContext<'a> {
    repo: &'a Repository,
    sender: &'a SendQueue<(u16, Dispatch)>,
    handle: &'a Cell<Option<WakeRef>>,
    priority: &'a Cell<u16>,
    tasks: *mut BTreeMap<WakeRef, LocalBoxFuture<'a, ()>>,
}

impl<'a> ExecutionContext<'a> {
    #[inline]
    /// Gets the priority of the current task.
    pub fn priority(self) -> u16 {
        self.priority.get()
    }

    #[inline]
    /// Returns a future which, when awaited, sets the priority of the awaiting
    /// task.
    pub fn set_priority(self, priority: u16) -> impl Future<Output = ()> + 'a {
        struct PriorityFuture<'a>(Option<(ExecutionContext<'a>, u16)>);

        impl<'a> Future for PriorityFuture<'a> {
            type Output = ();

            fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                if let Some((ec, priority)) = self.0.take() {
                    ec.sender
                        .send((priority, Dispatch::Wake(ec.handle.get().unwrap())));
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            }
        }

        PriorityFuture(Some((self, priority)))
    }

    #[inline]
    /// Dispatches the given task to the executor at the given priority.
    pub fn dispatch(self, priority: u16, future: impl Future<Output = ()> + 'a) {
        self.dispatch_boxed(priority, future.boxed_local());
    }

    /// Dispatches the given task to the executor at the given priority.
    pub fn dispatch_boxed(self, priority: u16, future: LocalBoxFuture<'a, ()>) {
        let handle = WakeRef::from(&future);
        unsafe { &mut *self.tasks }.insert(handle, future);
        self.sender.send((priority, Dispatch::Wake(handle)));
    }

    /// Consumes an event and returns a future which resolves with
    /// its result. When the future is awaited, the executor's RTOS task waits
    /// on the event cooperatively with other async tasks. If the future is
    /// dropped, the event is also dropped.
    pub async fn proxy<T>(self, event: impl Selectable<Output = T>) -> T {
        self.repo.proxy(event).await
    }
}

mod promise;
mod repository;

pub use promise::*;
pub use repository::*;
