//! Technologies can be unlocked by consuming science packs.
//! They usually unlock new recipes or further technologies.
//!
//! This module defines the the science pack resources and the `Technology` trait.

use std::{fmt::Debug, marker::PhantomData};

pub use rustorio_derive::{TechnologyEx, technology_doc};

use crate::{
    ResourceType, Sealed,
    recipe::{MultiBundle, MultiBundleEx, Recipe, RecipeEx},
    resources::{Bundle, Resource},
    tick::Tick,
};

/// A technology can be unlocked out by calling the `research` method with the required science packs.
/// This will consume the science packs and the technology itself, and return whatever the technology unlocks, mostly recipes and other technologies.
pub trait Technology<'t>: Sealed + Debug + Sized + TechnologyEx<'t> {
    /// The name of the technology.
    const NAME: &'static str;
    /// How many of this technology's research points (`ResearchPoint<T>`) are needed to complete the research.
    const REQUIRED_RESEARCH_POINTS: u32 = Self::REQUIRED_RESEARCH_POINTS_EX;

    /// The reward for completing this technology.
    type Unlocks;

    /// Carries out the research by consuming the required science packs and the research itself, returning whatever this research unlocks.
    fn research(
        self,
        research_points: Bundle<'t, ResearchPoint<'t, Self>, { Self::REQUIRED_RESEARCH_POINTS }>,
    ) -> Self::Unlocks;
}

/// A trait handling the implementation details for a technology. Should only be implemented via the `#[derive(TechnologyEx)]` macro.
#[doc(hidden)]
pub trait TechnologyEx<'t> {
    /// A type guaranteed to contain exactly the input resources for one research point.
    /// Used in hand crafting.
    type InputBundle: MultiBundleEx;
    /// The amount of ticks it takes to create one research point for this technology.
    const POINT_RECIPE_TIME: u64;
    /// How many of this technology's research points (`ResearchPoint<T>`) are needed to complete the research.
    const REQUIRED_RESEARCH_POINTS_EX: u32;
}

/// A resource type representing one research point for a specific `Technology`.
/// Use them in the `research` method of the corresponding `Technology` to unlock the technology.
#[derive(Debug)]
#[non_exhaustive]
pub struct ResearchPoint<'t, T: Technology<'t>> {
    _marker: PhantomData<&'t T>,
}

impl<'t, T: Technology<'t>> Sealed for ResearchPoint<'t, T> {}
impl<'t, T: Technology<'t>> ResourceType for ResearchPoint<'t, T> {
    const NAME: &'static str = T::NAME;
}

/// A recipe for producing research points for specific technologies.
#[derive(Debug)]
pub struct TechRecipe<'t, T: Technology<'t>> {
    _marker: PhantomData<&'t T>,
}

impl<'t, T> Recipe<'t> for TechRecipe<'t, T>
where
    T: Technology<'t>,
{
    const TIME: u64 = T::POINT_RECIPE_TIME;
    type Inputs = <T::InputBundle as MultiBundle>::AsResources;
    type InputAmountsType = <T::InputBundle as MultiBundle>::AmountsType;
    const INPUT_AMOUNTS: Self::InputAmountsType = <T::InputBundle as MultiBundle>::AMOUNTS;
    type Outputs = (Resource<'t, ResearchPoint<'t, T>>,);

    type OutputAmountsType = (u32,);

    const OUTPUT_AMOUNTS: (u32,) = (1,);

    fn new_inputs() -> Self::Inputs {
        Default::default()
    }

    fn new_outputs() -> Self::Outputs {
        Default::default()
    }
}

impl<'t, T: Technology<'t>> RecipeEx<'t> for TechRecipe<'t, T> {
    type InputBundle = T::InputBundle;
    type OutputBundle = Bundle<'t, ResearchPoint<'t, T>, 1>;
}

/// Creates a new `TechRecipe<T>` for use in a `Machine`.
/// Should not be reexported, as that would allow players to create research points for researches they have not unlocked yet.
pub const fn tech_recipe<'t, T: Technology<'t>>() -> TechRecipe<'t, T> {
    TechRecipe {
        _marker: PhantomData,
    }
}
