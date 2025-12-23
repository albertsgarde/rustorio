//! Basic machine that can process recipes. Mods are encouraged to not export this, and instead define
//! their own wrappers like
//! ```rust
//! use rustorio_engine::{machine::Machine, recipe::Recipe, Sealed};

//! trait AssemblerRecipe: Recipe + Sealed {}

//! pub struct Assembler<R: AssemblerRecipe>(Machine<R>);
//! ```

use crate::{
    recipe::{Recipe, RecipeEx},
    tick::Tick,
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
    inputs: R::Inputs,
    outputs: R::Outputs,
    tick: u64,
    crafting_time: u64,
}

impl<R: RecipeEx> Machine<R> {
    fn new_inner(tick: u64) -> Self {
        Self {
            inputs: R::new_inputs(),
            outputs: R::new_outputs(),
            tick,
            crafting_time: 0,
        }
    }

    /// Build a new machine.
    pub fn new(tick: &Tick) -> Self {
        Self::new_inner(tick.cur())
    }

    /// Update internal state and access input buffers.
    pub fn inputs(&mut self, tick: &Tick) -> &mut R::Inputs {
        self.tick(tick);
        &mut self.inputs
    }

    /// Update internal state and access output buffers.
    pub fn outputs(&mut self, tick: &Tick) -> &mut R::Outputs {
        self.tick(tick);
        &mut self.outputs
    }

    fn iter_inputs(&mut self) -> impl Iterator<Item = (&'static str, u32, &mut u32)> {
        R::iter_inputs(&mut self.inputs)
    }

    fn iter_outputs(&mut self) -> impl Iterator<Item = (&'static str, u32, &mut u32)> {
        R::iter_outputs(&mut self.outputs)
    }

    /// Changes the [`Recipe`](crate::recipe) of the machine.
    /// Returns the original machine if the machine has any inputs or outputs.
    pub fn change_recipe<R2: RecipeEx>(
        mut self,
        recipe: R2,
    ) -> Result<Machine<R2>, MachineNotEmptyError<Self>> {
        let _ = recipe;
        fn nonempty(
            (resource_name, _needed, current): (&'static str, u32, &mut u32),
        ) -> Option<(&'static str, u32)> {
            let &mut current = current;
            if current > 0 {
                Some((resource_name, current))
            } else {
                None
            }
        }
        let failure = if let Some((resource_type, amount)) = self.iter_inputs().find_map(nonempty) {
            Some((resource_type, amount, BufferLocation::Input))
        } else if let Some((resource_type, amount)) = self.iter_outputs().find_map(nonempty) {
            Some((resource_type, amount, BufferLocation::Output))
        } else {
            None
        };
        if let Some((resource_type, amount, location)) = failure {
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
        assert!(tick.cur() >= self.tick, "Tick must be non-decreasing");

        self.crafting_time += tick.cur() - self.tick;
        let crafting_time = self.crafting_time;
        let count = self
            .iter_inputs()
            .map(|(_, needed, current)| *current / needed)
            .chain((R::TIME > 0).then(|| (crafting_time / R::TIME).try_into().unwrap()))
            .min()
            .unwrap();

        for (_, needed, current) in self.iter_inputs() {
            *current -= count * needed;
        }
        for (_, needed, current) in self.iter_outputs() {
            *current += count * needed;
        }
        self.crafting_time -= u64::from(count) * R::TIME;

        if self
            .iter_inputs()
            .any(|(_, needed, current)| *current < needed)
        {
            self.crafting_time = 0;
        }

        self.tick = tick.cur();
    }
}
