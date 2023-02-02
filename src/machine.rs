//! Support for synchronous and asynchronous state machines.

use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    sync::Arc,
};
use core::{
    any::Any,
    marker::{Send, Sync},
};

use crate::rtos::{Context, ContextWrapper, GenericSleep, Mutex, Promise, Selectable, Task};

/// Denotes afield2type which represents a state machine.
pub trait StateMachine {
    /// The state type used by the state machine.
    type State: StateType;

    /// Gets the current state of the state machine.
    fn state(&self) -> Self::State;

    /// Transitions the state machine to a new state.
    ///
    /// Returns the context in which the new state is running.
    fn transition(&self, state: Self::State) -> Context;
}

/// Denotes a type which represents the state of a state machine.
pub trait StateType: Clone + Send + Sync + 'static {
    /// The human-readable name for the state machine.
    const STATE_MACHINE_NAME: &'static str;

    /// Gives the human-readable name for the state.
    fn name(&self) -> &str;
}

/// Data structure used by state machines generated using the
/// [`state_machine!`](crate::state_machine!) macro.
pub struct StateMachineData<S: StateType> {
    state: S,
    task: Task,
    next_frame: Option<StateFrame<S>>,
    ctxw: ContextWrapper,
}

impl<S: StateType> StateMachineData<S> {
    /// Constructs a new data structure, wrapped in a [`StateMachineHandle`].
    pub fn new_wrapped(state: S) -> StateMachineHandle<S> {
        let mut ctxw = ContextWrapper::new_ext(Some(S::STATE_MACHINE_NAME.to_string()));
        Arc::new(Mutex::new(Self {
            state: state.clone(),
            task: Task::current(),
            next_frame: Some(StateFrame {
                state,
                ctx: ctxw.replace(),
                listener: ListenerBox(None),
            }),
            ctxw,
        }))
    }

    /// Gets a reference to the current state.
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Sets the task which the state machine is running on.
    pub fn set_task(&mut self, task: Task) {
        self.task = task;
    }

    /// Begins executing a new state.
    ///
    /// Called by the state machine main task. Returns the new state frame.
    pub fn try_begin(&mut self) -> Option<StateFrame<S>> {
        let frame = self.next_frame.take()?;

        #[cfg(feature = "logging")]
        log::debug!(
            target: S::STATE_MACHINE_NAME,
            "{} -> {} by request {}",
            self.state.name(),
            frame.state.name(),
            frame.ctx.name(),
        );

        self.state = frame.state.clone();
        Some(frame)
    }

    /// Transitions directly to a new state.
    ///
    /// Called by the state machine main task. Returns the new state frame.
    pub fn tail_transition(&mut self, frame: StateFrame<S>, state: S) -> StateFrame<S> {
        #[cfg(feature = "logging")]
        log::debug!(
            target: S::STATE_MACHINE_NAME,
            "{} -> {} by tail-transition",
            self.state.name(),
            state.name(),
        );

        self.state = state.clone();
        StateFrame { state, ..frame }
    }

    /// Begins setting up a transition request to a new state.
    ///
    /// Returns a builder for the transition operation.
    pub fn transition(&mut self, state: S) -> TransitionBuilder<'_, S> {
        self.transition_impl(None, state)
    }

    /// Begins setting up a transition request to a new state, given a parent
    /// context to limit the execution of the state body.
    pub fn transition_ext<'a>(
        &'a mut self,
        ctx: &'a Context,
        state: S,
    ) -> TransitionBuilder<'a, S> {
        self.transition_impl(Some(ctx), state)
    }

    fn transition_impl<'a>(
        &'a mut self,
        ctx: Option<&'a Context>,
        state: S,
    ) -> TransitionBuilder<'a, S> {
        #[cfg(feature = "logging")]
        log::trace!(target: S::STATE_MACHINE_NAME, "requesting {}", state.name());

        TransitionBuilder {
            state,
            ctx,
            listener: ListenerBox(None),
            data: self,
        }
    }
}

/// A shared instance of [`StateMachineData`].
pub type StateMachineHandle<S> = Arc<Mutex<StateMachineData<S>>>;

