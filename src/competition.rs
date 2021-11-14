//! The competition state.

use crate::bindings;
use bitflags::bitflags;

bitflags! {
    /// The state of competition.
    pub struct CompetitionStatus: u8{
        /// The robot is disabled.
        const DISABLED = bindings::COMPETITION_DISABLED as u8;
        /// The robot is connected.
        const CONNECTED = bindings::COMPETITION_CONNECTED as u8;
        /// The robot is in autonomous.
        const AUTONOMOUS = bindings::COMPETITION_AUTONOMOUS as u8;
        /// An invalid state.
        const INVALID = 1 << 7;
    }
}
impl CompetitionStatus {
    /// Gets the current competition state.
    pub fn get() -> Self {
        Self::from_bits_truncate(unsafe { bindings::competition_get_status() })
    }
}
