//! A recipe is a way of turning resources into other resources.
//! A specific recipe specifies the input and output resources, as well as the time it takes to complete the recipe.

use std::fmt::Debug;

use rustorio_engine::{
    Sealed,
    recipe::{Recipe, RecipeEx},
};

use crate::{
    research::RedScience,
    resources::{Copper, CopperOre, Iron, IronOre, Point},
};

/// Any recipe that implements this trait can be used in an [`Assembler`](crate::buildings::Assembler).
pub trait AssemblerRecipe: Debug + Sealed + RecipeEx {}

/// A recipe for crafting red science packs.
#[derive(Debug, Clone, Copy, Recipe, RecipeEx)]
#[recipe_inputs(
    (1, Iron),
    (1, Copper),
)]
#[recipe_outputs(
    (1, RedScience),
)]
#[recipe_ticks(5)]
pub struct RedScienceRecipe;
impl Sealed for RedScienceRecipe {}
impl AssemblerRecipe for RedScienceRecipe {}

/// The recipe you need to win! An [`Assembler`](crate::buildings::Assembler) recipe that creates points. Converts 4 iron and 4 copper into 1 point resource. Takes 20 ticks.
///
/// You can unlock this recipe by researching [`PointsTechnology`](crate::research::PointsTechnology).
#[derive(Debug, Clone, Copy, Recipe, RecipeEx)]
#[recipe_inputs(
    (4, Iron),
    (4, Copper),
)]
#[recipe_outputs(
    (1, Point),
)]
#[recipe_ticks(20)]
#[non_exhaustive]
pub struct PointRecipe;

impl Sealed for PointRecipe {}
impl AssemblerRecipe for PointRecipe {}

/// Any recipe that implements this trait can be used in a [`Furnace`](crate::buildings::Furnace).
pub trait FurnaceRecipe: Debug + Sealed + RecipeEx {}

/// A [`Furnace`](crate::buildings::Furnace) recipe that smelts iron ore into iron. Converts 2 iron ore into 1 iron. Takes 10 ticks.
#[derive(Debug, Clone, Copy, Recipe, RecipeEx)]
#[recipe_inputs(
    (2, IronOre),
)]
#[recipe_outputs(
    (1, Iron),
)]
#[recipe_ticks(10)]
pub struct IronSmelting;

impl Sealed for IronSmelting {}
impl FurnaceRecipe for IronSmelting {}

/// A [`Furnace`](crate::buildings::Furnace) recipe that smelts copper ore into copper. Converts 2 copper ore into 1 copper. Takes 10 ticks.
#[derive(Debug, Clone, Copy, Recipe, RecipeEx)]
#[recipe_inputs(
    (2, CopperOre),
)]
#[recipe_outputs(
    (1, Copper),
)]
#[recipe_ticks(10)]
pub struct CopperSmelting;

impl Sealed for CopperSmelting {}
impl FurnaceRecipe for CopperSmelting {}
