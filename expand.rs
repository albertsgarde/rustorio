pub mod research {
    //! Technologies can be unlocked by consuming science packs.
    //! They usually unlock new recipes or further technologies.
    //!
    //! For example, if you want to produce points using the `PointRecipe` recipe,
    //! you must first unlock it by researching the `PointsTechnology` technology.
    //!
    //! This module defines the technologies available in Rustorio.
    use rustorio_engine::{
        Sealed, research::{ResearchPoint, Technology, TechnologyEx, technology_doc},
        resource_type,
    };
    use crate::{Bundle, recipes::{PointRecipe, SteelSmelting}};
    /// The basic science pack used for researching technologies in [`Lab`](crate::buildings::Lab)s.
    ///
    /// Crafted from [this](crate::recipes::RedScienceRecipe) recipe.
    pub struct RedScience;
    #[automatically_derived]
    impl ::core::fmt::Debug for RedScience {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "RedScience")
        }
    }
    impl ::rustorio_engine::Sealed for RedScience {}
    impl ::rustorio_engine::ResourceType for RedScience {
        const NAME: &'static str = "RedScience";
    }
    /// Allows the further refining of iron into steel.
    #[research_inputs((1, RedScience))]
    #[research_point_cost(20)]
    #[research_ticks(5)]
    #[non_exhaustive]
    /**### Cost
- [`RedScience`] :  1

**Ticks**: 5

**Research points required**: 20*/
    pub struct SteelTechnology;
    #[automatically_derived]
    impl ::core::fmt::Debug for SteelTechnology {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "SteelTechnology")
        }
    }
    impl ::rustorio_engine::research::TechnologyEx for SteelTechnology {
        const POINT_RECIPE_TIME: u64 = 5;
        const REQUIRED_RESEARCH_POINTS_EX: u32 = 20;
        type InputBundle = (::rustorio_engine::resources::Bundle<'t, RedScience, 1u32>,);
    }
    impl Sealed for SteelTechnology {}
    impl Technology for SteelTechnology {
        const NAME: &'static str = "Steel";
        type Unlocks = (SteelSmelting, PointsTechnology);
        fn research(
            self,
            research_points: Bundle<
                ResearchPoint<Self>,
                { Self::REQUIRED_RESEARCH_POINTS },
            >,
        ) -> Self::Unlocks {
            let _ = research_points;
            (SteelSmelting, PointsTechnology)
        }
    }
    /// Unlocks the ability to produce points.
    #[research_inputs((1, RedScience))]
    #[research_point_cost(50)]
    #[research_ticks(5)]
    #[non_exhaustive]
    /**### Cost
- [`RedScience`] :  1

**Ticks**: 5

**Research points required**: 50*/
    pub struct PointsTechnology;
    #[automatically_derived]
    impl ::core::fmt::Debug for PointsTechnology {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "PointsTechnology")
        }
    }
    impl ::rustorio_engine::research::TechnologyEx for PointsTechnology {
        const POINT_RECIPE_TIME: u64 = 5;
        const REQUIRED_RESEARCH_POINTS_EX: u32 = 50;
        type InputBundle = (::rustorio_engine::resources::Bundle<'t, RedScience, 1u32>,);
    }
    impl Sealed for PointsTechnology {}
    impl Technology for PointsTechnology {
        const NAME: &'static str = "Points";
        type Unlocks = PointRecipe;
        fn research(
            self,
            research_points: Bundle<
                ResearchPoint<Self>,
                { Self::REQUIRED_RESEARCH_POINTS },
            >,
        ) -> Self::Unlocks {
            let _ = research_points;
            PointRecipe {}
        }
    }
}
