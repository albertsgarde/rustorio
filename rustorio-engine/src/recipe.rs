//! Recipes define all item transformations in the game via input items, output items, and time.

pub use rustorio_derive::{Recipe, RecipeEx, recipe_doc};

use crate::{
    ResourceType, Sealed,
    resources::{Bundle, Resource},
    tick::Tick,
};

/// A tuple of `Bundle<R, N>`.
pub trait MultiBundle: Sized + std::fmt::Debug {
    /// The corresponding tuple of `Resource<R>`.
    type AsResources: std::fmt::Debug;

    /// A tuple of `u32`, one for each resource; used for `AMOUNTS`.
    type AmountsType: std::fmt::Debug;
    /// Amount for each of the input resource types; used to help inspect the `Self` tuple.
    const AMOUNTS: Self::AmountsType;

    /// Create a new resource tuple with zero resources.
    fn new_empty() -> Self::AsResources;
    /// Count the number of bundle tuples available in the given resource tuple.
    fn bundle_count(res: &Self::AsResources) -> u32;
    /// Add the bundle tuple to the resource tuple.
    fn add(res: &mut Self::AsResources, bundle: Self);
    /// Pop a bundle tuple from a resource tuple, if there are enough resources.
    fn bundle(res: &mut Self::AsResources) -> Option<Self>;

    /// Factory function to create a new bundle tuple.
    #[doc(hidden)]
    fn new_bundle() -> Self;
    /// Iterate over the resources, giving direct mutable access to the amounts.
    #[doc(hidden)]
    fn iter(items: &mut Self::AsResources) -> impl Iterator<Item = (&'static str, u32, &mut u32)>;
}

// Special untupled case, for e.g. tech recipes that don't return a tuple.
impl<R1: ResourceType, const N1: u32> MultiBundle for Bundle<R1, N1> {
    type AsResources = (Resource<R1>,);

    type AmountsType = (u32,);
    const AMOUNTS: Self::AmountsType = (N1,);

    fn new_empty() -> Self::AsResources {
        <(Self,) as MultiBundle>::new_empty()
    }
    fn bundle_count(res: &Self::AsResources) -> u32 {
        <(Self,) as MultiBundle>::bundle_count(res)
    }
    fn add(res: &mut Self::AsResources, bundle: Self) {
        <(Self,) as MultiBundle>::add(res, (bundle,))
    }
    fn bundle(res: &mut Self::AsResources) -> Option<Self> {
        <(Self,) as MultiBundle>::bundle(res).map(|(r,)| r)
    }

    fn new_bundle() -> Self {
        <(Self,) as MultiBundle>::new_bundle().0
    }
    fn iter(items: &mut Self::AsResources) -> impl Iterator<Item = (&'static str, u32, &mut u32)> {
        <(Self,) as MultiBundle>::iter(items)
    }
}

impl<R1: ResourceType, const N1: u32> MultiBundle for (Bundle<R1, N1>,) {
    type AsResources = (Resource<R1>,);

    type AmountsType = (u32,);
    const AMOUNTS: Self::AmountsType = (N1,);

    fn new_empty() -> Self::AsResources {
        (Resource::new_empty(),)
    }
    fn bundle_count(res: &Self::AsResources) -> u32 {
        res.0.amount() / N1
    }
    fn add(res: &mut Self::AsResources, bundle: Self) {
        res.0 += bundle.0;
    }
    fn bundle(res: &mut Self::AsResources) -> Option<Self> {
        Some((res.0.bundle().ok()?,))
    }

