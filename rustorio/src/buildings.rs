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
    recipe::{Recipe, RecipeEx},
    research::{TechRecipe, Technology, tech_recipe},
};

use crate::{
    Bundle, Tick,
    recipes::{AssemblerRecipe, FurnaceRecipe},
    resources::{Copper, CopperWire, Iron},
};

/// The assembler can craft most items in the game.
///
/// To use, first build the assembler using [`Assembler::build`], providing the desired recipe and the required resources.
/// Then, add inputs using [`inputs`](Assembler::inputs), for example `assembler.inputs(&tick).0 += bundle`.
/// The assembler will automatically process the inputs over time, which can be advanced using the [`Tick`].
/// Outputs can be extracted using [`outputs`](Assembler::outputs), for example `assembler.outputs(&tick).0.bundle::<1>()`.
/// If you want to change the recipe, use [`change_recipe`](Assembler::change_recipe), but ensure the assembler is empty first.
///
/// See the [implementors](AssemblerRecipe#implementors) of the [`AssemblerRecipe`] trait for recipes that can be used in the assembler.
#[derive(Debug)]
pub struct Assembler<'t, R: AssemblerRecipe<'t>>(Machine<'t, R>);

impl<'t, R: AssemblerRecipe<'t>> Assembler<'t, R> {
    /// Builds an assembler. Costs 12 [copper wires](crate::resources::CopperWire) and 6 [iron](crate::resources::Iron).
    pub fn build(
        tick: &Tick<'t, '_>,
        recipe: R,
        copper_wires: Bundle<CopperWire, 12>,
        iron: Bundle<Iron, 6>,
    ) -> Self {
        let _ = (recipe, copper_wires, iron);
        Self(Machine::new(tick))
    }

    /// Changes the [`Recipe`](crate::recipes) of the assembler.
    /// Returns the original assembler if the the input and output buffers are not empty.
    pub fn change_recipe<R2: AssemblerRecipe<'t>>(
        self,
        recipe: R2,
    ) -> Result<Assembler<'t, R2>, MachineNotEmptyError<Self>> {
        match self.0.change_recipe(recipe) {
            Ok(machine) => Ok(Assembler(machine)),
            Err(err) => Err(err.map_machine(Assembler)),
        }
    }

    /// Update internal state and access input buffers.
    pub fn inputs<'a>(&'a mut self, tick: &'a Tick<'t, '_>) -> &'a mut <R as Recipe<'t>>::Inputs {
        self.0.inputs(tick)
    }

    /// Amount of each input resource needed for one recipe cycle
    pub const fn input_amounts(&self) -> <R as Recipe<'t>>::InputAmountsType {
        <R as Recipe<'t>>::INPUT_AMOUNTS
    }

    /// Update internal state and access output buffers.
    pub fn outputs<'a>(&'a mut self, tick: &'a Tick<'t, '_>) -> &'a mut <R as Recipe<'t>>::Outputs {
        self.0.outputs(tick)
    }

    /// Amount of each output resource created per recipe cycle
    pub const fn output_amounts(&self) -> <R as Recipe<'t>>::OutputAmountsType {
        <R as Recipe<'t>>::OUTPUT_AMOUNTS
    }
}

