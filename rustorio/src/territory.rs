//! Territories are where you can get ore.
//! To begin with you can mine by hand using the [`hand_mine`](Territory::hand_mine) function,
//! but later you can add [`Miner`]s to the territory to automate mining.

use std::fmt::Display;

use rustorio_engine::{
    ResourceType, bundle,
    mod_reexports::{Bundle, Resource, Tick},
    resource,
};

use crate::resources::{Copper, CopperOre, Iron, IronOre};

/// Sub-trait of `ResourceType` for ores that can be mined in a territory.
pub trait OreType: ResourceType {
    /// How long it takes to mine one unit of ore.
    const MINING_TIME: u64;
}

impl OreType for IronOre {
    const MINING_TIME: u64 = 2;
}

impl OreType for CopperOre {
    const MINING_TIME: u64 = 2;
}

/// A miner that can be added to a territory to mine resources.
#[derive(Debug)]
#[non_exhaustive]
pub struct Miner;

impl Miner {
    /// Builds a new miner. Requires 10 [iron](crate::resources::Iron) and 5 [copper](crate::resources::Copper) to build.
    pub const fn build(iron: Bundle<Iron, 10>, copper: Bundle<Copper, 5>) -> Self {
        let _ = (iron, copper);
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
pub struct Territory<Ore: ResourceType> {
    tick: u64,
    /// The maximum number of miners allowed in the territory.
    max_miners: u32,
    miners: Vec<u64>,
    resources: Resource<Ore>,
}

impl<Ore: OreType> Territory<Ore> {
    /// Creates a new territory that can hold up to `max_miners` miners.
    pub(crate) const fn new(tick: &Tick, max_miners: u32) -> Self {
        Self {
            tick: tick.cur(),
            max_miners,
            miners: Vec::new(),
            resources: Resource::new_empty(),
        }
    }

    /// Returns the the number of miner slots available in the territory.
    pub const fn max_miners(&self) -> u32 {
        self.max_miners
    }

    /// Returns the current number of miners in the territory.
    pub const fn num_miners(&self) -> u32 {
        self.miners.len() as u32
    }

    fn tick(&mut self, tick: &Tick) {
        assert!(tick.cur() >= self.tick, "Tick went backwards");
        for miner_tick in &mut self.miners {
            *miner_tick += tick.cur() - self.tick;
            self.resources += resource(
                u32::try_from(*miner_tick / Ore::MINING_TIME)
                    .expect("Number of resources exceeds u32::MAX."),
            );
            *miner_tick %= Ore::MINING_TIME;
        }
    }

    /// Mines ore by hand, advancing the tick by [`OreType::MINING_TIME`] for each unit mined.
    pub fn hand_mine<const AMOUNT: u32>(&mut self, tick: &mut Tick) -> Bundle<Ore, AMOUNT> {
        self.tick(tick);
        tick.advance_by((u64::from(AMOUNT)) * Ore::MINING_TIME);
        bundle()
    }

    /// Adds a miner to the territory.
    /// Returns an error including the given miner if the territory is already full.
    pub fn add_miner(&mut self, tick: &Tick, miner: Miner) -> Result<(), TerritoryFullError> {
        self.tick(tick);
        if self.miners.len()
            < usize::try_from(self.max_miners).expect("max_miners exceeds usize::MAX.")
        {
            self.miners.push(0);
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
    /// Always returns the miner furthest from producing a resource.
    pub fn take_miner(&mut self, tick: &Tick) -> Option<Miner> {
        self.tick(tick);
        if let Some(best_index) = self
            .miners
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(i, _)| i)
        {
            self.miners.swap_remove(best_index);
            Some(Miner)
        } else {
            None
        }
    }

    /// Access the resources mined in this territory.
    pub fn resources(&mut self, tick: &Tick) -> &mut Resource<Ore> {
        self.tick(tick);
        &mut self.resources
    }
}
