//! Resources are the fundamental units of value in Rustorio.
//! Resources are held in either [`Resource`] or [`Bundle`] objects.
//! [`Bundle`] objects are used to hold a fixed amount of a resource, while [`Resource`] objects can hold any amount.
//!
//! This is the core engine module for resource primitives. Most players should use the
//! player-facing resource docs in the `rustorio` crate instead. This module is mainly
//! useful for engine development and mods that need to define new resource types or build
//! APIs around resources.
//!
//! Engine and mod code can create resources with a [`TokenOfCreation`]:
//!
//! ```rust
//! # use rustorio_engine::{ResourceType, resource_type};
//! # use rustorio_engine::mod_reexports::{Bundle, Resource};
//!
//! resource_type!(Iron);
//!
//! let creation_token = rustorio_engine::resources::creation_token();
//!
//! let mut iron = Resource::<Iron>::new_empty();
//! iron += rustorio_engine::bundle::<Iron, 15>(creation_token);
//! ```
//!
//! This module defines the core resource APIs, including the [`ResourceType`] trait,
//! the [`Resource`] and [`Bundle`] structs, and the [`resource_type`] macro.

use std::{
    fmt::{Debug, Display},
    iter::Sum,
    marker::PhantomData,
    ops::{Add, AddAssign},
};

use crate::Sealed;

/// A type that represents a specific kind of resource in the game.
/// Implementors of this trait represent different resource types, such as iron, copper, or science packs.
/// Only useful as a type parameter; has no associated methods.
///
/// ## Modding
///
/// To define a new resource type, use the `resource_type!` macro.
pub trait ResourceType: Sealed + Debug {
    /// A human readable name for this resource type.
    const NAME: &'static str;
}

/// Macro to define a new resource type.
///
/// # Example
/// ```rust
/// use rustorio_engine::resource_type;
/// resource_type!(
///     /// Gold ingots used for advanced crafting.
///     Gold
/// );
/// ```
///
/// See the `rustorio::resources` docs for a bunch of uses.
#[macro_export]
macro_rules! resource_type {

    ($(#[$outer:meta])*
    $name:ident) => {
        $(#[$outer])*
        #[derive(Debug)]
        pub struct $name;
        impl $crate::Sealed for $name {}
        impl $crate::ResourceType for $name {
            const NAME: &'static str = stringify!($name);
        }
    };
}

pub use resource_type;

/// Error returned when there are insufficient resources in a [`Resource`] to fulfill a request.
#[derive(Debug, Clone)]
pub struct InsufficientResourceError<Resource>
where
    Resource: ResourceType,
{
    /// The amount of resource that was requested.
    pub requested_amount: u32,
    /// The amount of resource that was actually available.
    pub available_amount: u32,
    phantom: PhantomData<Resource>,
}

impl<Resource> InsufficientResourceError<Resource>
where
    Resource: ResourceType,
{
    /// Creates a new `InsufficientResourceError`.
    pub const fn new(requested_amount: u32, available_amount: u32) -> Self {
        Self {
            requested_amount,
            available_amount,
            phantom: PhantomData,
        }
    }
}

impl<Resource> Display for InsufficientResourceError<Resource>
where
    Resource: ResourceType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Insufficient {:?}: requested {}, but only {} available",
            Resource::NAME,
            self.requested_amount,
            self.available_amount
        )
    }
}

/// Holds an arbitrary amount of a resource.
/// A [`Resource`] object can be split into smaller parts, combined or [`Bundle`]s can be extracted from them.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[must_use = "This resource is being dropped without being used. If this is intentional, use the `let _ = resource;` pattern to silence this warning."]
pub struct Resource<R>
where
    R: ResourceType,
{
    /// The amount of the resource contained in this [`Resource`].
    pub(crate) amount: u32,
    phantom: PhantomData<R>,
}

/// A token required for any out-of-thin-air creation of resources. This can only be created with
/// [`creation_token`]; this is meant for mods and engine internals, and must not be exposed to
/// players.
// APIs that need this token take a borrow in order to enable APIs like `StartingResources` where
// we give temporary access to the token.
#[non_exhaustive]
pub struct TokenOfCreation;

