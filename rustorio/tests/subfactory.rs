use rustorio::{
    Bundle, Resource, Tick,
    buildings::{Assembler, Furnace},
    gamemodes::TutorialStartingResources,
    recipes::{CopperSmelting, CopperWireRecipe},
    resources::{CopperOre, CopperWire},
    territory::Territory,
};
use rustorio_engine::{bundle, resources::EngineToken, time_travel::TickSnapshot};

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

pub struct Subfactory {
    snapshot: TickSnapshot,
    furnace: Furnace<CopperSmelting>,
    assembler: Assembler<CopperWireRecipe>,
}

impl Subfactory {
    fn tick(&mut self, tick: &Tick) {
        // On each past tick, move resources from the furnace to the assembler as soon as they're
        // available.
        self.snapshot
            .on_each_tick(tick.snapshot(), |past| {
                let outputs = self.furnace.past_outputs(past).unwrap();
                if let Ok(bundle) = outputs.0.bundle::<1>() {
                    self.assembler.past_inputs(past).unwrap().0.add(bundle);
                }
            })
            .unwrap();
    }

    pub fn inputs<'a>(&'a mut self, tick: &'a Tick) -> &'a mut Resource<CopperOre> {
        self.tick(tick);
        &mut self.furnace.inputs(tick).0
    }
    pub fn outputs<'a>(&'a mut self, tick: &'a Tick) -> &'a mut Resource<CopperWire> {
        self.tick(tick);
        &mut self.assembler.outputs(tick).0
    }
}

fn user_main(mut tick: Tick, starting_resources: StartingResources) -> (Tick, VictoryResources) {
    tick.log(true);

    let StartingResources {
        mut copper_territory,
        furnace,
        assembler,
    } = starting_resources;

    let mut subfactory = Subfactory {
        snapshot: tick.snapshot(),
        furnace,
        assembler,
    };

    let copper_ore = copper_territory.hand_mine::<4>(&mut tick);

    *subfactory.inputs(&tick) += copper_ore;

    tick.advance_until(|tick| subfactory.outputs(tick).amount() >= 8, 100);

    let win_bundle = subfactory.outputs(&tick).bundle().unwrap();
    // Naive serial implementation takes 36 ticks.
    assert_eq!(tick.cur(), 33);
    (tick, win_bundle)
}
