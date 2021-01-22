//! Support for synchronous and asynchronous state machines.

/// Denotes a type which represents a state machine.
pub trait StateMachine {
    /// The state type used by the state machine.
    type State: Ord;

    /// Gets the current state of the state machine.
    fn state(&self) -> Self::State;

    /// Transitions the state machine to a new state.
    fn transition(&self, state: Self::State);
}
