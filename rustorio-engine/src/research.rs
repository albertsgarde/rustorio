//! Technologies can be unlocked by consuming science packs.
//! They usually unlock new recipes or further technologies.
//!
//! This module defines the the science pack resources and the `Technology` trait.

use std::{fmt::Debug, marker::PhantomData};

pub use rustorio_derive::{TechnologyEx, technology_doc};

use crate::{
    ResourceType, Sealed,
    recipe::{Recipe, RecipeEx},
    resources::{Bundle, Resource},
};

/// A technology can be unlocked out by calling the `research` method with the required science packs.
/// This will consume the science packs and the technology itself, and return whatever the technology unlocks, mostly recipes and other technologies.
pub trait Technology: Sealed + Debug + Sized + TechnologyEx {
    /// The name of the technology.
    const NAME: &'static str;
    /// How many of this technology's research points (`ResearchPoint<T>`) are needed to complete the research.
    const REQUIRED_RESEARCH_POINTS: u32 = Self::REQUIRED_RESEARCH_POINTS_EX;

    /// The reward for completing this technology.
    type Unlocks;

    /// Carries out the research by consuming the required science packs and the research itself, returning whatever this research unlocks.
    fn research(
        self,
        research_points: Bundle<ResearchPoint<Self>, { Self::REQUIRED_RESEARCH_POINTS }>,
    ) -> Self::Unlocks;
}

/// A trait handling the implementation details for a technology. Should only be implemented via the `#[derive(TechnologyEx)]` macro.
#[doc(hidden)]
pub trait TechnologyEx {
    /// The inputs needed to create one research point for this technology.
    /// Typically a tuple of multiple `RecipeItem`s.
    type Inputs: Debug;
    /// The type for `Self::InputAmountsType`, which is used to allow users to
    /// access the input amount for each of the input resource types, per recipe cycle.
    type InputAmountsType: Debug;
    /// A type guaranteed to contain exactly the input resources for one research point.
    /// Used in hand crafting.
    type InputBundle: Debug;
    /// Amount for each of the input resource types, per recipe cycle.
    const INPUT_AMOUNTS: Self::InputAmountsType;
    /// The amount of ticks it takes to create one research point for this technology.
    const POINT_RECIPE_TIME: u64;
    /// How many of this technology's research points (`ResearchPoint<T>`) are needed to complete the research.
    const REQUIRED_RESEARCH_POINTS_EX: u32;

    /// Factory function to create a new `Self::Inputs` with zero resources.
    fn new_inputs() -> Self::Inputs;

    /// Iterator helper over `Self::Inputs`.
    fn iter_inputs(items: &mut Self::Inputs)
    -> impl Iterator<Item = (&'static str, u32, &mut u32)>;
}

/// A resource type representing one research point for a specific `Technology`.
/// Use them in the `research` method of the corresponding `Technology` to unlock the technology.
#[derive(Debug)]
#[non_exhaustive]
pub struct ResearchPoint<T: Technology> {
    _marker: PhantomData<T>,
}

impl<T: Technology> Sealed for ResearchPoint<T> {}
impl<T: Technology> ResourceType for ResearchPoint<T> {
    const NAME: &'static str = T::NAME;
}

/// A recipe for producing research points for specific technologies.
#[derive(Debug)]
pub struct TechRecipe<T: Technology> {
    _marker: PhantomData<T>,
}

impl<T> Recipe for TechRecipe<T>
where
    T: Technology,
{
    const TIME: u64 = T::POINT_RECIPE_TIME;
    type Inputs = T::Inputs;
    type InputAmountsType = T::InputAmountsType;
    const INPUT_AMOUNTS: Self::InputAmountsType = T::INPUT_AMOUNTS;
    type Outputs = (Resource<ResearchPoint<T>>,);

    type OutputAmountsType = (u32,);

    const OUTPUT_AMOUNTS: (u32,) = (1,);

    fn new_inputs() -> Self::Inputs {
        T::new_inputs()
    }

    fn new_outputs() -> Self::Outputs {
        (Resource::new_empty(),)
    }
}

impl<T: Technology> RecipeEx for TechRecipe<T> {
    type InputBundle = T::InputBundle;
    type OutputBundle = Bundle<ResearchPoint<T>, 1>;

    fn new_output_bundle() -> Self::OutputBundle {
        Bundle::<ResearchPoint<T>, 1>::new()
    }

    fn iter_inputs(
        items: &mut Self::Inputs,
    ) -> impl Iterator<Item = (&'static str, u32, &mut u32)> {
        T::iter_inputs(items)
    }

    fn iter_outputs(
        items: &mut Self::Outputs,
    ) -> impl Iterator<Item = (&'static str, u32, &mut u32)> {
        [(ResearchPoint::<T>::NAME, 1u32, &mut items.0.amount)].into_iter()
    }
}

/// Creates a new `TechRecipe<T>` for use in a `Machine`.
/// Should not be reexported, as that would allow players to create research points for researches they have not unlocked yet.
pub const fn tech_recipe<T: Technology>() -> TechRecipe<T> {
    TechRecipe {
        _marker: PhantomData,
    }
}
