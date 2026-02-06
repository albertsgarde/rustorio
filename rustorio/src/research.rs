//! Technologies can be unlocked by consuming science packs.
//! They usually unlock new recipes or further technologies.
//!
//! For example, if you want to produce points using the `PointRecipe` recipe,
//! you must first unlock it by researching the `PointsTechnology` technology.
//!
//! This module defines the technologies available in Rustorio.

use rustorio_engine::{
    Sealed,
    research::{ResearchPoint, Technology, TechnologyEx, technology_doc},
    resource_type,
};

use crate::{
    Bundle,
    recipes::{PointRecipe, SteelSmelting},
};

resource_type!(
    /// The basic science pack used for researching technologies in [`Lab`](crate::buildings::Lab)s.
    ///
    /// Crafted from [this](crate::recipes::RedScienceRecipe) recipe.
    RedScience
);

/// Allows the further refining of iron into steel.
#[technology_doc]
#[derive(Debug, TechnologyEx)]
#[research_inputs((1, RedScience))]
#[research_point_cost(20)]
#[research_ticks(5)]
#[non_exhaustive]
pub struct SteelTechnology;
impl Sealed for SteelTechnology {}

impl Technology for SteelTechnology {
    const NAME: &'static str = "Steel";
    type Unlocks = (SteelSmelting, PointsTechnology);

    fn research(
        self,
        research_points: Bundle<ResearchPoint<Self>, { Self::REQUIRED_RESEARCH_POINTS }>,
    ) -> Self::Unlocks {
        let _ = research_points;
        (SteelSmelting, PointsTechnology)
    }
}

/// Unlocks the ability to produce points.
#[technology_doc]
#[derive(Debug, TechnologyEx)]
#[research_inputs((1, RedScience))]
#[research_point_cost(50)]
#[research_ticks(5)]
#[non_exhaustive]
pub struct PointsTechnology;
impl Sealed for PointsTechnology {}

impl Technology for PointsTechnology {
    const NAME: &'static str = "Points";
    type Unlocks = PointRecipe;

    fn research(
        self,
        research_points: Bundle<ResearchPoint<Self>, { Self::REQUIRED_RESEARCH_POINTS }>,
    ) -> Self::Unlocks {
        let _ = research_points;
        PointRecipe {}
    }
}
