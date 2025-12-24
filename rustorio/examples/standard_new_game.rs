#![forbid(unsafe_code)]

use rustorio::{self, Bundle, Tick, gamemodes::Standard, resources::Point};

type GameMode = Standard;

type StartingResources = <GameMode as rustorio::GameMode>::StartingResources;

fn main() {
    rustorio::play::<GameMode>(user_main);
}

#[allow(unused_variables)]
#[allow(unused_mut)]
fn user_main(mut tick: Tick, starting_resources: StartingResources) -> (Tick, Bundle<Point, 10>) {
    let StartingResources {
        iron,
        iron_territory,
        copper_territory,
        points_technology,
    } = starting_resources;

    todo!("Return the `tick` and the victory resources to win the game!")
}
