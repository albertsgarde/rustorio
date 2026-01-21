#![forbid(unsafe_code)]

use rustorio::{
    self, Bundle, HandRecipe, ResearchPoint, Resource, Technology, Tick,
    buildings::{Assembler, Furnace, Lab},
    gamemodes::Standard,
    recipes::{
        CopperSmelting, CopperWireRecipe, ElectronicCircuitRecipe, IronSmelting, RedScienceRecipe,
    },
    research::{PointsTechnology, SteelTechnology},
    resources::{CopperWire, Iron, Point},
    territory::Miner,
};

type GameMode = Standard;

type StartingResources = <GameMode as rustorio::GameMode>::StartingResources;

#[test]
fn standard_simple() {
    rustorio::play::<GameMode>(user_main);
}

fn craft<R: HandRecipe>(inputs: R::InputBundle, tick: &mut Tick) -> R::OutputBundle {
    R::craft(tick, inputs)
}

fn user_main(mut tick: Tick, starting_resources: StartingResources) -> (Tick, Bundle<Point, 200>) {
    let StartingResources {
        iron,
        mut iron_territory,
        mut copper_territory,
        steel_technology,
    } = starting_resources;

    let mut iron_furnace = Furnace::build(&tick, IronSmelting, iron);

    let mut copper_furnace = loop {
        iron_furnace.inputs(&tick).0 += iron_territory.hand_mine::<1>(&mut tick);
        if let Ok(iron) = iron_furnace.outputs(&tick).0.bundle() {
            break Furnace::build(&tick, CopperSmelting, iron);
        }
    };

    println!(
        "Created additional furnace for copper at tick {}",
        tick.cur()
    );

    while iron_furnace.outputs(&tick).0.amount() < 60
        || copper_furnace.outputs(&tick).0.amount() < 30
    {
        if copper_furnace.inputs(&tick).0.amount() < iron_furnace.inputs(&tick).0.amount() {
            copper_furnace.inputs(&tick).0 += copper_territory.hand_mine::<1>(&mut tick);
        } else {
            iron_furnace.inputs(&tick).0 += iron_territory.hand_mine::<1>(&mut tick);
        }
    }
    for _ in 0..3 {
        iron_territory
            .add_miner(
                &tick,
                Miner::build(
                    iron_furnace.outputs(&tick).0.bundle().unwrap(),
                    copper_furnace.outputs(&tick).0.bundle().unwrap(),
                ),
            )
            .unwrap();
    }
    for _ in 0..3 {
        copper_territory
            .add_miner(
                &tick,
                Miner::build(
                    iron_furnace.outputs(&tick).0.bundle().unwrap(),
                    copper_furnace.outputs(&tick).0.bundle().unwrap(),
                ),
            )
            .unwrap();
    }

    let mut copper_wire: Resource<CopperWire> = Resource::new_empty();
    let mut iron: Resource<Iron> = Resource::new_empty();
    while copper_wire.amount() < 12 || iron.amount() < 6 {
        iron_furnace
            .inputs(&tick)
            .0
            .add(iron_territory.resources(&tick).empty());
        copper_furnace
            .inputs(&tick)
            .0
            .add(copper_territory.resources(&tick).empty());
        iron += iron_furnace.outputs(&tick).0.empty();
        if let Ok(copper) = copper_furnace.outputs(&tick).0.bundle() {
            copper_wire += craft::<CopperWireRecipe>((copper,), &mut tick).0;
        }
        tick.advance();
    }

    let mut copper_wire_assembler = Assembler::build(
        &tick,
        CopperWireRecipe,
        copper_wire.bundle().unwrap(),
        iron.bundle().unwrap(),
    );

    println!("Created assembler for copper wire at tick {}", tick.cur());

    let mut circuit_assembler = loop {
        iron_furnace
            .inputs(&tick)
            .0
            .add(iron_territory.resources(&tick).empty());
        copper_furnace
            .inputs(&tick)
            .0
            .add(copper_territory.resources(&tick).empty());
        iron += iron_furnace.outputs(&tick).0.empty();
        copper_wire_assembler.inputs(&tick).0 += copper_furnace.outputs(&tick).0.empty();
        copper_wire += copper_wire_assembler.outputs(&tick).0.empty();
        if iron.amount() >= 6 && copper_wire.amount() >= 12 {
            break Assembler::build(
                &tick,
                ElectronicCircuitRecipe,
                copper_wire.bundle().unwrap(),
                iron.bundle().unwrap(),
            );
        }
        tick.advance();
    };

    println!("Created assembler for circuits at tick {}", tick.cur());

    let mut red_science_assembler = loop {
        iron_furnace
            .inputs(&tick)
            .0
            .add(iron_territory.resources(&tick).empty());
        copper_furnace
            .inputs(&tick)
            .0
            .add(copper_territory.resources(&tick).empty());
        iron += iron_furnace.outputs(&tick).0.empty();
        copper_wire_assembler.inputs(&tick).0 += copper_furnace.outputs(&tick).0.empty();
        copper_wire += copper_wire_assembler.outputs(&tick).0.empty();
        if iron.amount() >= 10 && copper_wire.amount() >= 6 {
            break Assembler::build(
                &tick,
                RedScienceRecipe,
                copper_wire.bundle().unwrap(),
                iron.bundle().unwrap(),
            );
        }
        tick.advance();
    };

    println!("Created assembler for red science at tick {}", tick.cur());

    let mut lab = loop {
        if copper_furnace.inputs(&tick).0.amount() < iron_furnace.inputs(&tick).0.amount() {
            copper_furnace.inputs(&tick).0 += copper_territory.hand_mine::<1>(&mut tick);
        } else {
            iron_furnace.inputs(&tick).0 += iron_territory.hand_mine::<1>(&mut tick);
        }
        iron_furnace
            .inputs(&tick)
            .0
            .add(iron_territory.resources(&tick).empty());
        copper_furnace
            .inputs(&tick)
            .0
            .add(copper_territory.resources(&tick).empty());
        iron += iron_furnace.outputs(&tick).0.empty();
        if iron.amount() >= 20
            && let Ok(copper) = copper_furnace.outputs(&tick).0.bundle()
        {
            break Lab::build(&tick, &steel_technology, iron.bundle().unwrap(), copper);
        }
        tick.advance();
    };

    println!("Created lab at tick {}", tick.cur());

    let mut research_points: Resource<ResearchPoint<SteelTechnology>> = Resource::new_empty();
    while research_points.amount() < SteelTechnology::REQUIRED_RESEARCH_POINTS {
        research_points += lab.outputs(&tick).0.empty();
        lab.inputs(&tick).0 += red_science_assembler.outputs(&tick).0.empty();

        let iron = iron_furnace
            .outputs(&tick)
            .0
            .split_off_max(5 - red_science_assembler.inputs(&tick).0.amount());
        red_science_assembler.inputs(&tick).0 += iron;
        red_science_assembler.inputs(&tick).1 += circuit_assembler.outputs(&tick).0.empty();

        circuit_assembler.inputs(&tick).0 += iron_furnace.outputs(&tick).0.empty();
        circuit_assembler.inputs(&tick).1 += copper_wire_assembler.outputs(&tick).0.empty();

        copper_wire_assembler.inputs(&tick).0 += copper_furnace.outputs(&tick).0.empty();

        iron_furnace
            .inputs(&tick)
            .0
            .add(iron_territory.resources(&tick).empty());
        copper_furnace
            .inputs(&tick)
            .0
            .add(copper_territory.resources(&tick).empty());
        tick.advance();
    }

    let (steel_smelting, points_technology) =
        steel_technology.research(research_points.bundle().unwrap());

    println!("Researched steel technology at tick {}", tick.cur());

    let mut lab = lab.change_technology(&points_technology).unwrap();

    let mut research_points: Resource<ResearchPoint<PointsTechnology>> = Resource::new_empty();
    while research_points.amount() < PointsTechnology::REQUIRED_RESEARCH_POINTS {
        research_points += lab.outputs(&tick).0.empty();
        lab.inputs(&tick).0 += red_science_assembler.outputs(&tick).0.empty();

        let iron = iron_furnace
            .outputs(&tick)
            .0
            .split_off_max(5 - red_science_assembler.inputs(&tick).0.amount());
        red_science_assembler.inputs(&tick).0 += iron;
        red_science_assembler.inputs(&tick).1 += circuit_assembler.outputs(&tick).0.empty();

        circuit_assembler.inputs(&tick).0 += iron_furnace.outputs(&tick).0.empty();
        circuit_assembler.inputs(&tick).1 += copper_wire_assembler.outputs(&tick).0.empty();

        copper_wire_assembler.inputs(&tick).0 += copper_furnace.outputs(&tick).0.empty();

        iron_furnace
            .inputs(&tick)
            .0
            .add(iron_territory.resources(&tick).empty());
        copper_furnace
            .inputs(&tick)
            .0
            .add(copper_territory.resources(&tick).empty());
        tick.advance();
    }

    let point_recipe = points_technology.research(research_points.bundle().unwrap());

    println!("Researched points technology at tick {}", tick.cur());

    let mut steel_furnace = loop {
        iron_furnace
            .inputs(&tick)
            .0
            .add(iron_territory.resources(&tick).empty());
        copper_furnace
            .inputs(&tick)
            .0
            .add(copper_territory.resources(&tick).empty());
        iron += iron_furnace.outputs(&tick).0.empty();
        if iron.amount() >= 10 {
            break Furnace::build(&tick, steel_smelting, iron.bundle().unwrap());
        }
        tick.advance();
    };

    println!("Created furnace for steel at tick {}", tick.cur());

    let mut copper_wire: Resource<CopperWire> = Resource::new_empty();
    let mut iron: Resource<Iron> = Resource::new_empty();
    let mut point_assembler = loop {
        iron_furnace
            .inputs(&tick)
            .0
            .add(iron_territory.resources(&tick).empty());
        copper_furnace
            .inputs(&tick)
            .0
            .add(copper_territory.resources(&tick).empty());

        copper_wire_assembler.inputs(&tick).0 += copper_furnace.outputs(&tick).0.empty();
        copper_wire += copper_wire_assembler.outputs(&tick).0.empty();
        iron += iron_furnace.outputs(&tick).0.empty();

        if copper_wire.amount() >= 12 && iron.amount() >= 6 {
            break Assembler::build(
                &tick,
                point_recipe,
                copper_wire.bundle().unwrap(),
                iron.bundle().unwrap(),
            );
        }
        tick.advance();
    };

    println!("Created assembler for points at tick {}", tick.cur());

    let mut points: Resource<Point> = Resource::new_empty();
    while points.amount() < 200 {
        points += point_assembler.outputs(&tick).0.empty();

        point_assembler.inputs(&tick).0 += circuit_assembler.outputs(&tick).0.empty();
        point_assembler.inputs(&tick).1 += steel_furnace.outputs(&tick).0.empty();

        let mut iron_output = iron_furnace.outputs(&tick).0.empty();
        // Prioritize steel furnace since it needs 5 iron per steel
        let iron_for_steel = iron_output.split_off_max(5 - steel_furnace.inputs(&tick).0.amount());
        steel_furnace.inputs(&tick).0 += iron_for_steel;
        // Rest goes to circuit assembler
        circuit_assembler.inputs(&tick).0 += iron_output;

        circuit_assembler.inputs(&tick).1 += copper_wire_assembler.outputs(&tick).0.empty();

        copper_wire_assembler.inputs(&tick).0 += copper_furnace.outputs(&tick).0.empty();

        iron_furnace
            .inputs(&tick)
            .0
            .add(iron_territory.resources(&tick).empty());
        copper_furnace
            .inputs(&tick)
            .0
            .add(copper_territory.resources(&tick).empty());
        tick.advance();
    }

    println!("Produced 200 points at tick {}", tick.cur());

    (tick, points.bundle().unwrap())
}
