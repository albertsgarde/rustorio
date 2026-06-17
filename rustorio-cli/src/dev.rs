use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use anyhow::{Context, Result, bail};
use chrono::Local;
use clap::{Args, Subcommand};

use crate::{GameMode, NewGameArgs, RunCommandExt, SetupArgs, project::Config};

fn find_repo_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()
        .context("Failed to get current directory")?
        .canonicalize()
        .context("Failed to canonicalize current directory")?;
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let contents = fs::read_to_string(&cargo_toml)
                .with_context(|| format!("Failed to read '{}'", cargo_toml.display()))?;
            if contents.contains("[workspace]") && dir.join("rustorio").join("Cargo.toml").exists()
            {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            bail!(
                "Could not find the rustorio repo root. Make sure you are running this command from inside the repo."
            );
        }
    }
}

fn current_timestamp() -> String {
    Local::now().format("%Y%m%d-%H%M%S").to_string()
}

fn tee_child_output(mut child: Child, log_path: &Path) -> Result<ExitStatus> {
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    let log = Arc::new(Mutex::new(log_file));

    let stdout = child.stdout.take().context("stdout was not piped")?;
    let stderr = child.stderr.take().context("stderr was not piped")?;

    let log_for_stdout = Arc::clone(&log);
    let stdout_thread = thread::spawn(move || -> Result<()> {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line.context("failed to read stdout line")?;
            println!("{line}"); // terminal: stdout channel
            let mut log = log_for_stdout.lock()
            .expect("other thread panicked while holding log lock. 
            This should be impossible since it can only panic if another thread panicked while holding the lock");
            writeln!(log, "{line}").context("failed to write to log")?;
        }
        Ok(())
    });

    let log_for_stderr = Arc::clone(&log);
    let stderr_thread = thread::spawn(move || -> Result<()> {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let line = line.context("failed to read stderr line")?;
            eprintln!("{line}"); // terminal: stderr channel
            let mut log = log_for_stderr.lock()
            .expect("other thread panicked while holding log lock. 
            This should be impossible since it can only panic if another thread panicked while holding the lock");
            writeln!(log, "{line}").context("failed to write to log")?;
        }
        Ok(())
    });

    stdout_thread
        .join()
        .expect("stdout thread panicked")
        .context("stdout thread failed")?;
    stderr_thread
        .join()
        .expect("stderr thread panicked")
        .context("stderr thread failed")?;

    child.wait().context("failed to wait for child")
}

#[derive(Subcommand)]
pub enum DevCommands {
    /// Set up and run an AI player for the given game mode.
    /// Locates the repo root by walking up the directory tree, then creates a
    /// timestamped run directory under ai-player/, sets up a Rustorio project,
    /// creates a save game, builds, generates docs, and launches Claude with
    /// sandboxed permissions. Logs to ai-player/<run-dir>/ai.log.
    AiTest(AiTestArgs),
}

impl DevCommands {
    pub fn run(&self) -> Result<()> {
        match self {
            DevCommands::AiTest(args) => args.run(),
        }
    }
}

#[derive(Args)]
pub struct AiTestArgs {
    /// The game mode to test.
    #[clap(value_enum)]
    game_mode: GameMode,
}

impl AiTestArgs {
    pub fn run(&self) -> Result<()> {
        let repo_root = find_repo_root()?;
        let ai_player_dir = repo_root.join("ai-player");
        let rustorio_crate = repo_root.join("rustorio");

        let game_mode_str = self.game_mode.as_str();
        let dir_name = format!("{game_mode_str}-{}", current_timestamp());
        let run_dir = ai_player_dir.join(&dir_name);
        fs::create_dir_all(&run_dir)
            .with_context(|| format!("Failed to create run directory '{}'", run_dir.display()))?;

        let ai_workspace_toml = ai_player_dir.join("Cargo.toml");
        let ai_workspace_toml_snapshot = fs::read_to_string(&ai_workspace_toml)
            .with_context(|| format!("Failed to read '{}'", ai_workspace_toml.display()))?;

        SetupArgs::new(run_dir.clone(), true, dir_name.clone())
            .run()
            .context("Failed to set up Rustorio project")?;

        // cargo new may register the new crate in ai-player/Cargo.toml; restore it.
        fs::write(&ai_workspace_toml, &ai_workspace_toml_snapshot)
            .with_context(|| format!("Failed to restore '{}'", ai_workspace_toml.display()))?;

        let project_dir = run_dir.join("rustorio");

        let ai_crate_toml_path = project_dir.join("Cargo.toml");
        let mut ai_crate_toml_file = OpenOptions::new()
            .append(true)
            .open(ai_crate_toml_path.as_path())
            .with_context(|| {
                format!(
                    "Failed to open the AI crate manifest file at {}",
                    ai_crate_toml_path.display()
                )
            })?;
        writeln!(ai_crate_toml_file, "[workspace]").with_context(|| {
            format!(
                "Failed to add an empty workspace setting to {}",
                ai_crate_toml_path.display()
            )
        })?;

        let ai_crate_gitignore_path = run_dir.join(".gitignore");
        fs::write(&ai_crate_gitignore_path, "*").with_context(|| {
            format!(
                "Failed to write .gitignore file for AI crate at {}",
                ai_crate_gitignore_path.display()
            )
        })?;

        NewGameArgs {
            name: Some(game_mode_str.to_string()),
            game_mode: self.game_mode.clone(),
            directory: project_dir.clone(),
        }
        .run()
        .context("Failed to create new game")?;

        // Set default_save_game in rustorio.toml
        let mut config = Config::load(&project_dir).context("Failed to load config")?;
        config.default_save_game = Some(game_mode_str.to_string());
        config.save(&project_dir).context("Failed to save config")?;

        // Swap published rustorio dep for the local crate
        Command::new(env!("CARGO"))
            .args(["remove", "rustorio"])
            .current_dir(&project_dir)
            .run()
            .context("Failed to remove published rustorio dependency")?;
        Command::new(env!("CARGO"))
            .arg("add")
            .arg("--path")
            .arg(&rustorio_crate)
            .current_dir(&project_dir)
            .run()
            .context("Failed to add local rustorio dependency")?;

        Command::new(env!("CARGO"))
            .arg("build")
            .current_dir(&project_dir)
            .run()
            .context("Failed to build project")?;

        Command::new(env!("CARGO"))
            .args(["doc", "-p", "rustorio", "--no-deps"])
            .current_dir(&project_dir)
            .run()
            .context("Failed to generate docs")?;

        let log_path = run_dir.join("ai.log");
        let settings_path = ai_player_dir.join(".claude").join("settings.json");
        let claude_md_path = ai_player_dir.join("CLAUDE.md");

        let claude = Command::new("claude")
            .args([
                "--print",
                "--setting-sources",
                "project",
                "--strict-mcp-config",
                "--settings",
                settings_path
                    .to_str()
                    .context("settings path is not valid UTF-8")?,
                "--append-system-prompt-file",
                claude_md_path
                    .to_str()
                    .context("CLAUDE.md path is not valid UTF-8")?,
                "--permission-mode",
                "dontAsk",
                "Begin!",
            ])
            .current_dir(&project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to launch Claude")?;

        let exit_status =
            tee_child_output(claude, &log_path).context("Failed to capture Claude output")?;

        if !exit_status.success() {
            bail!("Claude process exited with status {exit_status}");
        }

        Ok(())
    }
}
