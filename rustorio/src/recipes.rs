//! A recipe is a way of turning resources into other resources.
//! Some recipes can be performed by hand, but most can be automated using buildings like the [`Assembler`](crate::buildings::Assembler) or the [`Furnace`](crate::buildings::Furnace).
//! To figure out where the recipe can be performed, look at the trait impls.
//! For example, the [`CopperWireRecipe`] implements the [`AssemblerRecipe`] and [`HandRecipe`] traits, meaning it can be performed by hand and in an [`Assembler`](crate::buildings::Assembler).
//!
//! The documentation for each recipe struct provides more details about the recipe, including inputs, outputs and time taken.

use std::fmt::Debug;

use rustorio_engine::{
    Sealed,
    recipe::{HandRecipe, Recipe, RecipeEx, recipe_doc},
};

use crate::{
    research::RedScience,
    resources::{Copper, CopperOre, CopperWire, ElectronicCircuit, Iron, IronOre, Point, Steel},
};

/// Any recipe that implements this trait can be used in an [`Assembler`](crate::buildings::Assembler).
pub trait AssemblerRecipe<'a>: Debug + Sealed + RecipeEx<'a> {}

#[derive(Debug, Clone, Copy, Recipe, RecipeEx)]
#[recipe_doc]
#[recipe_inputs(
    (1, Copper),
)]
#[recipe_outputs(
    (2, CopperWire),
)]
#[recipe_ticks(1)]
pub struct CopperWireRecipe;
impl Sealed for CopperWireRecipe {}
impl<'a> AssemblerRecipe<'a> for CopperWireRecipe {}
impl<'a> HandRecipe<'a> for CopperWireRecipe {}

#[derive(Debug, Clone, Copy, Recipe, RecipeEx)]
#[recipe_doc]
#[recipe_inputs(
    (1, Iron),
    (2, CopperWire),
)]
#[recipe_outputs(
    (1, ElectronicCircuit),
)]
#[recipe_ticks(3)]
pub struct ElectronicCircuitRecipe;
impl Sealed for ElectronicCircuitRecipe {}
impl<'a> AssemblerRecipe<'a> for ElectronicCircuitRecipe {}
impl<'a> HandRecipe<'a> for ElectronicCircuitRecipe {}

/// A recipe for crafting red science packs.
#[derive(Debug, Clone, Copy, Recipe, RecipeEx)]
#[recipe_doc]
#[recipe_inputs(
    (1, Iron),
    (1, ElectronicCircuit),
)]
#[recipe_outputs(
    (1, RedScience),
)]
#[recipe_ticks(10)]
pub struct RedScienceRecipe;
impl Sealed for RedScienceRecipe {}
impl<'a> AssemblerRecipe<'a> for RedScienceRecipe {}
impl<'a> HandRecipe<'a> for RedScienceRecipe {}

/// The recipe you need to win! An [`Assembler`](crate::buildings::Assembler) recipe that creates points.
///
/// You can unlock this recipe by researching [`PointsTechnology`](crate::research::PointsTechnology).
#[derive(Debug, Clone, Copy, Recipe, RecipeEx)]
#[recipe_doc]
#[recipe_inputs(
    (4, ElectronicCircuit),
    (1, Steel),
)]
#[recipe_outputs(
    (1, Point),
)]
#[recipe_ticks(20)]
#[non_exhaustive]
pub struct PointRecipe;
impl Sealed for PointRecipe {}
impl<'a> AssemblerRecipe<'a> for PointRecipe {}

/// Any recipe that implements this trait can be used in a [`Furnace`](crate::buildings::Furnace).
pub trait FurnaceRecipe<'a>: Debug + Sealed + RecipeEx<'a> {}

/// A [`Furnace`](crate::buildings::Furnace) recipe that smelts iron ore into iron.
#[derive(Debug, Clone, Copy, Recipe, RecipeEx)]
#[recipe_doc]
#[recipe_inputs(
    (1, IronOre),
)]
#[recipe_outputs(
    (1, Iron),
)]
#[recipe_ticks(6)]
pub struct IronSmelting;
impl Sealed for IronSmelting {}
impl<'a> FurnaceRecipe<'a> for IronSmelting {}

/// A [`Furnace`](crate::buildings::Furnace) recipe that smelts copper ore into copper.
#[derive(Debug, Clone, Copy, Recipe, RecipeEx)]
#[recipe_doc]
#[recipe_inputs(
    (1, CopperOre),
)]
#[recipe_outputs(
    (1, Copper),
)]
#[recipe_ticks(6)]
pub struct CopperSmelting;
impl Sealed for CopperSmelting {}
impl<'a> FurnaceRecipe<'a> for CopperSmelting {}

/// A [`Furnace`](crate::buildings::Furnace) recipe that smelts iron into steel.
#[derive(Debug, Clone, Copy, Recipe, RecipeEx)]
#[recipe_doc]
#[recipe_inputs(
    (5, Iron),
)]
#[recipe_outputs(
    (1, Steel),
)]
#[recipe_ticks(30)]
#[non_exhaustive]
pub struct SteelSmelting;
impl Sealed for SteelSmelting {}
impl<'a> FurnaceRecipe<'a> for SteelSmelting {}
