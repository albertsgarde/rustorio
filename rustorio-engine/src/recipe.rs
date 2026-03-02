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
    fn bundle_count(res: &Self::AsResources) -> u32 {
        Self::iter(res)
            .map(|(_, expected, current)| current / expected)
            .min()
            .unwrap_or(u32::MAX)
    }
    /// Add the bundle tuple to the resource tuple.
    fn add(res: &mut Self::AsResources, bundle: Self);
    /// Pop a bundle tuple from a resource tuple, if there are enough resources.
    fn bundle(res: &mut Self::AsResources) -> Option<Self>;

    /// Create a new bundle tuple out of thin air.
    ///
    /// For use in mods only, cannot be used from the game.
    fn new_bundle(token: &TokenOfCreation) -> Self;

    /// Iterate over the resources, returning for each the resource name, per-bundle expected
    /// amount, and current amount.
    fn iter(items: &Self::AsResources) -> impl Iterator<Item = (&'static str, u32, u32)>;

    /// Iterate over the resources, giving direct mutable access to the amounts.
    ///
    /// For use in mods only, cannot be used from the game.
    fn iter_mut<'a>(
        token: &'a TokenOfCreation,
        items: &'a mut Self::AsResources,
    ) -> impl Iterator<Item = (&'static str, u32, &'a mut u32)>;
}

/// Multiply the amounts in a bundle tuple by the given constant.
pub trait MultiBundleMultiply<const N: u32>: MultiBundle {
    /// Bundle tuple identical to `Self` except with all amounts multiplied by `N`.
    type Multiplied: MultiBundle;
}
/// Transform a tuple of bundles by multiplying all bundle quantities by `N`.
pub type MultiplyMultiBundle<MB, const N: u32> = <MB as MultiBundleMultiply<N>>::Multiplied;

// Special untupled case, for e.g. tech recipes that don't return a tuple.
impl<R1: ResourceType, const N1: u32> MultiBundle for Bundle<R1, N1> {
    type AsResources = (Resource<R1>,);

    type AmountsType = (u32,);
    const AMOUNTS: Self::AmountsType = (N1,);

    fn add(res: &mut Self::AsResources, bundle: Self) {
        <(Self,) as MultiBundle>::add(res, (bundle,))
    }
    fn bundle(res: &mut Self::AsResources) -> Option<Self> {
        <(Self,) as MultiBundle>::bundle(res).map(|(r,)| r)
    }
    fn new_bundle(token: &TokenOfCreation) -> Self {
        <(Self,) as MultiBundle>::new_bundle(token).0
    }
    fn iter(items: &Self::AsResources) -> impl Iterator<Item = (&'static str, u32, u32)> {
        <(Self,) as MultiBundle>::iter(items)
    }
    fn iter_mut<'a>(
        token: &'a TokenOfCreation,
        items: &'a mut Self::AsResources,
    ) -> impl Iterator<Item = (&'static str, u32, &'a mut u32)> {
        <(Self,) as MultiBundle>::iter_mut(token, items)
    }
}

impl<const N: u32, R1: ResourceType, const N1: u32> MultiBundleMultiply<N> for Bundle<R1, N1>
where
    // See https://github.com/rust-lang/rust/issues/145069 for why `Copy`
    [(); { N1 * N } as usize]: Copy,
{
    type Multiplied = Bundle<R1, { N1 * N }>;
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

            fn add(res: &mut Self::AsResources, bundle: Self) {
                $(
                    res.$n += bundle.$n;
                )*
            }
            fn bundle(res: &mut Self::AsResources) -> Option<Self> {
                if Self::bundle_count(res) >= 1 {
                    Some((
                        $(
                            res.$n.bundle().ok()?,
                        )*
                    ))
                } else {
                    None
                }
            }
            #[allow(clippy::unused_unit)]
            fn new_bundle(token: &TokenOfCreation) -> Self {
                (
                    $(
                        replace_expr!($ty, crate::resources::bundle(token)),
                    )*
                )
            }
            fn iter(
                items: &Self::AsResources,
            ) -> impl Iterator<Item = (&'static str, u32, u32)> {
                [
                    $(
                        (
                            <$ty as ResourceType>::NAME,
                            Self::AMOUNTS.$n,
                            items.$n.amount(),
                        ),
                    )*
                ]
                .into_iter()
            }
            fn iter_mut<'a>(
                token: &'a TokenOfCreation,
                items: &'a mut Self::AsResources,
            ) -> impl Iterator<Item = (&'static str, u32, &'a mut u32)> {
                [
                    $(
                        (
                            <$ty as ResourceType>::NAME,
                            Self::AMOUNTS.$n,
                            items.$n.amount_mut(token),
                        ),
                    )*
                ]
                .into_iter()
            }
        }

        impl<
            const N: u32,
            $($ty: ResourceType, const $amount: u32),*
        > MultiBundleMultiply<N> for ($(Bundle<$ty, $amount>,)*)
        where
            // See https://github.com/rust-lang/rust/issues/145069 for why `Copy`
            $(
                [(); { $amount * N } as usize]: Copy,
            )*
        {
            type Multiplied = ($(Bundle<$ty, { $amount * N }>,)*);
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

/// A recipe that can be hand-crafted by the player.
pub trait HandRecipe: std::fmt::Debug + Sealed + Recipe {
    /// Crafts the recipe by consuming the input bundle and producing the output bundle.
    /// Advances the provided `Tick` by the recipe's time.
    fn craft(tick: &mut Tick, inputs: Self::InputBundle) -> Self::OutputBundle {
        let _ = inputs;
        tick.advance_by(Self::TIME);
        Self::OutputBundle::new_bundle(creation_token())
    }

    /// Crafts the recipe `N` times by consuming `N` input bundles and producing `N` output bundles.
    /// Advances the provided `Tick` by `N` times the recipe's time.
    ///
    /// Note: Call this function using an explicit `N`: `MyRecipe::craft_n::<42>(..)`; `N` can't be
    /// inferred and omitting it may give rise to confusing errors.
    fn craft_n<const N: u32>(
        tick: &mut Tick,
        inputs: MultiplyMultiBundle<Self::InputBundle, N>,
    ) -> MultiplyMultiBundle<Self::OutputBundle, N>
    where
        Self::InputBundle: MultiBundleMultiply<N>,
        Self::OutputBundle: MultiBundleMultiply<N, Multiplied: MultiBundle>,
    {
        let token = creation_token();
        let _ = inputs;
        tick.advance_by(Self::TIME * N as u64);
        <<Self::OutputBundle as MultiBundleMultiply<N>>::Multiplied as MultiBundle>::new_bundle(
            token,
        )
    }
}

#[test]
fn test_handcrafting() {
    use rustorio_derive::Recipe;

    use crate as rustorio_engine; // For the derive macros

    crate::resource_type!(Copper);
    crate::resource_type!(CopperWire);

    #[derive(Debug, Clone, Copy, Recipe)]
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
    impl HandRecipe for CopperWireRecipe {}

    let mut tick = Tick::start(10000);
    let mut copper: Resource<Copper> = crate::resource(creation_token(), 42);
    let _: (Bundle<CopperWire, 2>,) =
        CopperWireRecipe::craft(&mut tick, (copper.bundle().unwrap(),));
    let _: (Bundle<CopperWire, 10>,) =
        CopperWireRecipe::craft_n::<5>(&mut tick, (copper.bundle().unwrap(),));
}