/// Gives an event which resolves once a state request is available.
///
/// Must be called from the task which runs the state machine.
pub fn state_begin<S: StateType>(
    handle: &StateMachineHandle<S>,
) -> impl Selectable<Output = StateFrame<S>> + '_ {
    struct BeginSelect<'a, S: StateType>(&'a StateMachineHandle<S>);

    impl<'a, S: StateType> Selectable for BeginSelect<'a, S> {
        type Output = StateFrame<S>;

        fn poll(self) -> Result<Self::Output, Self> {
            self.0.lock().try_begin().ok_or(self)
        }

        fn sleep(&self) -> crate::prelude::GenericSleep {
            if self.0.lock().next_frame.is_some() {
                GenericSleep::Ready
            } else {
                GenericSleep::NotifyTake(None)
            }
        }
    }

    BeginSelect(handle)
}

/// A builder for a transition request.
pub struct TransitionBuilder<'a, S: StateType> {
    state: S,
    ctx: Option<&'a Context>,
    listener: ListenerBox,
    data: &'a mut StateMachineData<S>,
}

impl<'a, S: StateType> TransitionBuilder<'a, S> {
    /// Executes the transition request.
    ///
    /// Returns the context under which that state will execute.
    pub fn finish(self) -> Context {
        let ctx = self.data.ctxw.replace_ext(&self.ctx);
        self.data.next_frame = Some(StateFrame {
            state: self.state,
            ctx: ctx.clone(),
            listener: self.listener,
        });
        self.data.task.notify();
        ctx
    }

    /// Adds a listener to the transition request.
    ///
    /// Returns a promise which will resolve with the result of the state when
    /// its execution completes.
    pub fn listen<T: Send + Sync>(&mut self) -> Promise<T> {
        self.listener
            .listen(format!("{}/{}", S::STATE_MACHINE_NAME, self.state.name()))
    }
}

/// The data and objects needed to execute a state in the state machine.
pub struct StateFrame<S: StateType> {
    /// The state to execute.
    pub state: S,
    /// The context in which to execute the state.
    pub ctx: Context,

    listener: ListenerBox,
}

impl<S: StateType> StateFrame<S> {
    /// Indicates that the state is finished executing and provides its result
    /// to any tasks that are waiting for it.
    pub fn resolve<T: 'static>(&mut self, result: T) {
        self.listener.resolve(result);
    }
}

#[repr(transparent)]
struct ListenerBox(Option<Box<dyn Any + Send + Sync>>);

impl ListenerBox {
    fn listen<T: Send + Sync>(&mut self, name: String) -> Promise<T> {
        if self.0.is_some() {
            panic!("cannot override listener")
        }

        let (promise, resolve) = Promise::new_ext(Some(name));
        let mut resolve = Some(resolve);
        let f = move |result| {
            if let Some(resolve) = resolve.take() {
                resolve(result);
            }
        };

        // The resolve function needs to be double-boxed as there are two unknown types
        // (T and the closure type).
        let inner_box: Box<dyn FnMut(T) + Send + Sync> = Box::new(f);
        let outer_box: Box<dyn Any + Send + Sync> = Box::new(inner_box);
        self.0 = Some(outer_box);

        promise
    }

    fn resolve<T: 'static>(&mut self, result: T) {
        if let Some(mut boxed) = self.0.take() {
            if let Some(resolve) = boxed.downcast_mut::<Box<dyn FnMut(T) + Send>>() {
                resolve(result);
            } else {
                #[cfg(feature = "logging")]
                log::warn!(
                    "state result type does not match: expected {:?}, got {:?}",
                    core::any::TypeId::of::<T>(),
                    boxed.type_id()
                );
            }
        }
    }
}

/// The possible results for state processing in a state machine.
///
/// `T` is the output type of the state, and `S` is the state type of the state
/// machine.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateResult<T, S> {
    /// Finishes processing the state with the given output, without
    /// transitioning to a new state.
    Simple(T),
    /// Finishes processing the state with the given output and transitions to
    /// the given next state.
    Transition(T, S),
}

impl<T, S> StateResult<T, S> {
    /// Produces the result as a tuple of output value and optional next state.
    pub fn into_tuple(self) -> (T, Option<S>) {
        match self {
            StateResult::Simple(result) => (result, None),
            StateResult::Transition(result, next) => (result, Some(next)),
        }
    }
}
