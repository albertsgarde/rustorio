//! Territories are where you can get ore.
//! To begin with you can mine by hand using the [`hand_mine`](Territory::hand_mine) function,
//! but later you can add [`Miner`]s to the territory to automate mining.

use std::{fmt::Display, marker::PhantomData};

use rustorio_engine::{
    ResourceType, Sealed, bundle,
    machine::Machine,
    mod_reexports::{Bundle, Resource, Tick},
    recipe::{HandRecipe, MultiBundle, Recipe, RecipeEx},
};

use crate::resources::{Copper, Iron};

/// Ore is mined every MINING_TICK_LENGTH ticks by each miner in a territory.
pub const MINING_TICK_LENGTH: u64 = 2;

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

// A recipe that encodes the operation of a territory: it takes no input.
#[derive(Debug)]
struct TerritoryRecipe<OreType: ResourceType>(PhantomData<OreType>);

impl<O: ResourceType> RecipeEx for TerritoryRecipe<O> {
    type InputBundle = ();
    type OutputBundle = Bundle<O, 1>;
}
impl<O: ResourceType> Recipe for TerritoryRecipe<O> {
    const TIME: u64 = MINING_TICK_LENGTH;

    type Inputs = <<Self as RecipeEx>::InputBundle as MultiBundle>::AsResources;
    type Outputs = <<Self as RecipeEx>::OutputBundle as MultiBundle>::AsResources;
    fn new_inputs() -> Self::Inputs {
        Default::default()
    }
    fn new_outputs() -> Self::Outputs {
        Default::default()
    }

    type InputAmountsType = <<Self as RecipeEx>::InputBundle as MultiBundle>::AmountsType;
    const INPUT_AMOUNTS: Self::InputAmountsType =
        <<Self as RecipeEx>::InputBundle as MultiBundle>::AMOUNTS;

    type OutputAmountsType = <<Self as RecipeEx>::OutputBundle as MultiBundle>::AmountsType;
    const OUTPUT_AMOUNTS: Self::OutputAmountsType =
        <<Self as RecipeEx>::OutputBundle as MultiBundle>::AMOUNTS;
}
impl<O: ResourceType> Sealed for TerritoryRecipe<O> {}
impl<O: ResourceType> HandRecipe for TerritoryRecipe<O> {}

/// A territory that can hold miners to mine a specific type of ore.
#[derive(Debug)]
#[non_exhaustive]
pub struct Territory<OreType: ResourceType> {
    /// The maximum number of miners allowed in the territory.
    max_miners: u32,
    machine: Machine<TerritoryRecipe<OreType>>,
}

impl<OreType: ResourceType> Territory<OreType> {
    /// Creates a new territory that can hold up to `max_miners` miners.
    pub(crate) fn new(tick: &Tick, max_miners: u32) -> Self {
        let mut machine = Machine::new(tick);
        *machine.productivity_mut(tick) = 0; // Start with 0 miners
        Self {
            max_miners,
            machine,
        }
    }

    /// Returns the the number of miner slots available in the territory.
    pub const fn max_miners(&self) -> u32 {
        self.max_miners
    }

    /// Returns the current number of miners in the territory.
    pub const fn num_miners(&self) -> u32 {
        self.machine.productivity()
    }

    fn tick(&mut self, tick: &Tick) {
        self.machine.outputs(tick); // `Machine::tick` isn't `pub` so we use this
    }

    /// Mines ore by hand, advancing the tick by [`MINING_TICK_LENGTH`] for each unit mined.
    pub fn hand_mine<const AMOUNT: u32>(&mut self, tick: &mut Tick) -> Bundle<OreType, AMOUNT> {
        // Would use `HandRecipe` but it doesn't supports doing it many times at once.
        self.tick(tick);
        tick.advance_by((u64::from(AMOUNT)) * MINING_TICK_LENGTH);
        bundle()
    }

    /// Adds a miner to the territory.
    /// Returns an error including the given miner if the territory is already full.
    pub fn add_miner(&mut self, tick: &Tick, miner: Miner) -> Result<(), TerritoryFullError> {
        let num_miners = self.machine.productivity_mut(tick);
        if *num_miners < self.max_miners {
            *num_miners += 1;
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
        let num_miners = self.machine.productivity_mut(tick);
        if *num_miners > 0 {
            *num_miners -= 1;
            Some(Miner)
        } else {
            None
        }
    }

    /// Access the resources mined in this territory.
    pub fn resources<'a>(&'a mut self, tick: &'a Tick) -> &'a mut Resource<OreType> {
        &mut self.machine.outputs(tick).0
    }
}
