//! Buildings take inputs to produce outputs over time.
//!
//! To use a building, you must first build it which takes a number of resources.
//! Then you can add inputs to it using `inputs`.
//! Once it has sufficient inputs, it will start producing outputs, which can be extracted using `outputs`.
//!
//! When created, a building is set to a specific [`Recipe`](crate::recipes), which defines the inputs and outputs.
//! This can be changed using the `change_recipe` method, but only if the building is empty (no inputs or outputs).

use rustorio_engine::{
    machine::{Machine, MachineNotEmptyError},
    recipe::Recipe,
};

use crate::{
    Bundle, Tick,
    recipes::{AssemblerRecipe, FurnaceRecipe},
    resources::{Copper, Iron},
};

/// The assembler is used for recipes that require two different inputs to produce an output.
///
/// To use, first build the assembler using [`Assembler::build`], providing the desired recipe and the required resources.
/// Then, add inputs using [`inputs`](Assembler::inputs), for example `assembler.inputs(&tick).0.add(bundle)`.
/// The assembler will automatically process the inputs over time, which can be advanced using the [`Tick`].
/// Outputs can be extracted using [`outputs`](Assembler::outputs), for example `assembler.outputs(&tick).0.bundle::<1>()`.
/// If you want to change the recipe, use [`change_recipe`](Assembler::change_recipe), but ensure the assembler is empty first.
#[derive(Debug)]
pub struct Assembler<R: AssemblerRecipe>(Machine<R>);

impl<R: AssemblerRecipe> Assembler<R> {
    /// Builds an assembler. Costs 15 iron and 10 copper.
    pub fn build(
        tick: &Tick,
        recipe: R,
        iron: Bundle<Iron, 15>,
        copper: Bundle<Copper, 10>,
    ) -> Self {
        let _ = (recipe, iron, copper);
        Self(Machine::new(tick))
    }

    /// Changes the [`Recipe`](crate::recipes) of the assembler.
    /// Returns the original assembler if the assembler has any inputs or outputs.
    pub fn change_recipe<R2: AssemblerRecipe>(
        self,
        recipe: R2,
    ) -> Result<Assembler<R2>, MachineNotEmptyError<Self>> {
        match self.0.change_recipe(recipe) {
            Ok(machine) => Ok(Assembler(machine)),
            Err(err) => Err(err.map_machine(|machine| Assembler(machine))),
        }
    }

    /// Update internal state and access input buffers.
    pub fn inputs(&mut self, tick: &Tick) -> &mut <R as Recipe>::Inputs {
        self.0.inputs(tick)
    }

    /// Update internal state and access output buffers.
    pub fn outputs(&mut self, tick: &Tick) -> &mut <R as Recipe>::Outputs {
        self.0.outputs(tick)
    }
}

/// The furnace is used to smelt ores into base resources.
///
/// To use, first build the furnace using [`Furnace::build`], providing the desired recipe and the required resources.
/// Then, add inputs using [`inputs`](Furnace::inputs), for example `furnamce.inputs(&tick).0.add(bundle)`.
/// The furnace will automatically process the inputs over time, which can be advanced using the [`Tick`].
/// Outputs can be extracted using [`outputs`](Furnace::outputs), for example `furnace.outputs(&tick).0.bundle::<1>()`.
/// If you want to change the recipe, use [`change_recipe`](Furnace::change_recipe), but ensure the furnace is empty first.
#[derive(Debug)]
pub struct Furnace<R: FurnaceRecipe>(Machine<R>);

impl<R: FurnaceRecipe> Furnace<R> {
    /// Builds a furnace. Costs 10 iron.
    pub fn build(tick: &Tick, recipe: R, iron: Bundle<Iron, 10>) -> Self {
        let _ = (recipe, iron);
        Self(Machine::new(tick))
    }

    /// Changes the [`Recipe`](crate::recipes) of the furnace.
    /// Returns the original furnace if the furnace has no inputs or outputs.
    pub fn change_recipe<R2: FurnaceRecipe>(
        self,
        recipe: R2,
    ) -> Result<Furnace<R2>, MachineNotEmptyError<Self>> {
        match self.0.change_recipe(recipe) {
            Ok(machine) => Ok(Furnace(machine)),
            Err(err) => Err(err.map_machine(|machine| Furnace(machine))),
        }
    }

    /// Update internal state and access input buffers.
    pub fn inputs(&mut self, tick: &Tick) -> &mut <R as Recipe>::Inputs {
        self.0.inputs(tick)
    }

    /// Update internal state and access output buffers.
    pub fn outputs(&mut self, tick: &Tick) -> &mut <R as Recipe>::Outputs {
        self.0.outputs(tick)
    }
}
