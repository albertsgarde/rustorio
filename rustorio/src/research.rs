//! Technologies can be unlocked by consuming science packs.
//! They usually unlock new recipes or further technologies.
//!
//! For example, if you want to produce points using the `PointRecipe` recipe,
//! you must first unlock it by researching the `PointsTechnology` technology.
//!
//! This module defines the technologies available in Rustorio.

use rustorio_engine::{
    Sealed,
    machine::{Machine, MachineNotEmptyError},
    mod_reexports::Tick,
    recipe::{Recipe, RecipeEx},
    research::{ResearchPoint, TechRecipe, Technology, TechnologyEx, tech_recipe, technology_doc},
    resource_type,
};

use crate::{
    Bundle,
    recipes::PointRecipe,
    resources::{Copper, Iron},
};

resource_type!(
    /// The basic science pack used for researching technologies.
    RedScience
);

/// Technology that unlocks the ability to produce points.
#[technology_doc]
#[derive(Debug, TechnologyEx)]
#[research_inputs((1, RedScience))]
#[research_point_cost(10)]
#[research_ticks(50)]
#[non_exhaustive]
pub struct PointsTechnology;
impl Sealed for PointsTechnology {}

impl Technology for PointsTechnology {
    const NAME: &'static str = "Points";
    type Unlocks = PointRecipe;

    fn research(
        self,
        research_points: Bundle<ResearchPoint<Self>, { Self::RESEARCH_POINT_COST }>,
    ) -> Self::Unlocks {
        let _ = research_points;
        PointRecipe {}
    }
}

/// Building that creates research points.
/// Set it to produce research points for a specific technology either when [`build`](Lab::build)ing it,
/// or using [`change_technology`](Lab::change_technology).
pub struct Lab<T: Technology>(Machine<TechRecipe<T>>)
where
    TechRecipe<T>: RecipeEx;

impl<T: Technology> Lab<T>
where
    TechRecipe<T>: RecipeEx,
{
    /// Creates a new `Lab` producing research points for the specified technology.
    pub fn build(
        tick: &Tick,
        technology: &T,
        iron: Bundle<Iron, 20>,
        copper: Bundle<Copper, 15>,
    ) -> Self {
        let _ = (technology, iron, copper);
        Self(Machine::new(tick))
    }

    /// Changes the technology this `Lab` is producing research points for.
    pub fn change_technology<T2: Technology>(
        self,
        technology: &T2,
    ) -> Result<Lab<T2>, MachineNotEmptyError<Self>>
    where
        TechRecipe<T2>: RecipeEx,
    {
        let _ = technology;
        match self.0.change_recipe(tech_recipe()) {
            Ok(machine) => Ok(Lab(machine)),
            Err(err) => Err(err.map_machine(Lab)),
        }
    }

    /// Get a mutable reference to input buffers.
    pub fn inputs(&mut self, tick: &Tick) -> &mut <TechRecipe<T> as Recipe>::Inputs {
        self.0.inputs(tick)
    }

    /// Get a mutable reference to output buffers.
    pub fn outputs(&mut self, tick: &Tick) -> &mut <TechRecipe<T> as Recipe>::Outputs {
        self.0.outputs(tick)
    }
}