/// Create a token that allows out-of-thin-air creation of resources. Using it basically allows
/// cheating; for this reason this function must not be re-exported to players.
pub const fn creation_token() -> &'static TokenOfCreation {
    &TokenOfCreation
}

/// Creates a new [`Resource`] with the specified amount.
/// Should not be reexported in mods.
pub const fn resource<R>(_token: &TokenOfCreation, amount: u32) -> Resource<R>
where
    R: ResourceType,
{
    Resource::new(amount)
}

/// Returns a mutable reference to the amount of resource contained in the given [`Resource`].
/// Should not be reexported in mods.
pub const fn resource_amount_mut<'a, R>(
    token: &'a TokenOfCreation,
    resource: &'a mut Resource<R>,
) -> &'a mut u32
where
    R: ResourceType,
{
    resource.amount_mut(token)
}

impl<R> Resource<R>
where
    R: ResourceType,
{
    /// Creates a new empty [`Resource`].
    pub const fn new_empty() -> Self {
        Self {
            amount: 0,
            phantom: PhantomData,
        }
    }

    const fn new(amount: u32) -> Self {
        Self {
            amount,
            phantom: PhantomData,
        }
    }

    /// The current amount of the resource contained in this [`Resource`].
    pub const fn amount(&self) -> u32 {
        self.amount
    }

    pub(crate) const fn amount_mut(&mut self, _token: &TokenOfCreation) -> &mut u32 {
        &mut self.amount
    }

    /// Splits the [`Resource`] into two smaller parts.
    /// If there are insufficient resources in the [`Resource`], it returns an error with the original resource.
    pub const fn split(self, amount: u32) -> Result<(Self, Self), Self> {
        if let Some(remaining) = self.amount.checked_sub(amount) {
            Ok((Self::new(remaining), Self::new(amount)))
        } else {
            Err(self)
        }
    }

    /// Removes a specified amount of resources from this [`Resource`] and returns them as a new [`Resource`].
    /// If there are insufficient resources in the [`Resource`], it returns `None`.
    pub const fn split_off(&mut self, amount: u32) -> Result<Self, InsufficientResourceError<R>> {
        if let Some(remaining) = self.amount.checked_sub(amount) {
            self.amount = remaining;
            Ok(Resource::new(amount))
        } else {
            Err(InsufficientResourceError::new(amount, self.amount))
        }
    }

    /// Removes up to the specified amount of resources from this [`Resource`] and returns them as a new [`Resource`].
    /// If there are insufficient resources in the [`Resource`], it returns all available resources.
    pub const fn split_off_max(&mut self, amount: u32) -> Self {
        if let Some(remaining) = self.amount.checked_sub(amount) {
            self.amount = remaining;
            Resource::new(amount)
        } else {
            let all = self.amount;
            self.amount = 0;
            Resource::new(all)
        }
    }

    /// Empties this [`Resource`], returning all contained resources as a new [`Resource`].
    pub const fn empty(&mut self) -> Self {
        #[allow(clippy::mem_replace_with_default)] // doesn't work in `const`
        std::mem::replace(self, Self::new_empty())
    }

    /// Empties this [`Resource`] except for the specified amount, returning the emptied resources as a new [`Resource`].
    pub const fn empty_except(&mut self, amount: u32) -> Self {
        let to_empty = self.amount.saturating_sub(amount);
        self.amount -= to_empty;
        Resource::new(to_empty)
    }

    /// Empties this [`Resource`] into another [`Resource`], transferring all contained resources.
    pub const fn empty_into(&mut self, other: &mut Self) {
        other.amount += self.amount;
        self.amount = 0;
    }

    /// Adds the entire Rs of another resource container to this one.
    /// You can also use `+=`.
    pub fn add(&mut self, other: impl Into<Self>) {
        self.amount += other.into().amount();
    }

    /// Takes a specified amount of resources from this [`Resource`] and puts it into a [`Bundle`].
    pub const fn bundle<const AMOUNT: u32>(
        &mut self,
    ) -> Result<Bundle<R, AMOUNT>, InsufficientResourceError<R>> {
        if let Some(remaining) = self.amount.checked_sub(AMOUNT) {
            self.amount = remaining;
            Ok(Bundle::new())
        } else {
            Err(InsufficientResourceError::new(AMOUNT, self.amount))
        }
    }
}

