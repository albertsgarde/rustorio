use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum AdvanceSubTickError {
    /// The sub tick is already ahead of the main tick.
    SubTickAheadOfMainTick,
}

/// The tick is used to keep track of time in the game.
/// You can advance the game using the [`advance`](Tick::advance) method or similar.
/// Many functions and building methods require a [`Tick`] to be passed in, which allows them to update their state.
/// If a function takes a [`&mut Tick`](Tick), then the function will take time.
/// If a function merely takes a [`&Tick`](Tick), it will never advance the game time, but instead just roll forward it's internal state to match the current tick.
///
/// # Examples
///
/// Let's say we have two furnaces the we want to fill with `iron_ore` and `copper_ore` respectively, and then advance time so they can smelt the ore into ingots:
/// ```
/// // Add ore to the furnaces at the current tick
/// furnace1.inputs(&tick).0 += iron_ore;
/// furnace2.inputs(&tick).0 += copper_ore;
/// // Advance time by 10 ticks so the furnaces can process some of the ore.
/// tick.advance_by(10);
/// // Now we can extract the smelted ingots from the furnaces
/// let iron_ingots = furnace1.outputs(&tick).0.empty().unwrap();
/// let copper_ingots = furnace2.outputs(&tick).0.empty().unwrap();
/// ```
#[derive(Debug)]
pub struct Tick<'own, 'parent>
where
    'parent: 'own,
{
    /// The current tick number.
    tick: u64,
    max_tick: u64,
    log: bool,
    _own_marker: std::marker::PhantomData<fn(&'own ()) -> &'own ()>,
    _parent_marker: std::marker::PhantomData<fn(&'parent ()) -> &'parent ()>,
}

pub type MainTick<'a> = Tick<'a, 'static>;

impl<'own> MainTick<'own> {
    pub(crate) const fn start() -> Self {
        Self {
            tick: 0,
            max_tick: u64::MAX,
            log: false,
            _own_marker: std::marker::PhantomData,
            _parent_marker: std::marker::PhantomData,
        }
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
}

impl<'own, 'parent> Tick<'own, 'parent> {
    pub(crate) const fn branch<'branch>(&self) -> Tick<'branch, 'own> {
        Tick {
            tick: self.tick,
            max_tick: self.tick,
            log: false,
            _own_marker: std::marker::PhantomData,
            _parent_marker: std::marker::PhantomData,
        }
    }

    /// Returns the current tick number.
    pub const fn cur(&self) -> u64 {
        self.tick
    }

    pub const fn parent_cur(&self) -> u64 {
        self.max_tick
    }

    pub(crate) const fn update(&mut self, main_tick: &Tick<'parent, '_>) {
        self.max_tick = main_tick.cur();
    }

    pub(crate) const fn advance_to_main(&mut self) {
        self.tick = self.max_tick;
    }

    /// Advances the game by one tick.
    pub const fn try_advance(&mut self) -> Result<(), AdvanceSubTickError> {
        self.try_advance_by(1)
    }

    /// Advances the game by the specified number of ticks.
    pub const fn try_advance_by(&mut self, ticks: u64) -> Result<(), AdvanceSubTickError> {
        if self.tick + ticks > self.max_tick {
            return Err(AdvanceSubTickError::SubTickAheadOfMainTick);
        }
        self.tick = self.tick.checked_add(ticks).expect("Tick overflow. Well done you've found an exploit! Or you would have if `https://github.com/albertsgarde/rustorio/issues/3` hadn't beaten you to it!");
        Ok(())
    }

    /// Advances the game until the specified tick number is reached.
    /// Does nothing if the target tick is less than or equal to the current tick.
    pub const fn try_advance_to_tick(
        &mut self,
        target_tick: u64,
    ) -> Result<(), AdvanceSubTickError> {
        if target_tick > self.tick {
            self.try_advance_by(target_tick - self.tick)
        } else {
            Ok(())
        }
    }

    /// Advances the game until the specified condition is met or the maximum number of ticks has passed.
    /// Returns `true` if the condition was met, or `false` if the maximum number of ticks or the main tick was reached first.
    pub fn try_advance_until<F>(&mut self, mut condition: F, max_ticks: u64) -> bool
    where
        F: FnMut(&Tick) -> bool,
    {
        let start_tick = self.tick;
        while !condition(self) {
            if self.tick - start_tick >= max_ticks || self.tick >= self.max_tick {
                return false;
            }
            self.try_advance().expect("Due to the checks in the while loop, this should never fail. If it does, please report this as a bug.");
        }
        true
    }
}

impl<'own, 'parent> From<&Tick<'own, 'parent>> for u64 {
    fn from(tick: &Tick<'own, 'parent>) -> Self {
        tick.tick
    }
}

impl<'own, 'parent> PartialOrd<u64> for &Tick<'own, 'parent> {
    fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
        Some(self.tick.cmp(other))
    }
}

impl<'own, 'parent> PartialOrd<&Tick<'own, 'parent>> for u64 {
    fn partial_cmp(&self, other: &&Tick<'own, 'parent>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other.tick))
    }
}

impl<'own, 'parent> PartialEq<u64> for &Tick<'own, 'parent> {
    fn eq(&self, other: &u64) -> bool {
        self.tick == *other
    }
}

impl<'own, 'parent> PartialEq<&Tick<'own, 'parent>> for u64 {
    fn eq(&self, other: &&Tick<'own, 'parent>) -> bool {
        *self == other.tick
    }
}

impl<'own, 'parent> Display for Tick<'own, 'parent> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tick {}", self.tick)
    }
}
