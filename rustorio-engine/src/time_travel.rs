//! This module contains utilities to manipulate past ticks. This is an advanced feature, that can
//! be used to build reuseable subfactories.
use std::marker::PhantomData;

use crate::{
    ResourceType,
    resources::{Bundle, InsufficientResourceError, Resource},
    tick::Tick,
};

/// A record of a point in time.
#[derive(Debug, Clone, Copy)]
pub struct TickSnapshot {
    tick: u64,
}

/// Error returned when trying to move a tick snapshot backwards in time.
#[derive(Debug)]
pub struct BackwardTickingError;

impl Tick {
    /// Record a snapshot of a past tick.
    pub const fn snapshot(&self) -> TickSnapshot {
        TickSnapshot { tick: self.cur() }
    }
}
impl TickSnapshot {
    /// Advance the snapshot to the target snapshot, returning the number of ticks elapsed.
    pub const fn advance_to(&mut self, until: TickSnapshot) -> Result<u64, BackwardTickingError> {
        if let Some(diff) = until.tick.checked_sub(self.tick) {
            self.tick = until.tick;
            Ok(diff)
        } else {
            Err(BackwardTickingError)
        }
    }

    /// Advances the snapshot to the given tick, triggering a callback on each intermediate past tick.
    pub fn on_each_tick(
        &mut self,
        until: TickSnapshot,
        mut on_each_tick: impl for<'tick> FnMut(&PastTick<'tick>),
    ) -> Result<u64, BackwardTickingError> {
        if let Some(diff) = until.tick.checked_sub(self.tick) {
            while self.tick < until.tick {
                self.tick += 1;
                let past_tick = PastTick {
                    tick: *self,
                    phantom: PhantomData,
                };
                on_each_tick(&past_tick)
            }
            Ok(diff)
        } else {
            Err(BackwardTickingError)
        }
    }
}

/// A tick from the past. If a machine hasn't yet been updated beyond this tick, its resources can
/// be accessed in the past so that we can move resources around before catching up to the present.
/// This is used with `TickSnapshot`.
///
/// The `'tick` lifetime is invariant and prevents mixing resources the come from different times.
#[derive(Debug)]
pub struct PastTick<'tick> {
    tick: TickSnapshot,
    /// Makes the lifetime invariant, which makes it usable as a branding lifetime à la
    /// `GhostCell`.
    phantom: PhantomData<fn(&'tick ()) -> &'tick ()>,
}

impl<'tick> PastTick<'tick> {
    /// The past time that this corresponds to.
    pub const fn as_snapshot(&self) -> TickSnapshot {
        self.tick
    }
}

/// An `X` from the past. `X` is typically `Bundle` or `Resource`.
///
/// The `'tick` lifetime is invariant and prevents mixing resources the come from different times.
#[derive(Debug)]
#[repr(transparent)]
pub struct Past<'tick, X> {
    x: X,
    phantom: PhantomData<fn(&'tick ()) -> &'tick ()>,
}

impl<'tick, X> Past<'tick, X> {
    fn as_inner_mut(&mut self) -> &mut X {
        // Safety: `Past` is `repr(transparent)`; the types are otherwise the same.
        unsafe { std::mem::transmute(self) }
    }
    fn from_inner(x: X) -> Self {
        Self {
            x,
            phantom: PhantomData,
        }
    }
    #[expect(unused)]
    fn from_inner_mut(x: &mut X) -> &mut Self {
        // Safety: `Past` is `repr(transparent)`; the types are otherwise the same.
        unsafe { std::mem::transmute(x) }
    }
}

impl<'tick, R> Past<'tick, Resource<R>>
where
    R: ResourceType,
{
    /// Adds the entire Rs of another resource container to this one.
    /// You can also use `+=`.
    pub fn add(&mut self, other: impl Into<Self>) {
        self.as_inner_mut().add(other.into().x);
    }

    /// Takes a specified amount of resources from this [`Resource`] and puts it into a [`Bundle`].
    pub fn bundle<const AMOUNT: u32>(
        &mut self,
    ) -> Result<Past<'tick, Bundle<R, AMOUNT>>, InsufficientResourceError<R>> {
        let bundle = self.as_inner_mut().bundle()?;
        Ok(Past::from_inner(bundle))
    }
}

impl<'tick, R, const AMOUNT: u32> From<Past<'tick, Bundle<R, AMOUNT>>> for Past<'tick, Resource<R>>
where
    R: ResourceType,
{
    fn from(bundle: Past<'tick, Bundle<R, AMOUNT>>) -> Self {
        Self::from_inner(bundle.x.into())
    }
}