impl<R> Default for Resource<R>
where
    R: ResourceType,
{
    fn default() -> Self {
        Self::new_empty()
    }
}

impl<R> Display for Resource<R>
where
    R: ResourceType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{amount} {R}", amount = self.amount, R = R::NAME)
    }
}

impl<R> PartialOrd<u32> for Resource<R>
where
    R: ResourceType,
{
    fn partial_cmp(&self, other: &u32) -> Option<std::cmp::Ordering> {
        Some(self.amount.cmp(other))
    }
}

impl<R> PartialEq<u32> for Resource<R>
where
    R: ResourceType,
{
    fn eq(&self, other: &u32) -> bool {
        self.amount == *other
    }
}

impl<R> PartialOrd<Resource<R>> for u32
where
    R: ResourceType,
{
    fn partial_cmp(&self, other: &Resource<R>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other.amount))
    }
}

impl<R> PartialEq<Resource<R>> for u32
where
    R: ResourceType,
{
    fn eq(&self, other: &Resource<R>) -> bool {
        *self == other.amount
    }
}

impl<R> AddAssign for Resource<R>
where
    R: ResourceType,
{
    fn add_assign(&mut self, rhs: Self) {
        self.amount += rhs.amount
    }
}

impl<R> Add for Resource<R>
where
    R: ResourceType,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<R> Sum for Resource<R>
where
    R: ResourceType,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Resource::new_empty(), |cur, next| cur + next)
    }
}

/// Contains a fixed (compile-time known) amount of a resource.
/// A [`Bundle`] can be used to build structures or as input for recipes.
///
/// See the [`resources`](crate::resources) module docs for info on the relationship between [`Bundle`] and [`Resource`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[must_use = "This bundle is being dropped without being used. If this is intentional, use the `let _ = bundle;` pattern to silence this warning."]
pub struct Bundle<R, const AMOUNT: u32>
where
    R: ResourceType,
{
    dummy: PhantomData<R>,
}

/// Creates a new [`Bundle`] with the specified resource type and amount.
/// Should not be reexported in mods.
pub fn bundle<R, const AMOUNT: u32>(_token: &TokenOfCreation) -> Bundle<R, AMOUNT>
where
    R: ResourceType,
{
    Bundle::new()
}

/// A compile-time assertion that a condition is true.
pub struct Assert<const OK: bool>;
/// A trait implemented only for `Assert<true>`.
pub trait IsTrue {}
impl IsTrue for Assert<true> {}

impl<R, const AMOUNT: u32> Bundle<R, AMOUNT>
where
    R: ResourceType,
{
    /// The fixed amount of resource contained in this [`Bundle`].
    pub const AMOUNT: u32 = AMOUNT;

    pub(crate) const fn new() -> Self {
        Self { dummy: PhantomData }
    }

    /// Returns the fixed amount of resource contained in this [`Bundle`].
    pub const fn amount(&self) -> u32 {
        AMOUNT
    }

    /// Splits this [`Bundle`] into two smaller [`Bundle`]s with the specified amounts.
    /// The sum of `AMOUNT1` and `AMOUNT2` must equal the amount of this [`Bundle`].
    pub const fn split<const AMOUNT1: u32, const AMOUNT2: u32>(
        self,
    ) -> (Bundle<R, AMOUNT1>, Bundle<R, AMOUNT2>)
    where
        Assert<{ AMOUNT1 + AMOUNT2 == AMOUNT }>: IsTrue,
    {
        (Bundle::new(), Bundle::new())
    }

    /// Converts this [`Bundle`] into a [`Resource`] with the same resource type and amount.
    pub const fn to_resource(self) -> Resource<R> {
        Resource::new(AMOUNT)
    }
}

