//! A game mode defines the starting resources and victory conditions for a game.
//!
//! This module defines the available game modes.

use rustorio_engine::{
    bundle,
    gamemodes::{GameMode, StartingResources},
    mod_reexports::Tick,
};

use crate::{
    Bundle,
    guide::Guide,
    research::SteelTechnology,
    resources::{Copper, CopperOre, Iron, IronOre, Point},
    territory::Territory,
};

/// Starting resources for the tutorial game mode.
pub struct TutorialStartingResources {
    /// Initial iron bundle.
    pub iron: Bundle<Iron, 10>,
    /// Initial territory to mine iron ore from.
    pub iron_territory: Territory<IronOre>,
    /// Initial territory to mine copper ore from.
    pub copper_territory: Territory<CopperOre>,
    /// The in-game guide that provides hints to the player.
    pub guide: Guide,
}

impl StartingResources for TutorialStartingResources {
    fn init(tick: &Tick) -> Self {
        Self {
            iron: bundle(),
            iron_territory: Territory::new(tick, 5),
            copper_territory: Territory::new(tick, 5),
            guide: Guide,
        }
    }
}

/// The tutorial game mode. Very short distance from start to victory, meant to teach the very most basic elements of play.
pub struct Tutorial;

impl GameMode for Tutorial {
    type StartingResources = TutorialStartingResources;
    type VictoryResources = Bundle<Copper, 4>;
}

/// Starting resources for the standard game mode. Includes 10 iron and the ability to research points.
pub struct StandardStartingResources {
    /// Initial iron bundle.
    pub iron: Bundle<Iron, 10>,
    /// Initial territory rich in iron ore.
    pub iron_territory: Territory<IronOre>,
    /// Initial territory rich in copper ore.
    pub copper_territory: Territory<CopperOre>,
    /// The first technology the player can research.
    pub steel_technology: SteelTechnology,
}
impl StartingResources for StandardStartingResources {
    fn init(tick: &Tick) -> Self {
        Self {
            iron: bundle(),
            iron_territory: Territory::new(tick, 20),
            copper_territory: Territory::new(tick, 20),
            steel_technology: SteelTechnology,
        }
    }
}

/// The standard game mode. Forces the player through the main gameplay mechanics.
pub struct Standard;

impl GameMode for Standard {
    type StartingResources = StandardStartingResources;
    type VictoryResources = Bundle<Point, 200>;
}
