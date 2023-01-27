//! Support for synchronous and asynchronous state machines.

use alloc::{boxed::Box, sync::Arc};
use core::{
    any::Any,
    marker::{Send, Sync},
};

use crate::rtos::{Context, ContextWrapper, Mutex, Promise};

/// Denotes a type which represents a state machine.
pub trait StateMachine {
    /// The state type used by the state machine.
    type State;

    /// Gets the current state of the state machine.
    fn state(&self) -> Self::State;

    /// Transitions the state machine to a new state.
    ///
    /// Returns the context in which the new state is running.
    fn transition(&self, state: Self::State) -> Context;
}

/// Data structure used by state machines generated using the
/// [`state_machine!`](crate::state_machine!) macro.
pub struct StateMachineData<S: Clone> {
    state: S,
    listener: ListenerBox,
    ctxw: ContextWrapper,
}

impl<S: Clone> StateMachineData<S> {
    /// Constructs a new data structure, wrapped in a [`StateMachineHandle`].
    pub fn new_wrapped(state: S) -> StateMachineHandle<S> {
        Arc::new(Mutex::new(Self {
            state,
            listener: ListenerBox(None),
            ctxw: ContextWrapper::new(),
        }))
    }

    /// Gets a reference to the current state.
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Begins executing a new state.
    ///
    /// Returns the state to execute and the context for the execution.
    pub fn begin(&mut self) -> (S, Context) {
        (
            self.state.clone(),
            if let Some(ctx) = self.ctxw.current() {
                ctx.clone()
            } else {
                self.ctxw.replace()
            },
        )
    }

    /// Instructs a transition to a new state.
    ///
    /// Returns the context under which that state will execute.
    pub fn transition(&mut self, state: S) -> Context {
        self.state = state;
        self.listener.clear();
        self.ctxw.replace()
    }

    /// Instructs a transition to a new state, given a parent context to limit
    /// the execution of the state body.
    pub fn transition_ext(&mut self, ctx: Context, state: S) -> Context {
        self.state = state;
        self.listener.clear();
        self.ctxw.replace_ext(ctx)
    }

    /// Produces a promise which listens for the result of the current state.
    ///
    /// The promise will only be resolved if `T` matches the result type of the
    /// current state.
    pub fn listen<T: Send + Sync>(&mut self) -> Promise<T> {
        self.listener.listen::<T>()
    }

    /// Resolves the listener promise, if there is one and its type matches.
    pub fn resolve<T: 'static>(&mut self, result: T) {
        self.listener.resolve::<T>(result);
    }
}

/// A shared instance of [`StateMachineData`].
pub type StateMachineHandle<S> = Arc<Mutex<StateMachineData<S>>>;

struct ListenerBox(Option<Box<dyn Any + Send>>);

impl ListenerBox {
    fn clear(&mut self) {
        self.0.take();
    }

    fn listen<T: Send + Sync>(&mut self) -> Promise<T> {
        if self.0.is_some() {
            panic!("cannot override listener")
        }

        let (promise, resolve) = Promise::new();
        let mut resolve = Some(resolve);
        let f = move |result| {
            if let Some(resolve) = resolve.take() {
                resolve(result);
            }
        };

        let inner_box: Box<dyn FnMut(T) + Send> = Box::new(f);
        let outer_box: Box<dyn Any + Send> = Box::new(inner_box);
        self.0 = Some(outer_box);

        promise
    }

    fn resolve<T: 'static>(&mut self, result: T) {
        if let Some(mut boxed) = self.0.take() {
            if let Some(resolve) = boxed.downcast_mut::<Box<dyn FnMut(T) + Send>>() {
                resolve(result)
            }
        }
    }
}
