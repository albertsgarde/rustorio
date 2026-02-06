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

pub use rustorio_engine::mod_reexports::*;
