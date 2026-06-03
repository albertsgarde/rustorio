#![warn(missing_docs)]
//! The base mod for Rustorio.
//! Contains all the main content of the game.
//! Your saves should depend on this crate.
//!
//! For more information, including help on getting started, see the [repo](https://github.com/albertsgarde/rustorio)

pub mod buildings;
pub mod gamemodes;
pub mod guide;
pub mod recipes;
pub mod research;
pub mod resources;
pub mod territory;

/// The tick is used to keep track of time in the game.
/// You can advance the game using the [`advance`](Tick::advance) method or similar.
/// Many functions and building methods require a [`Tick`] to be passed in, which allows them to update their state.
/// If a function takes a [`&mut Tick`](Tick), then the function will take time.
/// If a function merely takes a [`&Tick`](Tick), it will never advance the game time, but instead just roll forward its internal state to match the current tick.
///
/// # Examples
///
/// Let's say we have two furnaces the we want to fill with `iron_ore` and `copper_ore` respectively, and then advance time so they can smelt the ore into ingots:
/// ```
/// # use rustorio::{
/// #     Bundle, Tick,
/// #     buildings::Furnace,
/// #     recipes::{CopperSmelting, IronSmelting},
/// #     resources::{CopperOre, Iron, IronOre},
/// # };
/// # fn example(mut tick: Tick) {
/// # let token = rustorio_engine::resources::creation_token();
/// # let iron1: Bundle<Iron, 10> = rustorio_engine::bundle(token);
/// # let iron2: Bundle<Iron, 10> = rustorio_engine::bundle(token);
/// # let mut furnace1 = Furnace::build(&tick, IronSmelting, iron1);
/// # let mut furnace2 = Furnace::build(&tick, CopperSmelting, iron2);
/// # let iron_ore: Bundle<IronOre, 10> = rustorio_engine::bundle(token);
/// # let copper_ore: Bundle<CopperOre, 10> = rustorio_engine::bundle(token);
/// // Add ore to the furnaces at the current tick
/// furnace1.inputs(&tick).0 += iron_ore;
/// furnace2.inputs(&tick).0 += copper_ore;
/// // Advance time by 10 ticks so the furnaces can process some of the ore.
/// tick.advance_by(10);
/// // Now we can extract the smelted ingots from the furnaces
/// let iron_ingots = furnace1.outputs(&tick).0.empty();
/// let copper_ingots = furnace2.outputs(&tick).0.empty();
/// # let _ = (iron_ingots, copper_ingots);
/// # }
/// ```
pub use rustorio_engine::tick::Tick;
pub use rustorio_engine::{
    gamemodes::GameMode,
    play,
    recipe::{HandRecipe, Recipe},
    research::{ResearchPoint, TechRecipe, Technology},
    resources::{Bundle, InsufficientResourceError, Resource, ResourceType},
};
