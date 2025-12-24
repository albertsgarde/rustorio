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
    pub fn change_recipe<R2: RecipeEx>(mut self, recipe: R2) -> Result<Machine<R2>, Self> {
        fn nonempty((_, _, current): (&'static str, u32, &mut u32)) -> bool {
            *current > 0
        }

        let _ = recipe;
        if self.iter_inputs().any(nonempty) || self.iter_outputs().any(nonempty) {
            Err(self)
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
