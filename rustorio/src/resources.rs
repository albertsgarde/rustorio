//! Resources are the fundamental units of value in Rustorio.
//! Resources are held in either [`Resource`](crate::Resource) or [`Bundle`](crate::Bundle) objects.
//! [`Bundle`](crate::Bundle) objects are used to hold a fixed amount of a resource, while [`Resource`](crate::Resource) objects can hold any amount.
//!
//! This module defines the core resources used in Rustorio.

use rustorio_engine::resource_type;

resource_type!(
    /// Raw iron ore mined from the ground.
    /// Can be smelted into iron ingots using a [`Furnace`](crate::buildings::Furnace).
    IronOre
);

resource_type!(
    /// Refined iron ingots produced by smelting [iron ore](crate::resources::IronOre).
    /// Used in various recipes and to build structures.
    Iron
);

resource_type!(
    /// Raw copper ore mined from the ground.
    /// Can be smelted into copper ingots using a [`Furnace`](crate::buildings::Furnace).
    CopperOre
);

resource_type!(
    /// Refined copper ingots produced by smelting [copper ore](crate::resources::CopperOre).
    /// Used in various recipes and to build structures.
    Copper
);

resource_type!(
    /// Made by smelting [`iron`](crate::resources::Iron) again in a [`Furnace`](crate::buildings::Furnace).
    /// One of the two components for making [`Point`]s.
    Steel
);

resource_type!(
    /// Wire made from [copper](crate::resources::Copper).
    /// Copper wire used for making [`ElectronicCircuit`]s.
    CopperWire
);

resource_type!(
    /// Circuits made from [iron](crate::resources::Iron) and [copper wire](crate::resources::CopperWire).
    /// Used to make [`Assembler`](crate::buildings::Assembler)s and a primary component of [`Point`]s.
    ElectronicCircuit
);

resource_type!(
    /// Used to win the game in the standard game mode.
    /// Made from [`steel`](crate::resources::Steel) and [`electronic circuits`](crate::resources::ElectronicCircuit).
    Point
);
