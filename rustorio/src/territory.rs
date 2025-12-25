//! Territories are where you can get ore.
//! To begin with you can mine by hand using the [`hand_mine`](Territory::hand_mine) function,
//! but later you can add [`Miner`](Miner)s to the territory to automate mining.

use std::fmt::Display;

use rustorio_engine::{
    ResourceType, bundle,
    mod_reexports::{Bundle, Resource, Tick},
    resource,
};

use crate::resources::Iron;

/// Ore is mined every MINING_TICK_LENGTH ticks by each miner in a territory.
pub const MINING_TICK_LENGTH: u64 = 2;

const fn tick_to_mining_tick(tick: u64) -> u64 {
    tick / MINING_TICK_LENGTH
}

/// A miner that can be added to a territory to mine resources.
#[derive(Debug)]
#[non_exhaustive]
pub struct Miner;

impl Miner {
    /// Builds a new miner. Requires 10 iron to build.
    pub const fn build(iron: Bundle<Iron, 10>) -> Self {
        let _ = iron;
        Miner
    }
}

/// Error returned when trying to add a miner to a full territory.
#[derive(Debug)]
pub struct TerritoryFullError {
    /// The maximum number of miners allowed in the territory.
    pub max_miners: u32,
    /// The miner that could not be added.
    pub miner: Miner,
}

impl Display for TerritoryFullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Territory is full: maximum number of miners is {}",
            self.max_miners
        )
    }
}

/// A territory that can hold miners to mine a specific type of ore.
#[derive(Debug)]
#[non_exhaustive]
pub struct Territory<OreType: ResourceType> {
    mining_tick: u64,
    /// The maximum number of miners allowed in the territory.
    max_miners: u32,
    miners: u32,
    resources: Resource<OreType>,
}

impl<OreType: ResourceType> Territory<OreType> {
    /// Creates a new territory that can hold up to `max_miners` miners.
    pub(crate) const fn new(tick: &Tick, max_miners: u32) -> Self {
        Self {
            mining_tick: tick_to_mining_tick(tick.cur()),
            max_miners,
            miners: 0,
            resources: Resource::new_empty(),
        }
    }

    /// Returns the the number of miner slots available in the territory.
    pub const fn max_miners(&self) -> u32 {
        self.max_miners
    }

    /// Returns the current number of miners in the territory.
    pub const fn num_miners(&self) -> u32 {
        self.miners
    }

    fn tick(&mut self, tick: &Tick) {
        let mining_tick = tick_to_mining_tick(tick.cur());
        assert!(self.mining_tick <= mining_tick, "Tick went backwards");
        let mining_tick_delta = mining_tick - self.mining_tick;
        self.resources += resource(
            u32::try_from(mining_tick_delta).expect("Mining tick delta too large") * self.miners,
        );
        self.mining_tick = mining_tick;
    }

    /// Mines ore by hand, advancing the tick by [`MINING_TICK_LENGTH`] for each unit mined.
    pub fn hand_mine<const AMOUNT: u32>(&mut self, tick: &mut Tick) -> Bundle<OreType, AMOUNT> {
        self.tick(tick);
        tick.advance_by((u64::from(AMOUNT)) * MINING_TICK_LENGTH);
        bundle()
    }

    /// Adds a miner to the territory.
    /// Returns an error including the given miner if the territory is already full.
    pub fn add_miner(&mut self, tick: &Tick, miner: Miner) -> Result<(), TerritoryFullError> {
        self.tick(tick);
        if self.miners < self.max_miners {
            self.miners += 1;
            Ok(())
        } else {
            Err(TerritoryFullError {
                max_miners: self.max_miners,
                miner,
            })
        }
    }

    /// Takes a miner from the territory.
    /// Returns `None` if there are no miners in the territory.
    pub fn take_miner(&mut self, tick: &Tick) -> Option<Miner> {
        self.tick(tick);
        if self.miners > 0 {
            self.miners -= 1;
            Some(Miner)
        } else {
            None
        }
    }

    /// Access the resources mined in this territory.
    pub fn resources(&mut self, tick: &Tick) -> &mut Resource<OreType> {
        self.tick(tick);
        &mut self.resources
    }
}
