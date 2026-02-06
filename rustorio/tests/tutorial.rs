use rustorio::{Tick, buildings::Furnace, gamemodes::Tutorial, recipes::CopperSmelting};

type GameMode = Tutorial;

type StartingResources = <GameMode as rustorio::GameMode>::StartingResources;
type VictoryResources = <GameMode as rustorio::GameMode>::VictoryResources;

#[test]
fn tutorial() {
    rustorio::play::<GameMode>(user_main);
}

fn user_main(mut tick: Tick, starting_resources: StartingResources) -> (Tick, VictoryResources) {
    tick.log(true);

    let StartingResources {
        iron,
        iron_territory: _iron_territory,
        mut copper_territory,
        guide: _,
    } = starting_resources;

    let mut furnace = Furnace::build(&tick, CopperSmelting, iron);

    let copper_ore = copper_territory.hand_mine::<8>(&mut tick);

    furnace.inputs(&tick).0 += copper_ore;
    tick.advance_until(|tick| furnace.outputs(tick).0.amount() >= 4, 100);

    let win_bundle = furnace.outputs(&tick).0.bundle().unwrap();
    (tick, win_bundle)
}