impl<R, const AMOUNT: u32> AddAssign<Bundle<R, AMOUNT>> for Resource<R>
where
    R: ResourceType,
{
    fn add_assign(&mut self, bundle: Bundle<R, AMOUNT>) {
        let _ = bundle;
        self.amount += AMOUNT;
    }
}

impl<R, const AMOUNT: u32> Add<Bundle<R, AMOUNT>> for Resource<R>
where
    R: ResourceType,
{
    type Output = Self;

    fn add(mut self, rhs: Bundle<R, AMOUNT>) -> Self::Output {
        self += rhs;
        self
    }
}

impl<R, const AMOUNT: u32> Add<Resource<R>> for Bundle<R, AMOUNT>
where
    R: ResourceType,
{
    type Output = Resource<R>;

    fn add(self, mut rhs: Resource<R>) -> Self::Output {
        rhs += self;
        rhs
    }
}

impl<R, const AMOUNT_LHS: u32, const AMOUNT_RHS: u32> Add<Bundle<R, AMOUNT_RHS>>
    for Bundle<R, AMOUNT_LHS>
where
    R: ResourceType,
    [(); { AMOUNT_LHS + AMOUNT_RHS } as usize]:,
{
    type Output = Bundle<R, { AMOUNT_LHS + AMOUNT_RHS }>;

    fn add(self, rhs: Bundle<R, AMOUNT_RHS>) -> Self::Output {
        let _ = rhs;
        Bundle::new()
    }
}

impl<R, const AMOUNT: u32> PartialEq<Bundle<R, AMOUNT>> for Resource<R>
where
    R: ResourceType,
{
    fn eq(&self, _other: &Bundle<R, AMOUNT>) -> bool {
        self.amount == AMOUNT
    }
}

impl<R, const AMOUNT: u32> PartialEq<Resource<R>> for Bundle<R, AMOUNT>
where
    R: ResourceType,
{
    fn eq(&self, other: &Resource<R>) -> bool {
        AMOUNT == other.amount
    }
}

impl<R, const AMOUNT: u32> PartialOrd<Bundle<R, AMOUNT>> for Resource<R>
where
    R: ResourceType,
{
    fn partial_cmp(&self, _other: &Bundle<R, AMOUNT>) -> Option<std::cmp::Ordering> {
        Some(self.amount.cmp(&AMOUNT))
    }
}

impl<R, const AMOUNT: u32> PartialOrd<Resource<R>> for Bundle<R, AMOUNT>
where
    R: ResourceType,
{
    fn partial_cmp(&self, other: &Resource<R>) -> Option<std::cmp::Ordering> {
        Some(AMOUNT.cmp(&other.amount))
    }
}

impl<R, const AMOUNT: u32> PartialEq<u32> for Bundle<R, AMOUNT>
where
    R: ResourceType,
{
    fn eq(&self, other: &u32) -> bool {
        AMOUNT == *other
    }
}

impl<R, const AMOUNT: u32> PartialEq<Bundle<R, AMOUNT>> for u32
where
    R: ResourceType,
{
    fn eq(&self, _other: &Bundle<R, AMOUNT>) -> bool {
        *self == AMOUNT
    }
}

impl<R, const AMOUNT: u32> PartialOrd<u32> for Bundle<R, AMOUNT>
where
    R: ResourceType,
{
    fn partial_cmp(&self, other: &u32) -> Option<std::cmp::Ordering> {
        Some(AMOUNT.cmp(other))
    }
}

impl<R, const AMOUNT: u32> PartialOrd<Bundle<R, AMOUNT>> for u32
where
    R: ResourceType,
{
    fn partial_cmp(&self, _other: &Bundle<R, AMOUNT>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&AMOUNT))
    }
}

impl<R, const AMOUNT: u32> From<Bundle<R, AMOUNT>> for Resource<R>
where
    R: ResourceType,
{
    fn from(bundle: Bundle<R, AMOUNT>) -> Self {
        let _ = bundle;
        Resource::new(AMOUNT)
    }
}
