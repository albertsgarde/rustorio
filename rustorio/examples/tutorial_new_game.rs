#![forbid(unsafe_code)]
#![forbid(internal_features)]

use rustorio::{self, Bundle, MainTick, gamemodes::Tutorial, resources::Copper};

type GameMode = Tutorial;

type StartingResources = <GameMode as rustorio::GameMode>::StartingResources;

fn main() {
    rustorio::play::<GameMode>(user_main);
}

#[allow(unused_variables)]
#[allow(unused_mut)]
fn user_main(
    mut tick: MainTick,
    starting_resources: StartingResources,
) -> (MainTick, Bundle<'static, Copper, 4>) {
    tick.log(true);

    let StartingResources {
        iron,
        mut iron_territory,
        mut copper_territory,
        guide,
    } = starting_resources;

    // To start, run the game using `rustorio play tutorial` (or whatever this save is called), and follow the hint.
    // If you get stuck, try giving the guide other objects you've found, like the `tick` object.
    guide.hint(iron)
}
