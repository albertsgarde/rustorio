#![forbid(unsafe_code)]
#![forbid(internal_features)]

use rustorio::{self, Bundle, MainTick, gamemodes::Standard, resources::Point};

type GameMode = Standard;

type StartingResources = <GameMode as rustorio::GameMode>::StartingResources;

fn main() {
    rustorio::play::<GameMode>(user_main);
}

#[allow(unused_variables)]
#[allow(unused_mut)]
fn user_main(
    mut tick: MainTick,
    starting_resources: StartingResources,
) -> (MainTick, Bundle<'static, Point, 200>) {
    let StartingResources {
        iron,
        mut iron_territory,
        mut copper_territory,
        steel_technology,
    } = starting_resources;

    todo!("Return the `tick` and the victory resources to win the game!")
}
