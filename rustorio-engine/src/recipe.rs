//! Recipes define all item transformations in the game via input items, output items, and time.

pub use rustorio_derive::{Recipe, recipe_doc};

use crate::{
    ResourceType, Sealed,
    resources::{Bundle, Resource, TokenOfCreation, creation_token},
    tick::Tick,
};

/// A tuple of `Bundle<R, N>`.
pub trait MultiBundle: Sized + std::fmt::Debug {
    /// The corresponding tuple of `Resource<R>`.
    type AsResources: Default + std::fmt::Debug;

    /// A tuple of `u32`, one for each resource; used for `AMOUNTS`.
    type AmountsType: std::fmt::Debug;
    /// Amount for each of the input resource types; used to help inspect the `Self` tuple.
    const AMOUNTS: Self::AmountsType;

    /// Count the number of bundle tuples available in the given resource tuple.
    fn bundle_count(res: &Self::AsResources) -> u32;
    /// Add the bundle tuple to the resource tuple.
    fn add(res: &mut Self::AsResources, bundle: Self);
    /// Pop a bundle tuple from a resource tuple, if there are enough resources.
    fn bundle(res: &mut Self::AsResources) -> Option<Self>;
}

#[doc(hidden)]
pub trait MultiBundleEx: MultiBundle {
    /// Factory function to create a new bundle tuple.
    fn new_bundle(token: &TokenOfCreation) -> Self;
    /// Iterate over the resources, giving direct mutable access to the amounts.
    fn iter<'a>(
        token: &'a TokenOfCreation,
        items: &'a mut Self::AsResources,
    ) -> impl Iterator<Item = (&'static str, u32, &'a mut u32)>;
}

// Special untupled case, for e.g. tech recipes that don't return a tuple.
impl<R1: ResourceType, const N1: u32> MultiBundle for Bundle<R1, N1> {
    type AsResources = (Resource<R1>,);

    type AmountsType = (u32,);
    const AMOUNTS: Self::AmountsType = (N1,);

    fn bundle_count(res: &Self::AsResources) -> u32 {
        <(Self,) as MultiBundle>::bundle_count(res)
    }
    fn add(res: &mut Self::AsResources, bundle: Self) {
        <(Self,) as MultiBundle>::add(res, (bundle,))
    }
    fn bundle(res: &mut Self::AsResources) -> Option<Self> {
        <(Self,) as MultiBundle>::bundle(res).map(|(r,)| r)
    }
}
impl<R1: ResourceType, const N1: u32> MultiBundleEx for Bundle<R1, N1> {
    fn new_bundle(token: &TokenOfCreation) -> Self {
        <(Self,) as MultiBundleEx>::new_bundle(token).0
    }
    fn iter<'a>(
        token: &'a TokenOfCreation,
        items: &'a mut Self::AsResources,
    ) -> impl Iterator<Item = (&'static str, u32, &'a mut u32)> {
        <(Self,) as MultiBundleEx>::iter(token, items)
    }
}

macro_rules! replace_expr {
    ($_t:tt, $($sub:tt)*) => {
        $($sub)*
    };
}

macro_rules! impl_multi_bundle {
    ($($n:tt $ty:ident $amount:ident),*) => {
        #[allow(unused)]
        impl<
            $(
                $ty: ResourceType,
                const $amount: u32,
            )*
        > MultiBundle for
            (
                $(Bundle<$ty, $amount>,)*
            )
        {
            type AsResources =
                (
                    $(Resource<$ty>,)*
                );

            type AmountsType =
                (
                    $(replace_expr!($amount, u32),)*
                );
            const AMOUNTS: Self::AmountsType =
                (
                    $($amount,)*
                );

            fn bundle_count(res: &Self::AsResources) -> u32 {
                [
                    $(
                        res.$n.amount() / $amount,
                    )*
                ].into_iter().min().unwrap_or(u32::MAX)
            }
            fn add(res: &mut Self::AsResources, bundle: Self) {
                $(
                    res.$n += bundle.$n;
                )*
            }
            fn bundle(res: &mut Self::AsResources) -> Option<Self> {
                let enough_resources = true $(
                    && res.$n.amount() >= $amount
                )*;
                if enough_resources {
                    Some((
                        $(
                            res.$n.bundle().ok()?,
                        )*
                    ))
                } else {
                    None
                }
            }
        }

        #[allow(unused)]
        impl<
            $($ty: ResourceType, const $amount: u32),*
        > MultiBundleEx for ($(Bundle<$ty, $amount>,)*)
        {
            #[allow(clippy::unused_unit)]
            fn new_bundle(token: &TokenOfCreation) -> Self {
                (
                    $(
                        replace_expr!($ty, crate::resources::bundle(token)),
                    )*
                )
            }
            fn iter<'a>(
                token: &'a TokenOfCreation,
                items: &'a mut Self::AsResources,
            ) -> impl Iterator<Item = (&'static str, u32, &'a mut u32)> {
                [
                    $(
                        (
                            <$ty as ResourceType>::NAME,
                            Self::AMOUNTS.$n,
                            crate::resources::resource_amount_mut(token, &mut items.$n),
                        ),
                    )*
                ]
                .into_iter()
            }
        }
    };
}

impl_multi_bundle!();
impl_multi_bundle!(0 R1 N1);
impl_multi_bundle!(0 R1 N1, 1 R2 N2);
impl_multi_bundle!(0 R1 N1, 1 R2 N2, 2 R3 N3);

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

    /// A tuple of bundles that describes the input resources for one recipe cycle.
    type InputBundle: MultiBundle<AsResources = Self::InputResources>;

    /// A tuple of bundles that describes the output resources for one recipe cycle.
    type OutputBundle: MultiBundle<AsResources = Self::OutputResources>;

    /// A tuple of `Resource<R>` corresponding to the input bundles.
    type InputResources: std::fmt::Debug + Default;

    /// A tuple of `Resource<R>` corresponding to the output bundles.
    type OutputResources: std::fmt::Debug + Default;
}

#[doc(hidden)]
pub trait RecipeEx: Recipe<InputBundle: MultiBundleEx, OutputBundle: MultiBundleEx> {}
impl<R: Recipe<InputBundle: MultiBundleEx, OutputBundle: MultiBundleEx>> RecipeEx for R {}

/// A recipe that can be hand-crafted by the player.
pub trait HandRecipe: std::fmt::Debug + Sealed + RecipeEx {
    /// Crafts the recipe by consuming the input bundle and producing the output bundle.
    /// Advances the provided `Tick` by the recipe's time.
    fn craft(tick: &mut Tick, inputs: Self::InputBundle) -> Self::OutputBundle {
        let _ = inputs;
        tick.advance_by(Self::TIME);
        Self::OutputBundle::new_bundle(creation_token())
    }
}
