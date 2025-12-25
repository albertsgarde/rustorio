#![feature(generic_const_exprs)]
#![allow(incomplete_features)] // silence the “still incomplete” lint
#![warn(missing_docs)]
#![warn(clippy::missing_const_for_fn)]
//! The core engine for Rustorio.
//! Only relevant if you are writing a mod for Rustorio.
//! To play the game, depend on the `rustorio` crate instead.
//!
//! For more information, see the [repo](https://github.com/albertsgarde/rustorio)

pub mod gamemodes;
pub mod machine;
pub mod recipe;
pub mod research;
mod resources;
mod tick;

use std::sync::Once;

pub use crate::resources::{ResourceType, bundle, resource};
use crate::{
    gamemodes::{GameMode, StartingResources},
    tick::Tick,
};

static ONCE: Once = Once::new();

/// Runs your play. If it is run multiple times, it will panic. This is to prevent using multiple threads to cheat.
pub fn play<G: GameMode>(main: fn(Tick, G::StartingResources) -> (Tick, G::VictoryResources)) -> ! {
    let mut call_once_ran = false;
    ONCE.call_once(|| call_once_ran = true);
    if !call_once_ran {
        panic!(
            "play() can only be called once per program execution to prevent cheating via multithreading."
        );
    }
    let tick = Tick::start();
    let start_resources = G::StartingResources::init(&tick);
    let (tick, _points) = main(tick, start_resources);
    println!("You won in {} ticks!", tick.cur());
    std::process::exit(0);
}

/// A trait to prevent players from implementing certain traits.
/// Should not be reexported in mods.
pub trait Sealed {}

/// These are the items that a player should have direct access to.
/// Should be glob reexported at the top level of mods.
///
/// ```rust
/// pub use rustorio_engine::mod_reexports::*;
/// ```
pub mod mod_reexports {
    pub use crate::{
        gamemodes::GameMode,
        play,
        recipe::Recipe,
        research::{RedScience, Technology},
        resources::{Bundle, InsufficientResourceError, Resource},
        tick::Tick,
    };
}
