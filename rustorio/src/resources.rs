//! Resources are the fundamental units of value in Rustorio.
//! Resources are held in either [`Resource`](crate::Resource) or [`Bundle`](crate::Bundle) objects.
//! [`Bundle`](crate::Bundle) objects are used to hold a fixed amount of a resource, while [`Resource`](crate::Resource) objects can hold any amount.
//!
//! Machine input and output buffers use [`Resource`](crate::Resource) because their amounts change over time.
//! Building costs, recipe inputs, research costs, and victory conditions use [`Bundle`](crate::Bundle) because they need exact amounts.
//!
//! When you have a flexible [`Resource`](crate::Resource) and need to pay an exact cost, extract a bundle from it:
//!
//! ```rust
//! # use rustorio::{Bundle, Resource, resources::Iron};
//! # let token = rustorio_engine::resources::creation_token();
//! # let mut furnace_output = rustorio_engine::resource::<Iron>(token, 12);
//! let mut iron: Resource<Iron> = Resource::new_empty();
//! iron += furnace_output.empty();
//!
//! let ten_iron: Bundle<Iron, 10> = iron.bundle().unwrap();
//! # assert_eq!(iron.amount(), 2);
//! ```
//!
//! The `.bundle::<N>()` method removes `N` items from the resource buffer and
//! returns a [`Bundle<T, N>`](crate::Bundle). If Rust can infer `N` from the
//! function you pass it to, you can usually write `.bundle().unwrap()`.
//!
//! This module defines the core resources used in Rustorio.

use rustorio_engine::documented_resource_type;

documented_resource_type!(
    /// Raw iron ore mined from the ground.
    /// Can be smelted into iron ingots using a [`Furnace`](crate::buildings::Furnace).
    IronOre
);

documented_resource_type!(
    /// Refined iron ingots produced by smelting [iron ore](crate::resources::IronOre).
    /// Used in various recipes and to build structures.
    Iron
);

documented_resource_type!(
    /// Raw copper ore mined from the ground.
    /// Can be smelted into copper ingots using a [`Furnace`](crate::buildings::Furnace).
    CopperOre
);

documented_resource_type!(
    /// Refined copper ingots produced by smelting [copper ore](crate::resources::CopperOre).
    /// Used in various recipes and to build structures.
    Copper
);

documented_resource_type!(
    /// Made by smelting [`iron`](crate::resources::Iron) again in a [`Furnace`](crate::buildings::Furnace).
    /// One of the two components for making [`Point`]s.
    Steel
);

documented_resource_type!(
    /// Wire made from [copper](crate::resources::Copper).
    /// Copper wire used for making [`ElectronicCircuit`]s.
    CopperWire
);

documented_resource_type!(
    /// Circuits made from [iron](crate::resources::Iron) and [copper wire](crate::resources::CopperWire).
    /// Used to make [`Assembler`](crate::buildings::Assembler)s and a primary component of [`Point`]s.
    ElectronicCircuit
);

documented_resource_type!(
    /// Used to win the game in the standard game mode.
    /// Made from [`steel`](crate::resources::Steel) and [`electronic circuits`](crate::resources::ElectronicCircuit).
    Point
);
