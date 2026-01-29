# Rustorio &emsp; [![Latest Version]][crates.io] [![Docs]][docs.rs] [![Discord]][discord.gg]

[Latest Version]: https://img.shields.io/crates/v/rustorio.svg
[crates.io]: https://crates.io/crates/rustorio
[Docs]: https://img.shields.io/docsrs/rustorio.svg
[docs.rs]: https://docs.rs/rustorio
[Discord]: https://dcbadge.limes.pink/api/server/uKJugp85Fk?style=flat
[discord.gg]: https://discord.gg/uKJugp85Fk

The first game written _and played_ entirely in Rust's type system. Not only do you play by writing Rust code, but the rules of the game are enforced by the Rust compiler. If you can write the program so it compiles and doesn't panic, you win!

A while ago I realized that with Rust's affine types and ownership, it was possible to simulate resource scarcity. Combined with the richness of the type system, I wondered if it was possible to create a game with the rules enforced entirely by the Rust compiler. Well, it looks like it is. 

The actual mechanics are heavily inspired by Factorio and similar games, but you play by filling out a function, and if it compiles and doesn't panic, you've won! As an example, in the tutorial level, you start with [10 iron](https://docs.rs/rustorio/latest/rustorio/gamemodes/struct.TutorialStartingResources.html)

```rust
fn user_main(mut tick: Tick, starting_resources: StartingResources) -> (Tick, Bundle<Copper, 4>) {
    let StartingResources { iron, mut copper_territory } = starting_resources;
```

You can use this to create a [`Furnace`](https://docs.rs/rustorio/latest/rustorio/buildings/struct.Furnace.html) to turn copper ore (which you get by using [`Territory::handmine`](https://docs.rs/rustorio/latest/rustorio/territory/struct.Territory.html#method.hand_mine)) into copper.

```rust
    let mut furnace = Furnace::build(&tick, CopperSmelting, iron);

    let copper_ore = copper_territory.hand_mine::<8>(&mut tick);

    furnace.inputs(&tick).0 += copper_ore;
    tick.advance_until(|tick| furnace.outputs(tick).0.amount() >= 1, 100);
```

Because none of these types implement `Copy` or `Clone` and because they all have hidden fields, the only way (I hope) to create them is through the use of other resources, or in the case of ore, [time](https://docs.rs/rustorio/latest/rustorio/struct.Tick.html).

The game is pretty simple and easy right now, but I have many ideas for future features. I really enjoy figuring out how to wrangle the Rust language into doing what I want in this way, and I really hope some of you enjoy this kind of thing as well. Please do give it a try and tell me what you think. I'm especially interested in hearing what makes it work or not work as a *game*. It's a very weird user interface, so we're kinda reinventing some parts of game design from scratch here.

## How to play
1. Install [Rust](https://www.rust-lang.org/tools/install). Specifically it's
   important to have the entire rustup toolchain and cargo, all of which you get
   automatically by following the instructions in the link.
2. Install `rustorio` by running `cargo +nightly install rustorio`.
3. Set up a new Rustorio project by running `rustorio setup <path>`, where
   `<path>` is the directory you want to create the project in (defaults to
   '.').
4. Under `src/bin/tutorial/` you will find a tutorial save. You can start by
   playing that one.
5. Playing the game consists of filling out the `user_main` function in the
   `main.rs` file in the save game folder created for you.
6. Run with `rustorio play <save name>` (e.g. `rustorio play tutorial`). This will compile and run your save. If
   it compiles and completes without panicking, you win! It'll then tell you how
   many ticks it took you to win.

### After the tutorial
To play other game modes, run `rustorio new-game SAVENAME -g GAMEMODE` and specify a game mode like `standard`.
Use `rustorio new-game --help` to see all available game modes.

## Rules
The rules are mostly enforced by the compiler. The only two (current) exceptions are:
1. Do not remove `#![forbid(unsafe_code)]` at the top of the `main.rs` file.
2. Do not exploit unsoundness in the compiler.

Both these would enable you to bypass the rules enforced by the compiler and make the game trivial.
If you find other ways to bypass the rules or to do things that feel like cheating (e.g. [this ingenious exploit](https://github.com/albertsgarde/rustorio/issues/3)),
please file an issue.
Part of my interest in this project is seeing how close we can get to rule out all possible
cheating vectors using only the Rust compiler. So I'd love to hear about any ways to cheat.

## Help
Documentation for the Rustorio library can be found
[here](https://docs.rs/rustorio/latest/rustorio/). A good place to start
is to build a
[furnace](https://docs.rs/rustorio/latest/rustorio/buildings/struct.Furnace.html)
and start
[mining](https://docs.rs/rustorio/latest/rustorio/territory/struct.Territory.html#method.hand_mine) and
smelting iron. Alternatively, you can work backwards by looking at the
[recipe](https://docs.rs/rustorio/latest/rustorio/recipes/struct.PointRecipe.html)
for points to figure out how to get them.
