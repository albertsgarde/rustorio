#![forbid(unsafe_code)]
#![forbid(internal_features)]

use rustorio::{self, Bundle, Tick, gamemodes::Standard, resources::Point};

type GameMode = Standard;

type StartingResources = <GameMode as rustorio::GameMode>::StartingResources;

fn main() {
    rustorio::play::<GameMode>(user_main);
}

#[allow(unused_variables)]
#[allow(unused_mut)]
fn user_main(mut tick: Tick, starting_resources: StartingResources) -> (Tick, Bundle<Point, 200>) {
    let StartingResources {
        iron,
        mut iron_territory,
        mut copper_territory,
        steel_technology,
    } = starting_resources;

    // Standard mode asks you to build a larger production chain than the tutorial:
    // smelt iron and copper, craft copper wire, craft circuits, make red science,
    // research steel, research points, then produce 200 points.
    //
    // Resource<T> holds a flexible amount and is used for machine buffers.
    // Bundle<T, N> holds exactly N items and is used for build costs, recipe inputs,
    // research costs, and the final victory return value.
    //
    // Use `.bundle::<N>()` to pay an exact cost from a Resource<T> buffer, e.g.
    // `let iron_for_furnace = iron_buffer.bundle::<10>().unwrap();`
    // If the required amount is obvious from context, `.bundle().unwrap()` is enough.
    //
    // Machine input and output buffers are tuples in recipe order. For example,
    // `furnace.inputs(&tick).0 += ore;` fills the first input buffer, and
    // `furnace.outputs(&tick).0.empty()` drains the first output buffer
    // (remember to store the output somewhere).
    //
    // Research works the same way: a Lab outputs ResearchPoint<T> resources.
    // Collect enough, convert them to a bundle, then call `technology.research(...)`
    // to unlock the next recipe or technology.

    todo!("Return the `tick` and the victory resources to win the game!")
}