/// The furnace is used to smelt ores into base resources.
///
/// To use, first build the furnace using [`Furnace::build`], providing the desired recipe and the required resources.
/// Then, add inputs using [`inputs`](Furnace::inputs), for example `furnace.inputs(&tick).0 += bundle`.
/// The furnace will automatically process the inputs over time, which can be advanced using the [`Tick`].
/// Outputs can be extracted using [`outputs`](Furnace::outputs), for example `furnace.outputs(&tick).0.bundle::<1>()`.
/// If you want to change the recipe, use [`change_recipe`](Furnace::change_recipe), but ensure the furnace is empty first.
///
/// See the [implementors](FurnaceRecipe#implementors) of the [`FurnaceRecipe`] trait for recipes that can be used in the furnace.
#[derive(Debug)]
pub struct Furnace<'t, R: FurnaceRecipe<'t>>(Machine<'t, R>);

impl<'t, R: FurnaceRecipe<'t>> Furnace<'t, R> {
    /// Builds a furnace. Costs 10 [iron](crate::resources::Iron).
    pub fn build(tick: &Tick<'t, '_>, recipe: R, iron: Bundle<Iron, 10>) -> Self {
        let _ = (recipe, iron);
        Self(Machine::new(tick))
    }

    /// Changes the [`Recipe`](crate::recipes) of the furnace.
    /// Returns the original furnace if the the input and output buffers are not empty.
    pub fn change_recipe<R2: FurnaceRecipe<'t>>(
        self,
        recipe: R2,
    ) -> Result<Furnace<'t, R2>, MachineNotEmptyError<Self>> {
        match self.0.change_recipe(recipe) {
            Ok(machine) => Ok(Furnace(machine)),
            Err(err) => Err(err.map_machine(Furnace)),
        }
    }

    /// Update internal state and access input buffers.
    pub fn inputs<'a, 'b, 'c>(
        &'a mut self,
        tick: &'b Tick<'t, '_>,
    ) -> &'c mut <R as Recipe<'t>>::Inputs
    where
        'b: 'c,
        'a: 'c,
    {
        self.0.inputs(tick)
    }

    /// Amount of each input resource needed for one recipe cycle
    pub const fn input_amounts(&self) -> <R as Recipe<'t>>::InputAmountsType {
        <R as Recipe<'t>>::INPUT_AMOUNTS
    }

    /// Update internal state and access output buffers.
    pub fn outputs<'a, 'b, 'c>(
        &'a mut self,
        tick: &'b Tick<'t, '_>,
    ) -> &'c mut <R as Recipe<'t>>::Outputs
    where
        'b: 'c,
        'a: 'c,
    {
        self.0.outputs(tick)
    }

    /// Amount of each output resource created per recipe cycle
    pub const fn output_amounts(&self) -> <R as Recipe<'t>>::OutputAmountsType {
        <R as Recipe<'t>>::OUTPUT_AMOUNTS
    }
}

/// Performs research to unlock new technologies.
/// Set it to produce research points for a specific technology either when [`build`](Lab::build)ing it,
/// or using [`change_technology`](Lab::change_technology).
#[derive(Debug)]
pub struct Lab<'t, T: Technology<'t>>(Machine<'t, TechRecipe<'t, T>>)
where
    TechRecipe<'t, T>: RecipeEx<'t>;

impl<'t, T: Technology<'t>> Lab<'t, T>
where
    TechRecipe<'t, T>: RecipeEx<'t>,
{
    /// Creates a new `Lab` producing research points for the specified technology.
    pub fn build(
        tick: &Tick<'t, '_>,
        technology: &T,
        iron: Bundle<Iron, 20>,
        copper: Bundle<Copper, 15>,
    ) -> Self {
        let _ = (technology, iron, copper);
        Self(Machine::new(tick))
    }

    /// Changes the technology this `Lab` is producing research points for.
    pub fn change_technology<T2: Technology<'t>>(
        self,
        technology: &T2,
    ) -> Result<Lab<'t, T2>, MachineNotEmptyError<Self>>
    where
        TechRecipe<'t, T2>: RecipeEx<'t>,
    {
        let _ = technology;
        match self.0.change_recipe(tech_recipe()) {
            Ok(machine) => Ok(Lab(machine)),
            Err(err) => Err(err.map_machine(Lab)),
        }
    }

    /// Get a mutable reference to input buffers.
    pub fn inputs<'a>(
        &'a mut self,
        tick: &'a Tick<'t, '_>,
    ) -> &'a mut <TechRecipe<'t, T> as Recipe<'t>>::Inputs {
        self.0.inputs(tick)
    }

    /// Amount of each input resource needed for one recipe cycle
    pub const fn input_amounts(&self) -> <TechRecipe<'t, T> as Recipe<'t>>::InputAmountsType {
        <TechRecipe<'t, T> as Recipe<'t>>::INPUT_AMOUNTS
    }

    /// Get a mutable reference to output buffers.
    pub fn outputs<'a>(
        &'a mut self,
        tick: &'a Tick<'t, '_>,
    ) -> &'a mut <TechRecipe<'t, T> as Recipe<'t>>::Outputs {
        self.0.outputs(tick)
    }
}
