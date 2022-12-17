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

pub struct StateMachineData<S: Clone> {
    state: S,
    promise: Option<Box<dyn Any + Send>>,
    ctxw: ContextWrapper,
}

impl<S: Clone> StateMachineData<S> {
    pub fn new_wrapped(state: S) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            state,
            promise: None,
            ctxw: ContextWrapper::new(),
        }))
    }

    pub fn state(&self) -> &S {
        &self.state
    }

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

    pub fn transition(&mut self, state: S) -> Context {
        self.state = state;
        self.promise.take();
        self.ctxw.replace()
    }

    pub fn listen<T: Send + Sync>(&mut self) -> Promise<T> {
        let (promise, resolve) = Promise::new();
        let mut resolve = Some(resolve);
        let f = move |result| {
            if let Some(resolve) = resolve.take() {
                resolve(result);
            }
        };

        let inner_box: Box<dyn FnMut(T) + Send> = Box::new(f);
        let outer_box: Box<dyn Any + Send> = Box::new(inner_box);
        self.promise = Some(outer_box);

        promise
    }

    pub fn resolve<T: 'static>(&mut self, result: T) {
        if let Some(mut boxed) = self.promise.take() {
            if let Some(resolve) = boxed.downcast_mut::<Box<dyn FnMut(T) + Send>>() {
                resolve(result)
            }
        }
    }
}

pub type StateMachineHandle<S> = Arc<Mutex<StateMachineData<S>>>;
