use rustorio::{
    Bundle, Tick,
    buildings::{Assembler, Furnace},
    gamemodes::TutorialStartingResources,
    recipes::{CopperSmelting, CopperWireRecipe},
    resources::{CopperOre, CopperWire},
    territory::Territory,
};
use rustorio_engine::{bundle, resources::EngineToken};

pub struct Test;

pub struct TestStartingResources {
    pub copper_territory: Territory<CopperOre>,
    pub furnace: Furnace<CopperSmelting>,
    pub assembler: Assembler<CopperWireRecipe>,
}

impl rustorio_engine::gamemodes::StartingResources for TestStartingResources {
    fn init(tk: &EngineToken, tick: &Tick) -> Self {
        let tutorial_resources = TutorialStartingResources::init(tk, tick);
        let furnace = Furnace::build(tick, CopperSmelting, bundle(tk));
        let assembler = Assembler::build(tick, CopperWireRecipe, bundle(tk), bundle(tk));
        Self {
            copper_territory: tutorial_resources.copper_territory,
            furnace,
            assembler,
        }
    }
}

impl rustorio::GameMode for Test {
    type StartingResources = TestStartingResources;
    type VictoryResources = Bundle<CopperWire, 8>;

    const NAME: &str = "test";
}

type GameMode = Test;
type StartingResources = <GameMode as rustorio::GameMode>::StartingResources;
type VictoryResources = <GameMode as rustorio::GameMode>::VictoryResources;

#[test]
fn tutorial() {
    rustorio::play::<GameMode>(user_main);
}

use copper_wire_factory::CopperWireFactory;
mod copper_wire_factory {
    use rustorio::{
        Resource, Tick,
        buildings::{Assembler, Furnace},
        recipes::{CopperSmelting, CopperWireRecipe},
        resources::{CopperOre, CopperWire},
    };
    use rustorio_engine::time_travel::{OnEachTick, PastTick, Subfactory};

    struct CopperWireFactoryInner {
        furnace: Furnace<CopperSmelting>,
        assembler: Assembler<CopperWireRecipe>,
    }

    impl OnEachTick for CopperWireFactoryInner {
        fn on_each_tick<'tick>(&mut self, tick: &PastTick<'tick>) {
            let outputs = self.furnace.past_outputs(tick).unwrap();
            if let Ok(bundle) = outputs.0.bundle::<1>() {
                self.assembler.past_inputs(tick).unwrap().0.add(bundle);
            }
        }
    }

    pub struct CopperWireFactory(Subfactory<CopperWireFactoryInner>);

    impl CopperWireFactory {
        pub const fn new(
            tick: &Tick,
            furnace: Furnace<CopperSmelting>,
            assembler: Assembler<CopperWireRecipe>,
        ) -> Self {
            CopperWireFactory(Subfactory::new(
                tick,
                CopperWireFactoryInner { furnace, assembler },
            ))
        }
        pub fn inputs<'a>(&'a mut self, tick: &'a Tick) -> &'a mut Resource<CopperOre> {
            &mut self.0.inner(tick).furnace.inputs(tick).0
        }
        pub fn outputs<'a>(&'a mut self, tick: &'a Tick) -> &'a mut Resource<CopperWire> {
            &mut self.0.inner(tick).assembler.outputs(tick).0
        }
    }
}

fn user_main(mut tick: Tick, starting_resources: StartingResources) -> (Tick, VictoryResources) {
    tick.log(true);

    let StartingResources {
        mut copper_territory,
        furnace,
        assembler,
    } = starting_resources;

    let mut subfactory = CopperWireFactory::new(&tick, furnace, assembler);

    let copper_ore = copper_territory.hand_mine::<4>(&mut tick);

    *subfactory.inputs(&tick) += copper_ore;

    tick.advance_until(|tick| subfactory.outputs(tick).amount() >= 8, 100);

    let win_bundle = subfactory.outputs(&tick).bundle().unwrap();
    // Naive serial implementation takes 36 ticks.
    assert_eq!(tick.cur(), 33);
    (tick, win_bundle)
}
