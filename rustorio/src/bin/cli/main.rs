use std::{
    fmt::Display,
    fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use dialoguer::Confirm;
use thiserror::Error;

// Macro to build paths to game bin files relative to workspace root
macro_rules! game_bin_file {
    ($gamemode:expr) => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/",
            $gamemode,
            "_new_game.rs"
        )
    };
}

const RUST_TOOLCHAIN: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/rust-toolchain.toml"));

#[derive(Error, Debug)]
pub enum RunCommandError {
    CommandFailed(ExitStatus),
    IoError(io::Error),
}

impl Display for RunCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunCommandError::CommandFailed(status) => {
                write!(f, "Command failed with exit status: {}", status)
            }
            RunCommandError::IoError(err) => write!(f, "IO error occurred: {}", err),
        }
    }
}

pub trait RunCommandExt {
    fn run(&mut self) -> Result<(), RunCommandError>;
}

impl RunCommandExt for Command {
    fn run(&mut self) -> Result<(), RunCommandError> {
        let status = self.status().map_err(RunCommandError::IoError)?;
        if !status.success() {
            return Err(RunCommandError::CommandFailed(status));
        }
        Ok(())
    }
}

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Setup(args) => args.run(),
            Commands::NewGame(args) => args.run(),
            Commands::Play(args) => args.run(),
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Set up a new Rustorio folder in the specified directory.
    Setup(SetupArgs),
    /// Create a new save game with the specified name. If not called in a Rustorio project, will prompt to set one up.
    NewGame(NewGameArgs),
    /// Play an existing save game with the specified name.
    /// Can only be run in a Rustorio project.
    ///
    /// For example, in most Rustorio folders, there'll be a `tutorial` save game.
    /// To run it, use `rustorio play tutorial`.
    Play(PlayArgs),
}

#[derive(Args)]
pub struct SetupArgs {
    #[clap(default_value = ".")]
    path: PathBuf,
    #[clap(long, default_value_t = true)]
    include_tutorial: bool,
}

impl SetupArgs {
    pub fn run(&self) -> Result<()> {
        if !self.path.exists() {
            bail!(
                "The specified path '{}' does not exist.",
                self.path.display()
            );
        }

        let canonical_path = self
            .path
            .canonicalize()
            .context("Could not canonicalize specified path")?;

        if canonical_path.join("rustorio").exists() {
            bail!(
                "There is already a 'rustorio' directory at the specified path '{}'. Please run the command in an empty directory.",
                canonical_path.display()
            );
        }

        println!("Setting up Rustorio at '{}'...", canonical_path.display());
        // Run `cargo new --bin self.name` with the same `cargo` binary as used to build this CLI
        Command::new(env!("CARGO"))
            .arg("new")
            .arg("--bin")
            .arg("--name")
            .arg("rustorio-game")
            .arg("rustorio")
            .current_dir(&canonical_path)
            .run()
            .context("Failed to create new Rustorio project")?;
        let path = canonical_path.join("rustorio");
        Command::new(env!("CARGO"))
            .arg("add")
            .arg("rustorio")
            .arg("--no-default-features")
            .current_dir(&path)
            .run()
            .context("Failed to add Rustorio as a dependency")?;
        fs::write(path.join("rustorio.toml"), "").context("Failed to create rustorio.toml")?;
        fs::write(path.join("rust-toolchain.toml"), RUST_TOOLCHAIN)
            .context("Failed to create rust-toolchain file")?;
        let save_path = path.join("src").join("bin");
        fs::create_dir_all(&save_path).context("Failed to create save directory")?;
        if self.include_tutorial {
            let tutorial_start_file = GameMode::Tutorial.start_file();
            let tutorial_save_dir = save_path.join("tutorial");
            fs::create_dir_all(&tutorial_save_dir)
                .context("Failed to create tutorial save directory")?;
            fs::write(tutorial_save_dir.join("main.rs"), tutorial_start_file)
                .context("Failed to create tutorial/main.rs")?;
        }
        fs::remove_file(path.join("src").join("main.rs")).context("Failed to remove main.rs")?;
        println!(
            "Rustorio set up at '{}'! Open the directory in your favorite Rust editor to get started.",
            path.display()
        );
        Ok(())
    }
}

#[derive(ValueEnum, Clone)]
pub enum GameMode {
    Tutorial,
    Standard,
}

