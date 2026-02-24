use std::convert::Infallible;

use crate::tick::Tick;

/// The player defined contents of a [`Subfactory`].
/// An instance of this trait should contain all the machines and resource containers needed in the subfactory.
pub trait SubfactoryContents<'own, 'parent>
where
    'parent: 'own,
{
    /// The error type used for failures in the tick code.
    type TickError;

    /// Ticks the subfactory contents.
    /// Implement how resources are moved between machines and resource containers in this function.
    /// The Tick functions very similarly to the main tick, but advancing it will fail if it is advanced beyond the main tick.
    /// The Tick need not be advanced all the way to the main tick in this function,
    /// as this will happen automatically when this function returns.
    fn tick(&mut self, tick: &Tick<'own, 'parent>) -> Result<(), Self::TickError>;
}

struct SubfactoryContentsInner<'own, 'parent, T, TickFn, TickError>
where
    TickFn: Fn(&mut T, &Tick<'own, 'parent>) -> Result<(), TickError>,
    'parent: 'own,
{
    tick_fn: TickFn,
    contents: T,
    _marker: std::marker::PhantomData<(&'own (), &'parent ())>,
}

impl<'own, 'parent, T, TickFn, TickError> SubfactoryContents<'own, 'parent>
    for SubfactoryContentsInner<'own, 'parent, T, TickFn, TickError>
where
    TickFn: Fn(&mut T, &Tick<'own, 'parent>) -> Result<(), TickError>,
    'parent: 'own,
{
    type TickError = TickError;

    fn tick(&mut self, tick: &Tick<'own, 'parent>) -> Result<(), Self::TickError> {
        (self.tick_fn)(&mut self.contents, tick)
    }
}

/// A subfactory is in a sense a player defined machine like a furnace or an assembler.
/// It can contain any number of machines and resource containers, and you defined how it ticks by implementing the [`SubfactoryContents`] trait for its contents.
/// This allows you to abstract away parts of your factory like smelting or circuit production, and treat them as a single machine in the main factory.
#[derive(Debug)]
pub struct Subfactory<'own, 'parent, T>
where
    T: SubfactoryContents<'own, 'parent>,
{
    tick: Tick<'own, 'parent>,
    contents: T,
}

impl<'own, 'parent> Tick<'own, 'parent> {
    /// Creates a new subfactory with the given contents and tick function.
    pub const fn subfactory<'branch, T, TickFn, TickError>(
        &self,
        contents: T,
        tick_fn: TickFn,
    ) -> Subfactory<'branch, 'own, SubfactoryContentsInner<'branch, 'own, T, TickFn, TickError>>
    where
        TickFn: Fn(&mut T, &Tick<'branch, 'own>) -> Result<(), TickError>,
    {
        Subfactory {
            tick: self.branch(),
            contents: SubfactoryContentsInner {
                tick_fn,
                contents,
                _marker: std::marker::PhantomData,
            },
        }
    }
}

impl<'own, 'parent, T> Subfactory<'own, 'parent, T>
where
    T: SubfactoryContents<'own, 'parent>,
{
    /// Attempts to advance the subfactory tick by ticking its contents using the [`tick`](SubfactoryContents::tick) function from the [`SubfactoryContents`] trait.
    /// Even if the tick function itself doesn't advance the Tick all the way to the main tick,
    /// the Tick will be advanced to the main tick at the end of this function.
    fn try_tick(&mut self, tick: &Tick<'parent, '_>) -> Result<(), T::TickError> {
        self.tick.update(tick);
        self.contents.tick(&self.tick)?;
        assert!(
            self.tick.cur() <= tick.cur(),
            "Subfactory tick is ahead of main tick"
        );
        self.tick.advance_to_main();
        assert!(
            self.tick.cur() == tick.cur(),
            "Subfactory tick is not in sync with main tick after advancing to main tick"
        );
        Ok(())
    }

    /// Equivalent of a machine's `inputs` or `outputs` function, but for the subfactory contents.
    /// Allows access to the contents of the subfactory while ensuring the contents are updated to the current tick before that.
    ///
    /// Attempts to advance the subfactory tick by ticking its contents using the [`tick`](SubfactoryContents::tick) function from the [`SubfactoryContents`] trait.
    ///
    /// If the tick function is infallible and its error is [`Infallible`](std::convert::Infallible), you can use the [`contents`](Subfactory::contents) function instead.
    pub fn try_contents<'a>(
        &'a mut self,
        tick: &'a Tick<'parent, '_>,
    ) -> Result<&'a mut T, T::TickError> {
        self.tick.update(tick);
        self.try_tick(tick).map(|()| &mut self.contents)
    }
}

impl<'own, 'parent, T> Subfactory<'own, 'parent, T>
where
    T: SubfactoryContents<'own, 'parent, TickError = Infallible>,
{
    /// Equivalent of a machine's `inputs` or `outputs` function, but for the subfactory contents.
    /// Allows access to the contents of the subfactory while ensuring the contents are updated to the current tick before that.
    ///
    /// Advances the subfactory tick by ticking its contents using the [`tick`](SubfactoryContents::tick) function from the [`SubfactoryContents`] trait.
    pub fn contents<'a>(&'a mut self, tick: &'a Tick<'parent, '_>) -> &'a mut T {
        match self.try_contents(tick) {
            Ok(contents) => contents,
        }
    }
}
