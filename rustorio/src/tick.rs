use std::fmt::Display;

/// The tick is used to keep track of time in the game.
/// You can advance the game by one tick using the [`next`](Tick::next) method.
/// Many functions and building methods require a [`Tick`](Tick) to be passed in, which allows them to update their state.
/// If a function takes a [`&mut Tick`](Tick), then the function will take time.
/// If a function merely takes a [`&Tick`](Tick), or no [`Tick`](Tick) at all, it will never advance the game time.
#[derive(Debug)]
pub struct Tick {
    tick: u64,
    log: bool,
}

impl Tick {
    pub(crate) fn start() -> Self {
        Self { tick: 0, log: true }
    }

    /// Sets whether or not to log on tick advancement.
    pub fn log(&mut self, log: bool) {
        self.log = log;
    }

    /// Advances the game by one tick.
    ///
    /// By default prints the current tick number to the console.
    /// If you want to disable this, use the [`log`](Tick::log) method.
    pub fn next(&mut self) {
        self.advance_by(1);
    }

    /// Advances the game by a specified number of ticks.
    ///
    /// By default prints the current tick number to the console.
    /// If you want to disable this, use the [`log`](Tick::log) method.
    pub fn advance_by(&mut self, adv: u64) {
        if self.log {
            println!("{self}");
        }
        self.tick = self.tick.checked_add(adv).expect("Tick overflow");
    }

    /// Advances the game until the specified tick number.
    ///
    /// By default prints the current tick number to the console.
    /// If you want to disable this, use the [`log`](Tick::log) method.
    pub fn advance_until(&mut self, tick: u64) {
        if tick < self.tick {
          eprintln!("Warning: Attempting to advance to a tick ({tick}) earlier than the current tick ({}). This will result in no change.", self.tick);
          return;
        }
        self.advance_by(tick - self.tick);
    }

    /// Advances the game until the specified condition is met.
    ///
    /// By default prints the current tick number to the console.
    /// If you want to disable this, use the [`log`](Tick::log) method.
    pub fn wait_until<F>(&mut self, mut condition: F)
    where
        F: FnMut(&Tick) -> bool,
    {
        while !condition(self) {
            self.next();
        }
    }

    /// Returns the current tick number.
    pub fn cur(&self) -> u64 {
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
