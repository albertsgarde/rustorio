//! Recipes define all item transformations in the game via input items, output items, and time.

use std::ops::{Deref, DerefMut};

pub use rustorio_derive::{Recipe, RecipeEx, recipe_doc};

use crate::{ResourceType, resources::Resource};

/// One recipe item and its current amount inside a machine's input/output buffer.
#[derive(Debug)]
pub struct RecipeItem<const AMOUNT: u32, Content: ResourceType>(Resource<Content>);

impl<const AMOUNT: u32, Content: ResourceType> Default for RecipeItem<AMOUNT, Content> {
    fn default() -> Self {
        Self(Resource::new_empty())
    }
}

impl<const AMOUNT: u32, Content: ResourceType> Deref for RecipeItem<AMOUNT, Content> {
    type Target = Resource<Content>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const AMOUNT: u32, Content: ResourceType> DerefMut for RecipeItem<AMOUNT, Content> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const AMOUNT: u32, Content: ResourceType> RecipeItem<AMOUNT, Content> {
    /// Needed amount of the resource for one cycle of its recipe.
    pub const fn needed_amount(&self) -> u32 {
        AMOUNT
    }
}

/// Get a mutable reference to the inner amount.
/// This is not a function of `RecipeItem` to allow mods to choose to not export it to user code.
pub fn recipe_item_amount<const AMOUNT: u32, Content: ResourceType>(
    item: &mut RecipeItem<AMOUNT, Content>,
) -> &mut u32 {
    &mut item.0.amount
}

/// Basic recipe trait. A building's specific recipe trait can then be defined like
/// ```rust
/// trait AssemblerRecipe: rustorio_engine::recipe::Recipe + rustorio_engine::Sealed {}
/// ```
/// For example, one could define a recipe that takes three inputs and gives two outputs like:
/// ```rust
/// use rustorio_engine::{recipe::Recipe, resource_type};
///
/// resource_type!(Resource1);
/// resource_type!(Resource2);
/// resource_type!(Resource3);
/// resource_type!(Resource4);
/// resource_type!(Resource5);
///
/// #[derive(Recipe)]
/// #[recipe_inputs(
///     (10, Resource1),
///     (5, Resource2),
///     (1, Resource3),
/// )]
/// #[recipe_outputs(
///     (1, Resource4),
///     (100, Resource5),
/// )]
/// #[recipe_ticks(10)]
/// pub struct ThreeToTwoRecipe;
/// ```
/// The recipe will then take 10 ticks per cycle, consuming 10 `Resource1`, 5 `Resource2`,
/// and 1 `Resource3`, and produce 1 `Resource4` and 100 `Resource5`.
pub trait Recipe {
    /// Amount of ticks one cycle of the recipe takes to complete.
    const TIME: u64;

    /// Typically a tuple of multiple `RecipeTypes`, to define the inputs
    /// for one cycle of the recipe.
    type Inputs: std::fmt::Debug;

    /// Typically a tuple of multiple `RecipeTypes`, to define the outputs
    /// for one cycle of the recipe.
    type Outputs: std::fmt::Debug;

    /// The type for `Self::InputAmountsType`, which is used to allow users to
    /// access the input amount for each of the input resource types, per recipe cycle.
    type InputAmountsType: std::fmt::Debug;

    /// Amount for each of the input resource types, per recipe cycle.
    const INPUT_AMOUNTS: Self::InputAmountsType;

    /// The type for `Self::OuptutAmountsType`, which is used to allow users to
    /// access the output amount for each of the output resource types, per recipe cycle.
    type OutputAmountsType: std::fmt::Debug;

    /// Amount for each of the output resource types, per recipe cycle.
    const OUTPUT_AMOUNTS: Self::OutputAmountsType;
}

#[doc(hidden)]
pub trait RecipeEx: Recipe {
    /// Factory function to create a new `Self::Inputs` with zero resources.
    fn new_inputs() -> Self::Inputs;

    /// Factory function to create a new `Self::Outputs` with zero resources.
    fn new_outputs() -> Self::Outputs;

    /// Iterator helper over `Self::Inputs`.
    fn iter_inputs(items: &mut Self::Inputs)
    -> impl Iterator<Item = (&'static str, u32, &mut u32)>;

    /// Iterator helper over `Self::Outputs`.
    fn iter_outputs(
        items: &mut Self::Outputs,
    ) -> impl Iterator<Item = (&'static str, u32, &mut u32)>;
}
