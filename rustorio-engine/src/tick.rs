//! Ticks keep track of time elapsed in the game.
use std::fmt::Display;

use crate::resources::TokenOfCreation;

/// A record of a point in time.
#[derive(Debug, Clone, Copy)]
pub struct TickSnapshot {
    /// The tick number.
    tick: u64,
}

/// Error returned when trying to move a tick snapshot backwards in time.
pub struct BackwardTickingError;

impl std::fmt::Debug for BackwardTickingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tried to move a tick snapshot backwards in time",)
    }
}

impl TickSnapshot {
    /// Create a snapshot at the given tick number.
    pub const fn new(tick: u64) -> Self {
        Self { tick }
    }

    /// Returns the current tick number.
    pub const fn cur(&self) -> u64 {
        self.tick
    }

    /// Advances the snapshot by the specified number of ticks.
    pub const fn advance_by(&mut self, ticks: u64) {
        self.tick = self.tick.checked_add(ticks).expect(
            "Tick overflow. Well done you've found an exploit! \
            Or you would have if `https://github.com/albertsgarde/rustorio/issues/3` \
            hadn't beaten you to it!",
        );
    }

    /// Advance the snapshot to the target snapshot, returning the number of ticks elapsed.
    pub fn advance_to(
        &mut self,
        until: impl Into<TickSnapshot>,
    ) -> Result<u64, BackwardTickingError> {
        let until = until.into();
        if let Some(diff) = until.tick.checked_sub(self.tick) {
            self.tick = until.tick;
            Ok(diff)
        } else {
            Err(BackwardTickingError)
        }
    }
}

/// Tracks elapsed game time.
///
/// This is the core engine type re-exported by content crates like the base mod `rustorio`.
/// Player-facing documentation and examples are available in the documentation in the `rustorio` crate.
#[derive(Debug)]
pub struct Tick {
    /// The current point in time.
    tick: TickSnapshot,
    /// Whether the tick should print a log message on advancement. By default, this is `false`.
    log: bool,
    /// The maximum tick number before the game panics.
    /// This is to prevent infinite loops, so should be set low enough that the game will crash quickly, but high enough that it won't crash during normal play.
    /// Initial value is set by the gamemode, but the player can change it using the [`set_max_tick`](Tick::set_max_tick) method.
    max_tick: u64,
}

impl Tick {
    pub(crate) const fn start(max_tick: u64) -> Self {
        Self {
            tick: TickSnapshot::new(0),
            log: false,
            max_tick,
        }
    }

    /// Sets whether or not to log on tick advancement.
    pub const fn log(&mut self, log: bool) {
        self.log = log;
    }

    /// Sets the maximum tick.
    /// If you attempt to advance beyond this number of ticks, the game will panic.
    /// This is to prevent infinite loops, so if you think you are hitting this without an infinite loop, you should increase this number.
    /// The initial value is set by the gamemode, but you can change it using this method.
    pub const fn set_max_tick(&mut self, max_tick: u64) {
        self.max_tick = max_tick;
    }

    /// Advances the game by one tick.
    ///
    /// By default prints the current tick number to the console.
    /// If you want to disable this, use the [`log`](Tick::log) method.
    pub fn advance(&mut self) {
        self.advance_by(1);
    }

    /// Advances the game by the specified number of ticks.
    ///
    /// By default prints the current tick number to the console.
    /// If you want to disable this, use the [`log`](Tick::log) method.
    pub fn advance_by(&mut self, ticks: u64) {
        self.tick.advance_by(ticks);
        if self.tick.cur() > self.max_tick {
            panic!(
                "Tick {} exceeded the maximum tick of {}. \
                This is likely due to an infinite loop. \
                If you intend to reach this tick, please increase the maximum tick using `Tick::set_max_tick`.",
                self.tick.cur(),
                self.max_tick
            );
        }
        if self.log {
            println!("{self}");
        }
    }

    /// Advances the game until the specified tick number is reached.
    /// Does nothing if the target tick is less than or equal to the current tick.
    ///
    /// By default prints the current tick number to the console.
    /// If you want to disable this, use the [`log`](Tick::log) method.
    pub fn advance_to_tick(&mut self, target_tick: u64) {
        // self.tick.advance_to(TickSnapshot::new(target_tick))
        if target_tick > self.tick.cur() {
            self.advance_by(target_tick - self.tick.cur());
        }
    }

    /// Advances the game until the specified condition is met or the maximum number of ticks has passed.
    /// Returns `true` if the condition was met, or `false` if the maximum number of ticks was reached first.
    ///
    /// By default prints the current tick number to the console every tick.
    /// If you want to disable this, use the [`log`](Tick::log) method.
    pub fn advance_until<F>(&mut self, mut condition: F)
    where
        F: FnMut(&Tick) -> bool,
    {
        while !condition(self) {
            self.advance();
        }
    }

    /// Returns the current tick number.
    pub const fn cur(&self) -> u64 {
        self.tick.cur()
    }

    /// Record a snapshot of a past tick.
    pub const fn snapshot(&self) -> TickSnapshot {
        self.tick
    }
}

/// Creates a new [`Tick`] with the specified maximum tick.
/// Should not be reexported in mods.
pub const fn tick(token: &TokenOfCreation, max_tick: u64) -> Tick {
    let _ = token;
    Tick::start(max_tick)
}

impl From<&Tick> for u64 {
    fn from(tick: &Tick) -> Self {
        tick.tick.cur()
    }
}

impl From<&Tick> for TickSnapshot {
    fn from(tick: &Tick) -> Self {
        tick.snapshot()
    }
}

impl PartialOrd<u64> for &Tick {
    fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
        Some(self.tick.cur().cmp(other))
    }
}

impl PartialOrd<&Tick> for u64 {
    fn partial_cmp(&self, other: &&Tick) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other.tick.cur()))
    }
}

impl PartialEq<u64> for &Tick {
    fn eq(&self, other: &u64) -> bool {
        self.tick.cur() == *other
    }
}

impl PartialEq<&Tick> for u64 {
    fn eq(&self, other: &&Tick) -> bool {
        *self == other.tick.cur()
    }
}

impl Display for Tick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tick {}", self.tick.cur())
    }
}