impl GameMode {
    pub const fn as_str(&self) -> &str {
        match self {
            GameMode::Tutorial => "tutorial",
            GameMode::Standard => "standard",
        }
    }

    pub fn start_file(&self) -> String {
        match self {
            GameMode::Tutorial => include_str!(game_bin_file!("tutorial")),
            GameMode::Standard => include_str!(game_bin_file!("standard")),
        }
        .replace("\n#[allow(unused_variables)]", "")
        .replace("\n#[allow(unused_mut)]", "")
    }
}

fn find_rustorio_root() -> Result<Option<std::path::PathBuf>> {
    let mut current_dir = Path::new(".")
        .canonicalize()
        .context("Failed to canonicalize current directory")?;
    loop {
        if current_dir.join("rustorio.toml").exists() {
            return Ok(Some(current_dir));
        }
        if !current_dir.pop() {
            break;
        }
    }
    Ok(None)
}

#[derive(Args)]
pub struct NewGameArgs {
    #[clap()]
    name: Option<String>,
    #[clap(long, short, value_enum, default_value_t = GameMode::Standard)]
    game_mode: GameMode,
}

impl NewGameArgs {
    pub fn run(&self) -> Result<()> {
        let rustorio_root = match find_rustorio_root()
            .context("Failed while looking for Rustorio root")?
        {
            Some(path) => path,
            None => {
                let setup_rustorio = Confirm::new()
                    .with_prompt(
                        "Could not find 'rustorio.toml'. Do you want to set up Rustorio here?",
                    )
                    .interact()
                    .context("Failed to confirm Rustorio setup")?;
                if setup_rustorio {
                    let setup_args = SetupArgs {
                        path: PathBuf::from("./"),
                        include_tutorial: false,
                    };
                    setup_args
                        .run()
                        .context("Failed while running command to set up Rustorio")?;
                    Path::new("rustorio")
                        .canonicalize()
                        .context("Failed to canonicalize Rustorio path")?
                } else {
                    bail!(
                        "Can only run command in a Rustorio project. Please run 'rustorio setup' first."
                    );
                }
            }
        };
        let rustorio_root = rustorio_root.as_path();
        let saves_dir = rustorio_root.join("src").join("bin");
        fs::create_dir_all(saves_dir.as_path()).context("Failed to create saves directory")?;
        let start_file = self.game_mode.start_file();
        let (save_game_path, save_game_name) = {
            let save_game_name = self.name.clone().unwrap_or_else(|| {
                println!("No save game name specified, generating one based on game mode...");
                let mut save_game_name = self.game_mode.as_str().to_string();
                while saves_dir.join(save_game_name.as_str()).exists() {
                    save_game_name = format!("{}_", save_game_name.as_str());
                }
                save_game_name
            });
            let save_game_path = saves_dir.join(save_game_name.as_str());
            if save_game_path.exists() {
                bail!("Save game '{}' already exists.", save_game_name.as_str());
            }
            (save_game_path, save_game_name)
        };
        fs::create_dir_all(&save_game_path).context("Failed to create save game directory")?;
        fs::write(save_game_path.join("main.rs").as_path(), start_file)
            .context("Failed to create save game file")?;
        println!(
            "New game '{}' with game mode '{}' created at {}! For help getting started, go to https://albertsgarde.github.io/rustorio",
            save_game_name,
            self.game_mode.as_str(),
            save_game_path.display()
        );
        Ok(())
    }
}

#[derive(Args)]
pub struct PlayArgs {
    /// The name of the save game to run.
    save_name: String,
}

impl PlayArgs {
    pub fn run(&self) -> Result<()> {
        let rustorio_root = if let Some(rustorio_root) =
            find_rustorio_root().context("Failed while looking for Rustorio root")?
        {
            rustorio_root
        } else {
            bail!(
                "Can only run command in a Rustorio project. Please either navigate to a Rustorio project or run 'rustorio setup' first."
            );
        };
        let save_game_path = rustorio_root.join("src").join("bin").join(&self.save_name);
        if !save_game_path.exists() {
            bail!("Save game '{}' does not exist.", self.save_name);
        }
        // Use a raw "cargo" to allow the toolchain file to take effect.
        Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg(&self.save_name)
            .current_dir(rustorio_root)
            .run()
            .context("Failed to run Rustorio game")?;
        Ok(())
    }
}

pub fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run()
}
