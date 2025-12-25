use std::fmt::Display;

/// The tick is used to keep track of time in the game.
/// You can advance the game using the [`advance`](Tick::advance) method or similar.
/// Many functions and building methods require a [`Tick`] to be passed in, which allows them to update their state.
/// If a function takes a [`&mut Tick`](Tick), then the function will take time.
/// If a function merely takes a [`&Tick`](Tick), or no [`Tick`]s at all, it will never advance the game time.
#[derive(Debug)]
pub struct Tick {
    /// The current tick number.
    tick: u64,
    log: bool,
}

impl Tick {
    pub(crate) const fn start() -> Self {
        Self { tick: 0, log: true }
    }

    /// Sets whether or not to log on tick advancement.
    pub const fn log(&mut self, log: bool) {
        self.log = log;
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
        self.tick = self.tick.checked_add(ticks).expect("Tick overflow. Well done you've found an exploit! Or you would have if `https://github.com/albertsgarde/rustorio/issues/3` hadn't beaten you to it!");
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
        if target_tick > self.tick {
            self.advance_by(target_tick - self.tick);
        }
    }

    /// Advances the game until the specified condition is met or the maximum number of ticks has passed.
    /// Returns `true` if the condition was met, or `false` if the maximum number of ticks was reached first.
    ///
    /// By default prints the current tick number to the console every tick.
    /// If you want to disable this, use the [`log`](Tick::log) method.
    pub fn advance_until<F>(&mut self, mut condition: F, max_ticks: u64) -> bool
    where
        F: FnMut(&Tick) -> bool,
    {
        let start_tick = self.tick;
        while !condition(self) && self.tick - start_tick < max_ticks {
            self.advance();
        }
        self.tick - start_tick < max_ticks
    }

    /// Returns the current tick number.
    pub const fn cur(&self) -> u64 {
        self.tick
    }
}

impl From<&Tick> for u64 {
    fn from(tick: &Tick) -> Self {
        tick.tick
    }
}

impl PartialOrd<u64> for &Tick {
    fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
        Some(self.tick.cmp(other))
    }
}

impl PartialOrd<&Tick> for u64 {
    fn partial_cmp(&self, other: &&Tick) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other.tick))
    }
}

impl PartialEq<u64> for &Tick {
    fn eq(&self, other: &u64) -> bool {
        self.tick == *other
    }
}

impl PartialEq<&Tick> for u64 {
    fn eq(&self, other: &&Tick) -> bool {
        *self == other.tick
    }
}

impl Display for Tick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tick {}", self.tick)
    }
}
