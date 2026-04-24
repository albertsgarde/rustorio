# Rustorio AI Player

You are playing Rustorio, a factory-building game where the game logic is enforced by Rust's type system. Your goal is to win the Tutorial game mode.

## Your goal

Get `rustorio play tutorial` to exit with code 0.

## How to learn the API

The API documentation has been pre-generated for you. Read the HTML files in `target/doc/rustorio/`. Start with the index:

- `target/doc/rustorio/index.html` — top-level overview
- `target/doc/rustorio/gamemodes/struct.Tutorial.html` — the game mode you're playing
- `target/doc/rustorio/gamemodes/struct.TutorialStartingResources.html` — what you start with

Do not read source files from outside your current directory. The docs are your only permitted API reference.

## Your save file

Edit `src/bin/tutorial/main.rs`. This is the only file you should write to. 

## How to play

The `README.md` file in this directory is copied from the Rustorio github. Follow the instructions there and use the API docs as reference.

## Testing

Do not use `cargo` commands directly. Use `rustorio play tutorial` or `rustorio play tutorial 2>&1` to run the game. These are the only two bash commands you have access to and are the only you should need.

## Issues

If you feel like you need to use a command or access a file that is denied to you, please tell me and I'll consider adding it to your permissions. If you find a bug in the game or the API, please report it.

## Done

You are done when `rustorio play tutorial` exits with code 0. Report the tick count from the output.
