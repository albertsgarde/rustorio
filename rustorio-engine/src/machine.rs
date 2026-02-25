//! Basic machine that can process recipes. Mods are encouraged to not export this, and instead define
//! their own wrappers like
//! ```rust
//! use rustorio_engine::{machine::Machine, recipe::Recipe, Sealed};

//! trait AssemblerRecipe: Recipe + Sealed {}

//! pub struct Assembler<R: AssemblerRecipe>(Machine<R>);
//! ```

use crate::{
    recipe::{MultiBundle, Recipe},
    resources::{TokenOfCreation, creation_token},
    tick::{Tick, TickSnapshot},
};

/// Location of a resource buffer in a machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferLocation {
    /// Input buffer.
    Input,
    /// Output buffer.
    Output,
}

/// Error returned when trying to change a machine's recipe while it has non-empty input or output buffers.
#[derive(Debug)]
pub struct MachineNotEmptyError<M> {
    /// Returning the machine with the original recipe.
    pub machine: M,
    /// Name of the type of the resource in the machine's buffers.
    pub resource_type: &'static str,
    /// The amount of the resource in the machine's buffers.
    pub amount: u32,
    /// Whether the resource is in the input or the output.
    pub location: BufferLocation,
}

impl<M> MachineNotEmptyError<M> {
    /// Converts the error to another machine type, keeping the same resource information.
    pub fn map_machine<F, M2>(self, f: F) -> MachineNotEmptyError<M2>
    where
        F: FnOnce(M) -> M2,
    {
        MachineNotEmptyError {
            machine: f(self.machine),
            resource_type: self.resource_type,
            amount: self.amount,
            location: self.location,
        }
    }
}

impl<R: Recipe> std::fmt::Display for MachineNotEmptyError<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Machine is not empty: machine has {} of resource {} in its {:?} buffer",
            self.amount, self.resource_type, self.location
        )
    }
}

/// Basic machine that can process recipes.
#[derive(Debug)]
pub struct Machine<R: Recipe> {
    inputs: R::InputResources,
    outputs: R::OutputResources,
    tick: TickSnapshot,
    crafting_time: u64,
}

impl<R: Recipe> Machine<R> {
    fn new_inner(tick: TickSnapshot) -> Self {
        Self {
            inputs: Default::default(),
            outputs: Default::default(),
            tick,
            crafting_time: 0,
        }
    }

    /// Build a new machine.
    // Needs a token because this can be used to create resources by making a custom recipe.
    pub fn new(_token: &TokenOfCreation, tick: &Tick) -> Self {
        Self::new_inner(tick.snapshot())
    }

    /// Update internal state and access input buffers.
    pub fn inputs<'a>(&'a mut self, tick: &'a Tick) -> &'a mut R::InputResources {
        self.tick(tick);
        &mut self.inputs
    }

    /// Update internal state and access output buffers.
    pub fn outputs<'a>(&'a mut self, tick: &'a Tick) -> &'a mut R::OutputResources {
        self.tick(tick);
        &mut self.outputs
    }

    fn iter_inputs<'a>(
        &'a mut self,
        token: &'a TokenOfCreation,
    ) -> impl Iterator<Item = (&'static str, u32, &'a mut u32)> {
        <R::InputBundle as MultiBundle>::iter_mut(token, &mut self.inputs)
    }

    fn iter_outputs<'a>(
        &'a mut self,
        token: &'a TokenOfCreation,
    ) -> impl Iterator<Item = (&'static str, u32, &'a mut u32)> {
        <R::OutputBundle as MultiBundle>::iter_mut(token, &mut self.outputs)
    }

    /// Changes the [`Recipe`](crate::recipe) of the machine.
    /// Returns the original machine if the machine has any inputs or outputs.
    pub fn change_recipe<R2: Recipe>(
        mut self,
        recipe: R2,
    ) -> Result<Machine<R2>, MachineNotEmptyError<Self>> {
        let token = creation_token();
        let _ = recipe;
        fn find_nonempty<'a>(
            mut iter: impl Iterator<Item = (&'static str, u32, &'a mut u32)>,
            location: BufferLocation,
        ) -> Option<(&'static str, u32, BufferLocation)> {
            iter.find_map(|(resource_name, _needed, &mut current)| {
                (current > 0).then_some((resource_name, current, location))
            })
        }

        if let Some((resource_type, amount, location)) =
            find_nonempty(self.iter_inputs(token), BufferLocation::Input)
                .or_else(|| find_nonempty(self.iter_outputs(token), BufferLocation::Output))
        {
            Err(MachineNotEmptyError {
                machine: self,
                resource_type,
                amount,
                location,
            })
        } else {
            Ok(Machine::new_inner(self.tick))
        }
    }

    fn tick(&mut self, tick: &Tick) {
        let time_elapsed = self.tick.advance_to(tick).unwrap();
        let token = creation_token();

        self.crafting_time += time_elapsed;
        let crafting_time = self.crafting_time;
        let count = self
            .iter_inputs(token)
            .map(|(_, needed, current)| *current / needed)
            .chain((R::TIME > 0).then(|| (crafting_time / R::TIME).try_into().unwrap()))
            .min()
            .unwrap();

        for (_, needed, current) in self.iter_inputs(token) {
            *current -= count * needed;
        }
        for (_, needed, current) in self.iter_outputs(token) {
            *current += count * needed;
        }
        self.crafting_time -= u64::from(count) * R::TIME;

        if self
            .iter_inputs(token)
            .any(|(_, needed, current)| *current < needed)
        {
            self.crafting_time = 0;
        }
    }
}
