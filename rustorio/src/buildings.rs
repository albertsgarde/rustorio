//! Buildings take inputs to produce outputs over time.
//!
//! To use a building, you must first build it which takes a number of resources.
//! Then you can add inputs to it using `inputs`.
//! Once it has sufficient inputs, it will start producing outputs, which can be extracted using `outputs`.
//! Input and output buffers are tuples of [`Resource`](crate::Resource) values in the same order as the recipe docs.
//! For example, a recipe with iron and copper-wire inputs exposes those buffers as `.0` and `.1`.
//!
//! When created, a building is set to a specific [`Recipe`](crate::recipes), which defines the inputs and outputs.
//! This can be changed using the `change_recipe` method, but only if the building is empty (no inputs or outputs).

use rustorio_engine::{
    machine::{Machine, MachineNotEmptyError},
    recipe::{MultiBundle, Recipe},
    research::{TechRecipe, Technology, tech_recipe},
    resources::creation_token,
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
pub struct Assembler<R: AssemblerRecipe>(Machine<R>);

impl<R: AssemblerRecipe> Assembler<R> {
    /// Builds an assembler. Costs 12 [copper wires](crate::resources::CopperWire) and 6 [iron](crate::resources::Iron).
    pub fn build(
        tick: &Tick,
        recipe: R,
        copper_wires: Bundle<CopperWire, 12>,
        iron: Bundle<Iron, 6>,
    ) -> Self {
        let token = creation_token();
        let _ = (recipe, copper_wires, iron);
        Self(Machine::new(token, tick))
    }

    /// Changes the [`Recipe`](crate::recipes) of the assembler.
    /// Returns the original assembler if the the input and output buffers are not empty.
    pub fn change_recipe<R2: AssemblerRecipe>(
        self,
        recipe: R2,
    ) -> Result<Assembler<R2>, MachineNotEmptyError<Self>> {
        match self.0.change_recipe(recipe) {
            Ok(machine) => Ok(Assembler(machine)),
            Err(err) => Err(err.map_machine(Assembler)),
        }
    }

    /// Update internal state and access input buffers.
    pub fn inputs<'a>(&'a mut self, tick: &'a Tick) -> &'a mut <R as Recipe>::InputResources {
        self.0.inputs(tick)
    }

    /// Amount of each input resource needed for one recipe cycle
    pub const fn input_amounts(&self) -> <R::InputBundle as MultiBundle>::AmountsType {
        <R::InputBundle as MultiBundle>::AMOUNTS
    }

    /// Update internal state and access output buffers.
    pub fn outputs<'a>(&'a mut self, tick: &'a Tick) -> &'a mut <R as Recipe>::OutputResources {
        self.0.outputs(tick)
    }

    /// Amount of each output resource created per recipe cycle
    pub const fn output_amounts(&self) -> <R::OutputBundle as MultiBundle>::AmountsType {
        <R::OutputBundle as MultiBundle>::AMOUNTS
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
/// # Example
///
/// ```rust
/// # use rustorio::{
/// #     Bundle, Tick,
/// #     buildings::Furnace,
/// #     recipes::CopperSmelting,
/// #     resources::{Copper, CopperOre, Iron},
/// # };
/// # fn example(mut tick: Tick) -> Bundle<Copper, 4> {
/// # let token = rustorio_engine::resources::creation_token();
/// # let iron = rustorio_engine::bundle::<Iron, 10>(token);
/// # let copper_ore = rustorio_engine::bundle::<CopperOre, 8>(token);
/// let mut furnace = Furnace::build(&tick, CopperSmelting, iron);
///
/// furnace.inputs(&tick).0 += copper_ore;
/// tick.advance_until(|tick| furnace.outputs(tick).0 >= 4);
///
/// let copper = furnace.outputs(&tick).0.bundle::<4>().unwrap();
/// # copper
/// # }
/// ```
///
/// See the [implementors](FurnaceRecipe#implementors) of the [`FurnaceRecipe`] trait for recipes that can be used in the furnace.
#[derive(Debug)]
pub struct Furnace<R: FurnaceRecipe>(Machine<R>);

impl<R: FurnaceRecipe> Furnace<R> {
    /// Builds a furnace. Costs 10 [iron](crate::resources::Iron).
    pub fn build(tick: &Tick, recipe: R, iron: Bundle<Iron, 10>) -> Self {
        let token = creation_token();
        let _ = (recipe, iron);
        Self(Machine::new(token, tick))
    }

    /// Changes the [`Recipe`](crate::recipes) of the furnace.
    /// Returns the original furnace if the the input and output buffers are not empty.
    pub fn change_recipe<R2: FurnaceRecipe>(
        self,
        recipe: R2,
    ) -> Result<Furnace<R2>, MachineNotEmptyError<Self>> {
        match self.0.change_recipe(recipe) {
            Ok(machine) => Ok(Furnace(machine)),
            Err(err) => Err(err.map_machine(Furnace)),
        }
    }

    /// Update internal state and access input buffers.
    pub fn inputs<'a>(&'a mut self, tick: &'a Tick) -> &'a mut <R as Recipe>::InputResources {
        self.0.inputs(tick)
    }

    /// Amount of each input resource needed for one recipe cycle
    pub const fn input_amounts(&self) -> <R::InputBundle as MultiBundle>::AmountsType {
        <R::InputBundle as MultiBundle>::AMOUNTS
    }

    /// Update internal state and access output buffers.
    pub fn outputs<'a>(&'a mut self, tick: &'a Tick) -> &'a mut <R as Recipe>::OutputResources {
        self.0.outputs(tick)
    }

    /// Amount of each output resource created per recipe cycle
    pub const fn output_amounts(&self) -> <R::OutputBundle as MultiBundle>::AmountsType {
        <R::OutputBundle as MultiBundle>::AMOUNTS
    }
}

/// Performs research to unlock new technologies.
/// Set it to produce research points for a specific technology either when [`build`](Lab::build)ing it,
/// or using [`change_technology`](Lab::change_technology).
///
/// ```rust
/// # use rustorio::{
/// #     Bundle, Tick, Technology,
/// #     buildings::Lab,
/// #     research::{PointsTechnology, RedScience, SteelTechnology},
/// #     resources::{Copper, Iron},
/// # };
/// # use rustorio_engine::{tick, resources, research::TechnologyEx};
/// #
/// # let token = resources::creation_token();
/// # let iron: Bundle<Iron, 20> = rustorio_engine::bundle(token);
/// # let copper: Bundle<Copper, 15> = rustorio_engine::bundle(token);
/// # let red_science: Bundle<RedScience, 20> = rustorio_engine::bundle(token);
/// # let mut tick = tick::tick(&token, 1_000_000);
/// # let steel_technology = SteelTechnology::instance(&token);
///
/// let mut lab = Lab::build(&tick, &steel_technology, iron, copper);
///
/// lab.inputs(&tick).0 += red_science; // Add 20 red science to the lab's input buffer.
/// tick.advance_until(|tick| {
///     lab.outputs(tick).0 >= SteelTechnology::REQUIRED_RESEARCH_POINTS
/// });
///
///
/// let research_points = lab.outputs(&tick).0.bundle().expect("Given the above advance_until, the lab should contain enough research points");
/// let (_steel_smelting, points_research) = steel_technology.research(research_points);
///
/// let lab = lab.change_technology(&points_research).expect("We added exactly the required amount of science packs to the lab, so it should be empty");
/// ```
#[derive(Debug)]
pub struct Lab<T: Technology>(Machine<TechRecipe<T>>)
where
    TechRecipe<T>: Recipe;

impl<T: Technology> Lab<T>
where
    TechRecipe<T>: Recipe,
{
    /// Creates a new `Lab` producing research points for the specified technology.
    pub fn build(
        tick: &Tick,
        technology: &T,
        iron: Bundle<Iron, 20>,
        copper: Bundle<Copper, 15>,
    ) -> Self {
        let token = creation_token();
        let _ = (technology, iron, copper);
        Self(Machine::new(token, tick))
    }

    /// Changes the technology this `Lab` is producing research points for.
    pub fn change_technology<T2: Technology>(
        self,
        technology: &T2,
    ) -> Result<Lab<T2>, MachineNotEmptyError<Self>>
    where
        TechRecipe<T2>: Recipe,
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
        tick: &'a Tick,
    ) -> &'a mut <TechRecipe<T> as Recipe>::InputResources {
        self.0.inputs(tick)
    }

    /// Amount of each input resource needed for one recipe cycle
    pub const fn input_amounts(
        &self,
    ) -> <<TechRecipe<T> as Recipe>::InputBundle as MultiBundle>::AmountsType {
        <<TechRecipe<T> as Recipe>::InputBundle as MultiBundle>::AMOUNTS
    }

    /// Get a mutable reference to output buffers.
    pub fn outputs<'a>(
        &'a mut self,
        tick: &'a Tick,
    ) -> &'a mut <TechRecipe<T> as Recipe>::OutputResources {
        self.0.outputs(tick)
    }
}
