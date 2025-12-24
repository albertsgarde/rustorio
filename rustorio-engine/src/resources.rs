//! Resources are the fundamental units of value in Rustorio.
//! Resources are held in either [`Resource`] or [`Bundle`] objects.
//! [`Bundle`] objects are used to hold a fixed amount of a resource, while [`Resource`] objects can hold any amount.
//!
//! This module the core definitions for resources, including the `ResourceType` trait, the `Resource` and `Bundle` structs, and the macro to define new resources.

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Add, AddAssign},
};

use crate::Sealed;

/// A type that represents a specific kind of resource in the game.
/// Implementors of this trait represent different resource types, such as iron, copper, or science packs.
/// Only useful as a type parameter; has no associated methods.
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
///     Gold);
/// ```
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

/// Error returned when there are insufficient resources in a [`Resource`] to fulfill a request.
#[derive(Debug, Clone)]
pub struct InsufficientResourceError<Resource: ResourceType> {
    /// The amount of resource that was requested.
    pub requested_amount: u32,
    /// The amount of resource that was actually available.
    pub available_amount: u32,
    phantom: PhantomData<Resource>,
}

impl<Resource: ResourceType> InsufficientResourceError<Resource> {
    /// Creates a new `InsufficientResourceError`.
    pub fn new(requested_amount: u32, available_amount: u32) -> Self {
        Self {
            requested_amount,
            available_amount,
            phantom: PhantomData,
        }
    }
}

impl<Resource: ResourceType> Display for InsufficientResourceError<Resource> {
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
pub struct Resource<Content: ResourceType> {
    /// The amount of the resource contained in this [`Resource`].
    pub(crate) amount: u32,
    phantom: PhantomData<Content>,
}

/// Creates a new [`Resource`] with the specified amount.
/// Should not be reexported in mods.
pub fn resource<Content: ResourceType>(amount: u32) -> Resource<Content> {
    Resource::new(amount)
}

impl<Content: ResourceType> Resource<Content> {
    /// Creates a new empty [`Resource`].
    pub fn new_empty() -> Self {
        Self {
            amount: 0,
            phantom: PhantomData,
        }
    }

    pub(crate) fn new(amount: u32) -> Self {
        Self {
            amount,
            phantom: PhantomData,
        }
    }

    /// The current amount of the resource contained in this [`Resource`].
    pub fn amount(&self) -> u32 {
        self.amount
    }

    /// Splits the [`Resource`] into two smaller parts.
    /// If there are insufficient resources in the [`Resource`], it returns an error with the original resource.
    pub fn split(self, amount: u32) -> Result<(Self, Self), Self> {
        if let Some(remaining) = self.amount.checked_sub(amount) {
            Ok((Self::new(remaining), Self::new(amount)))
        } else {
            Err(self)
        }
    }

    /// Removes a specified amount of resources from this [`Resource`] and returns them as a new [`Resource`].
    /// If there are insufficient resources in the [`Resource`], it returns `None`.
    pub fn split_off(&mut self, amount: u32) -> Result<Self, InsufficientResourceError<Content>> {
        if let Some(remaining) = self.amount.checked_sub(amount) {
            self.amount = remaining;
            Ok(Resource::new(amount))
        } else {
            Err(InsufficientResourceError::new(amount, self.amount))
        }
    }

    /// Empties this [`Resource`], returning all contained resources as a new [`Resource`].
    pub fn empty(&mut self) -> Self {
        let amount = self.amount;
        self.amount = 0;
        Resource::new(amount)
    }

    /// Empties this [`Resource`] into another [`Resource`], transferring all contained resources.
    pub fn empty_into(&mut self, other: &mut Self) {
        other.amount += self.amount;
        self.amount = 0;
    }

    /// Adds the entire contents of another resource container to this one.
    pub fn add(&mut self, other: impl Into<Self>) {
        self.amount += other.into().amount();
    }

    /// Consumes a [`Bundle`] of the same resource type and adds the contained resources to this [`Resource`].
    pub fn add_bundle<const AMOUNT: u32>(&mut self, bundle: Bundle<Content, AMOUNT>) {
        self.amount += bundle.amount();
    }

    /// Takes a specified amount of resources from this [`Resource`] and puts it into a [`Bundle`].
    pub fn bundle<const AMOUNT: u32>(
        &mut self,
    ) -> Result<Bundle<Content, AMOUNT>, InsufficientResourceError<Content>> {
        if let Some(remaining) = self.amount.checked_sub(AMOUNT) {
            self.amount = remaining;
            Ok(Bundle::new())
        } else {
            Err(InsufficientResourceError::new(AMOUNT, self.amount))
        }
    }
}

impl<Content: ResourceType> Display for Resource<Content> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} x {}", Content::NAME, self.amount)
    }
}

