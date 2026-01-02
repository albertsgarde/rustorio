use rustorio::{
    Technology, Tick,
    buildings::{Assembler, Furnace},
    gamemodes::Standard,
    recipes::{CopperSmelting, IronSmelting, RedScienceRecipe},
    research::Lab,
    territory::Miner,
};

type GameMode = Standard;

type StartingResources = <GameMode as rustorio::GameMode>::StartingResources;
type VictoryResources = <GameMode as rustorio::GameMode>::VictoryResources;

#[test]
pub fn standard_simple() {
    rustorio::play::<GameMode>(user_main);
}

fn user_main(mut tick: Tick, starting_resources: StartingResources) -> (Tick, VictoryResources) {
    let StartingResources {
        iron,
        mut iron_territory,
        mut copper_territory,
        points_technology,
    } = starting_resources;

    let mut iron_furnace = Furnace::build(&tick, IronSmelting, iron);

    let iron_ore = iron_territory.hand_mine::<40>(&mut tick);

    iron_furnace.inputs(&tick).0.add(iron_ore.to_resource());
    tick.advance_until(|tick| iron_furnace.outputs(tick).0.amount() >= 20, 1000);
    iron_territory
        .add_miner(
            &tick,
            Miner::build(iron_furnace.outputs(&tick).0.bundle().unwrap()),
        )
        .unwrap();
    copper_territory
        .add_miner(
            &tick,
            Miner::build(iron_furnace.outputs(&tick).0.bundle().unwrap()),
        )
        .unwrap();

    tick.advance_until(
        |tick| {
            iron_furnace
                .inputs(tick)
                .0
                .add(iron_territory.resources(tick).empty());
            iron_furnace.outputs(tick).0.amount() >= 10
        },
        100000,
    );

    let mut copper_furnace = Furnace::build(
        &tick,
        CopperSmelting,
        iron_furnace.outputs(&tick).0.bundle().unwrap(),
    );

    tick.advance_until(
        |tick| {
            iron_furnace
                .inputs(tick)
                .0
                .add(iron_territory.resources(tick).empty());
            copper_furnace
                .inputs(tick)
                .0
                .add(copper_territory.resources(tick).empty());
            iron_furnace.outputs(tick).0.amount() >= 500
                && copper_furnace.outputs(tick).0.amount() >= 500
        },
        100000,
    );

    let mut iron = iron_furnace.outputs(&tick).0.empty();
    println!("Iron ingots produced: {}", iron.amount());

    let mut copper = copper_furnace.outputs(&tick).0.empty();
    println!("Copper ingots produced: {}", copper.amount());

    let mut assembler = Assembler::build(
        &tick,
        RedScienceRecipe,
        iron.bundle().unwrap(),
        copper.bundle().unwrap(),
    );
    println!("Iron left: {}", iron.amount());
    println!("Copper left: {}", copper.amount());

    assembler.inputs(&tick).0.add(iron.bundle::<10>().unwrap());
    assembler
        .inputs(&tick)
        .1
        .add(copper.bundle::<10>().unwrap());
    tick.advance_until(|tick| assembler.outputs(tick).0.amount() >= 10, 100);
    let red_science = assembler.outputs(&tick).0.empty();

    let mut lab = Lab::build(
        &tick,
        &points_technology,
        iron.bundle().unwrap(),
        copper.bundle().unwrap(),
    );

    lab.inputs(&tick).0.add(red_science);
    tick.advance_until(|tick| lab.inputs(tick).0.amount() == 0, 1000);

    let tech_points = lab.outputs(&tick).0.bundle().unwrap();

    let points_recipe = points_technology.research(tech_points);
    println!("Points researched!");

    println!("Iron left: {}", iron.amount());
    println!("Copper left: {}", copper.amount());

    let mut assembler = assembler.change_recipe(points_recipe).unwrap();

    assembler.inputs(&tick).0.add(iron);
    assembler.inputs(&tick).1.add(copper);
    tick.advance_until(|tick| assembler.outputs(tick).0.amount() >= 10, 10000);

    let points = assembler.outputs(&tick).0.bundle().unwrap();
    (tick, points)
}
