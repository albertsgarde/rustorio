#![forbid(unsafe_code)]

use rustorio::{self, Bundle, Tick, gamemodes::Tutorial, resources::Copper};

type GameMode = Tutorial;

type StartingResources = <GameMode as rustorio::GameMode>::StartingResources;

fn main() {
    rustorio::play::<GameMode>(user_main);
}

#[allow(unused_variables)]
#[allow(unused_mut)]
fn user_main(mut tick: Tick, starting_resources: StartingResources) -> (Tick, Bundle<Copper, 4>) {
    let StartingResources {
        iron,
        iron_territory,
        copper_territory,
        guide,
    } = starting_resources;

    // To start, run the game using `rustorio play tutorial` (or whatever this save is called), and follow the hint.
    // If you get stuck, try giving the guide other objects you've found, like the `tick` object.
    guide.hint(iron)
}
