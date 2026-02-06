//! A game mode defines the starting resources and victory conditions for a game.

use crate::tick::Tick;

/// The starting resources of a game mode. These are provided to the player at the beginning of the game.
pub trait StartingResources {
    /// Called once at the start of the game before control is handed to the player to create the starting resources.
    ///
    /// # Parameters
    /// - `tick`: The current game tick. Since this method is called at the start of the game, this will always be tick 0.
    fn init(tick: &Tick) -> Self;
}

/// A game mode defines the starting resources and victory conditions for a game.
pub trait GameMode {
    /// Starting resources provided to the player at the beginning of the game.
    #[allow(private_bounds)]
    type StartingResources: StartingResources;
    /// Resources required to achieve victory.
    type VictoryResources;
}
