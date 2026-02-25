//! This module contains utilities to manipulate past ticks.
use crate::tick::Tick;

/// A record of a point in time.
#[derive(Debug, Clone, Copy)]
pub struct TickSnapshot {
    tick: u64,
}

/// Error returned when trying to move a tick snapshot backwards in time.
#[derive(Debug)]
pub struct BackwardTickingError;

impl Tick {
    /// Record a snapshot of a past tick.
    pub const fn snapshot(&self) -> TickSnapshot {
        TickSnapshot { tick: self.cur() }
    }
}
impl TickSnapshot {
    /// Advance the snapshot to the target snapshot, returning the number of ticks elapsed.
    pub const fn advance_to(&mut self, until: TickSnapshot) -> Result<u64, BackwardTickingError> {
        if let Some(diff) = until.tick.checked_sub(self.tick) {
            self.tick = until.tick;
            Ok(diff)
        } else {
            Err(BackwardTickingError)
        }
    }
}