    fn new_bundle() -> Self {
        (crate::resources::bundle(),)
    }
    fn iter(items: &mut Self::AsResources) -> impl Iterator<Item = (&'static str, u32, &mut u32)> {
        [(
            <R1 as ResourceType>::NAME,
            Self::AMOUNTS.0,
            crate::resources::resource_amount_mut(&mut items.0),
        )]
        .into_iter()
    }
}

impl<R1: ResourceType, const N1: u32, R2: ResourceType, const N2: u32> MultiBundle
    for (Bundle<R1, N1>, Bundle<R2, N2>)
{
    type AsResources = (Resource<R1>, Resource<R2>);

    type AmountsType = (u32, u32);
    const AMOUNTS: Self::AmountsType = (N1, N2);

    fn new_empty() -> Self::AsResources {
        (Resource::new_empty(), Resource::new_empty())
    }
    fn bundle_count(res: &Self::AsResources) -> u32 {
        std::cmp::min(res.0.amount() / N1, res.1.amount() / N2)
    }
    fn add(res: &mut Self::AsResources, bundle: Self) {
        res.0 += bundle.0;
        res.1 += bundle.1;
    }
    fn bundle(res: &mut Self::AsResources) -> Option<Self> {
        if res.0.amount() >= N1 && res.1.amount() >= N2 {
            Some((res.0.bundle().ok()?, res.1.bundle().ok()?))
        } else {
            None
        }
    }

    fn new_bundle() -> Self {
        (crate::resources::bundle(), crate::resources::bundle())
    }
    fn iter(items: &mut Self::AsResources) -> impl Iterator<Item = (&'static str, u32, &mut u32)> {
        [
            (
                <R1 as ResourceType>::NAME,
                Self::AMOUNTS.0,
                crate::resources::resource_amount_mut(&mut items.0),
            ),
            (
                <R2 as ResourceType>::NAME,
                Self::AMOUNTS.1,
                crate::resources::resource_amount_mut(&mut items.1),
            ),
        ]
        .into_iter()
    }
}

impl<
    R1: ResourceType,
    const N1: u32,
    R2: ResourceType,
    const N2: u32,
    R3: ResourceType,
    const N3: u32,
> MultiBundle for (Bundle<R1, N1>, Bundle<R2, N2>, Bundle<R3, N3>)
{
    type AsResources = (Resource<R1>, Resource<R2>, Resource<R3>);

    type AmountsType = (u32, u32, u32);
    const AMOUNTS: Self::AmountsType = (N1, N2, N3);

    fn new_empty() -> Self::AsResources {
        (
            Resource::new_empty(),
            Resource::new_empty(),
            Resource::new_empty(),
        )
    }
    fn bundle_count(res: &Self::AsResources) -> u32 {
        std::cmp::min(
            std::cmp::min(res.0.amount() / N1, res.1.amount() / N2),
            res.2.amount() / N3,
        )
    }
    fn add(res: &mut Self::AsResources, bundle: Self) {
        res.0 += bundle.0;
        res.1 += bundle.1;
        res.2 += bundle.2;
    }
    fn bundle(res: &mut Self::AsResources) -> Option<Self> {
        if res.0.amount() >= N1 && res.1.amount() >= N2 && res.2.amount() >= N3 {
            Some((
                res.0.bundle().ok()?,
                res.1.bundle().ok()?,
                res.2.bundle().ok()?,
            ))
        } else {
            None
        }
    }

    fn new_bundle() -> Self {
        (
            crate::resources::bundle(),
            crate::resources::bundle(),
            crate::resources::bundle(),
        )
    }
    fn iter(items: &mut Self::AsResources) -> impl Iterator<Item = (&'static str, u32, &mut u32)> {
        [
            (
                <R1 as ResourceType>::NAME,
                Self::AMOUNTS.0,
                crate::resources::resource_amount_mut(&mut items.0),
            ),
            (
                <R2 as ResourceType>::NAME,
                Self::AMOUNTS.1,
                crate::resources::resource_amount_mut(&mut items.1),
            ),
            (
                <R3 as ResourceType>::NAME,
                Self::AMOUNTS.2,
                crate::resources::resource_amount_mut(&mut items.2),
            ),
        ]
        .into_iter()
    }
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

    /// Factory function to create a new `Self::Inputs` with zero resources.
    fn new_inputs() -> Self::Inputs;

    /// Factory function to create a new `Self::Outputs` with zero resources.
    fn new_outputs() -> Self::Outputs;

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
    /// A type guaranteed to contain exactly the input resources for one recipe cycle.
    /// Used in handcrafting.
    type InputBundle: MultiBundle<AsResources = Self::Inputs>;
    /// A type guaranteed to contain exactly the output resources for one recipe cycle.
    /// Used in handcrafting.
    type OutputBundle: MultiBundle<AsResources = Self::Outputs>;

    /// Factory function to create a new `Self::InputBundle`.
    fn new_output_bundle() -> Self::OutputBundle {
        Self::OutputBundle::new_bundle()
    }

    /// Iterator helper over `Self::Inputs`.
    fn iter_inputs(
        items: &mut Self::Inputs,
    ) -> impl Iterator<Item = (&'static str, u32, &mut u32)> {
        Self::InputBundle::iter(items)
    }

    /// Iterator helper over `Self::Outputs`.
    fn iter_outputs(
        items: &mut Self::Outputs,
    ) -> impl Iterator<Item = (&'static str, u32, &mut u32)> {
        Self::OutputBundle::iter(items)
    }
}

/// A recipe that can be hand-crafted by the player.
pub trait HandRecipe: std::fmt::Debug + Sealed + RecipeEx {
    /// Crafts the recipe by consuming the input bundle and producing the output bundle.
    /// Advances the provided `Tick` by the recipe's time.
    fn craft(tick: &mut Tick, inputs: Self::InputBundle) -> Self::OutputBundle {
        let _ = inputs;
        tick.advance_by(Self::TIME);
        Self::new_output_bundle()
    }
}