impl<Content: ResourceType> PartialOrd<u32> for Resource<Content> {
    fn partial_cmp(&self, other: &u32) -> Option<std::cmp::Ordering> {
        Some(self.amount.cmp(other))
    }
}

impl<Content: ResourceType> PartialEq<u32> for Resource<Content> {
    fn eq(&self, other: &u32) -> bool {
        self.amount == *other
    }
}

impl<Content: ResourceType> PartialOrd<Resource<Content>> for u32 {
    fn partial_cmp(&self, other: &Resource<Content>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other.amount))
    }
}

impl<Content: ResourceType> PartialEq<Resource<Content>> for u32 {
    fn eq(&self, other: &Resource<Content>) -> bool {
        *self == other.amount
    }
}

impl<Content: ResourceType> AddAssign for Resource<Content> {
    fn add_assign(&mut self, rhs: Self) {
        self.amount += rhs.amount
    }
}

impl<Content: ResourceType> Add for Resource<Content> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

/// Contains a fixed (compile-time known) amount of a resource.
/// A [`Bundle`] can be used to build structures or as input for recipes.
pub struct Bundle<Content: ResourceType, const AMOUNT: u32> {
    dummy: PhantomData<Content>,
}

/// Creates a new [`Bundle`] with the specified resource type and amount.
/// Should not be reexported in mods.
pub fn bundle<Content: ResourceType, const AMOUNT: u32>() -> Bundle<Content, AMOUNT> {
    Bundle::new()
}

pub struct Assert<const OK: bool>;
pub trait IsTrue {}
impl IsTrue for Assert<true> {}

impl<Content: ResourceType, const AMOUNT: u32> Bundle<Content, AMOUNT> {
    /// The fixed amount of resource contained in this [`Bundle`].
    pub const AMOUNT: u32 = AMOUNT;

    pub(crate) fn new() -> Self {
        Self { dummy: PhantomData }
    }

    /// Returns the fixed amount of resource contained in this [`Bundle`].
    pub fn amount(&self) -> u32 {
        AMOUNT
    }

    /// Splits this [`Bundle`] into two smaller [`Bundle`]s with the specified amounts.
    /// The sum of `AMOUNT1` and `AMOUNT2` must equal the amount of this [`Bundle`].
    pub fn split<const AMOUNT1: u32, const AMOUNT2: u32>(
        self,
    ) -> (Bundle<Content, AMOUNT1>, Bundle<Content, AMOUNT2>)
    where
        Assert<{ AMOUNT1 + AMOUNT2 == AMOUNT }>: IsTrue,
    {
        (Bundle::new(), Bundle::new())
    }

    /// Converts this [`Bundle`] into a [`Resource`] with the same resource type and amount.
    pub fn to_resource(self) -> Resource<Content> {
        Resource::new(AMOUNT)
    }
}

impl<Content: ResourceType, const AMOUNT: u32> AddAssign<Bundle<Content, AMOUNT>>
    for Resource<Content>
{
    fn add_assign(&mut self, bundle: Bundle<Content, AMOUNT>) {
        let _ = bundle;
        self.amount += AMOUNT;
    }
}

impl<Content: ResourceType, const AMOUNT: u32> Add<Bundle<Content, AMOUNT>> for Resource<Content> {
    type Output = Self;

    fn add(mut self, rhs: Bundle<Content, AMOUNT>) -> Self::Output {
        self += rhs;
        self
    }
}

impl<Content: ResourceType, const AMOUNT: u32> Add<Resource<Content>> for Bundle<Content, AMOUNT> {
    type Output = Resource<Content>;

    fn add(self, mut rhs: Resource<Content>) -> Self::Output {
        rhs += self;
        rhs
    }
}

impl<Content: ResourceType, const AMOUNT_LHS: u32, const AMOUNT_RHS: u32>
    Add<Bundle<Content, AMOUNT_RHS>> for Bundle<Content, AMOUNT_LHS>
where
    [(); { AMOUNT_LHS + AMOUNT_RHS } as usize]:,
{
    type Output = Bundle<Content, { AMOUNT_LHS + AMOUNT_RHS }>;

    fn add(self, rhs: Bundle<Content, AMOUNT_RHS>) -> Self::Output {
        let _ = rhs;
        Bundle::new()
    }
}

impl<Content: ResourceType, const AMOUNT: u32> From<Bundle<Content, AMOUNT>> for Resource<Content> {
    fn from(bundle: Bundle<Content, AMOUNT>) -> Self {
        let _ = bundle;
        Resource::new(AMOUNT)
    }
}
