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
    /// Multiplier on output units produced.
    productivity: u32,
    /// Multiplier on crafting speed.
    speed: u64,
}

impl<R: RecipeEx> Machine<R> {
    fn new_inner(tick: u64) -> Self {
        Self {
            inputs: R::new_inputs(),
            outputs: R::new_outputs(),
            tick,
            crafting_time: 0,
            productivity: 1,
            speed: 1,
        }
    }

    /// Build a new machine.
    pub fn new(tick: &Tick) -> Self {
        Self::new_inner(tick.cur())
    }

    /// Update internal state and access input buffers.
    pub fn inputs<'a>(&'a mut self, tick: &'a Tick) -> &'a mut R::Inputs {
        self.tick(tick);
        &mut self.inputs
    }

    /// Update internal state and access output buffers.
    pub fn outputs<'a>(&'a mut self, tick: &'a Tick) -> &'a mut R::Outputs {
        self.tick(tick);
        &mut self.outputs
    }

    /// The multiplier on output units produced.
    pub const fn productivity(&self) -> u32 {
        self.productivity
    }
    /// Set the multiplier on output units produced.
    pub fn productivity_mut(&mut self, tick: &Tick) -> &mut u32 {
        self.tick(tick);
        &mut self.productivity
    }

    /// The multiplier on crafting speed.
    pub const fn speed(&self) -> u64 {
        self.speed
    }
    /// Set the multiplier on crafting speed.
    pub fn speed_mut(&mut self, tick: &Tick) -> &mut u64 {
        self.tick(tick);
        &mut self.speed
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
        fn find_nonempty<'a>(
            mut iter: impl Iterator<Item = (&'static str, u32, &'a mut u32)>,
            location: BufferLocation,
        ) -> Option<(&'static str, u32, BufferLocation)> {
            iter.find_map(|(resource_name, _needed, &mut current)| {
                (current > 0).then_some((resource_name, current, location))
            })
        }

        if let Some((resource_type, amount, location)) =
            find_nonempty(self.iter_inputs(), BufferLocation::Input)
                .or_else(|| find_nonempty(self.iter_outputs(), BufferLocation::Output))
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
        assert!(tick.cur() >= self.tick, "Tick must be non-decreasing");
        let productivity = self.productivity;
        let time_per_unit = R::TIME.div_ceil(self.speed);

        self.crafting_time += tick.cur() - self.tick;
        let crafting_time = self.crafting_time;
        let count = self
            .iter_inputs()
            .map(|(_, needed, current)| *current / needed)
            .chain((time_per_unit > 0).then(|| (crafting_time / time_per_unit).try_into().unwrap()))
            .min()
            .unwrap();

        for (_, needed, current) in self.iter_inputs() {
            *current -= count * needed;
        }
        for (_, needed, current) in self.iter_outputs() {
            *current += count * needed * productivity;
        }
        self.crafting_time -= u64::from(count) * time_per_unit;

        if self
            .iter_inputs()
            .any(|(_, needed, current)| *current < needed)
        {
            self.crafting_time = 0;
        }

        self.tick = tick.cur();
    }
}
