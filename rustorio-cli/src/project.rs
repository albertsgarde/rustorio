use std::{
    fs,
    io::{self, Read},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, exit},
    thread,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use rustorio_common::{
    Username,
    cli::{PORT_ENV_NAME, PlayOutput},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub username: Option<Username>,
    pub rustorio_url: String,
    pub default_save_game: Option<String>,
}

impl Config {
    fn config_file_path(rustorio_root: &Path) -> PathBuf {
        rustorio_root.join("rustorio.toml")
    }

    pub fn load(rustorio_root: &Path) -> Result<Self> {
        let config_file_path = Self::config_file_path(rustorio_root);
        let config_file_path = config_file_path.as_path();
        let config_string = fs::read_to_string(config_file_path).with_context(|| {
            format!("Failed to read contents of rustorio config file at `{config_file_path:?}`")
        })?;
        toml::from_str(config_string.as_str()).with_context(|| {
            format!("Failed to parse rustorio config file at `{config_file_path:?}")
        })
    }

    pub fn save(&self, rustorio_root: &Path) -> Result<()> {
        let config_file_path = Self::config_file_path(rustorio_root);
        let config_file_path = config_file_path.as_path();
        let toml_string = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(config_file_path, toml_string)
            .with_context(|| format!("Failed to write config to file at `{config_file_path:?}`"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            username: None,
            rustorio_url: "https://rustor.io".to_string(),
            default_save_game: None,
        }
    }
}

pub struct ProjectInfo {
    pub root_path: PathBuf,
    pub config: Config,
}

impl ProjectInfo {
    pub fn get() -> Result<Option<Self>> {
        Self::get_at(Path::new("."))
    }

    pub fn get_at(path: &Path) -> Result<Option<Self>> {
        let mut current_dir = path
            .canonicalize()
            .context("Failed to canonicalize current directory")?;
        let root_path = loop {
            if current_dir.join("rustorio.toml").exists() {
                break current_dir;
            }
            if !current_dir.pop() {
                return Ok(None);
            }
        };
        let config = Config::load(root_path.as_path()).context("Failed to load config")?;
        Ok(Some(Self { root_path, config }))
    }

    pub fn play(&self, save_name: &str) -> Result<PlayOutput> {
        let save_game_path = self.root_path.join("src").join("bin").join(save_name);
        if !save_game_path.exists() || !save_game_path.is_dir() {
            bail!("Save game '{save_name}' does not exist.");
        }

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener
            .set_nonblocking(true)
            .context("Failed to set listener to non-blocking")?;
        let port = listener.local_addr().unwrap().port();

        // Use a raw "cargo" to allow the toolchain file to take effect.
        let mut child_handle = Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg(save_name)
            .env(PORT_ENV_NAME, format!("{port}"))
            .current_dir(self.root_path.as_path())
            .spawn()
            .context("Failed to spawn Rustorio game")?;

        // The child will only send a connection after the game has been won. This loop is to handle all the cases where that doesn't happen, such as the save failing to compile.
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Check if the child process has exited
                    match child_handle.try_wait() {
                        Ok(Some(status)) => {
                            if let Some(code) = status.code() {
                                if code == 0 {
                                    bail!(
                                        "Rustorio process exited successfully before sending output"
                                    );
                                } else {
                                    exit(code);
                                }
                            } else {
                                bail!(
                                    "Rustorio process exited with no status code before sending output"
                                );
                            }
                        }
                        Ok(None) => {
                            thread::sleep(Duration::from_millis(1));
                        }
                        Err(e) => {
                            bail!("Failed to wait for Rustorio process: {e}");
                        }
                    }
                }
                Err(e) => {
                    bail!("Failed to accept connection: {e}");
                }
            }
        };

        let mut output_buffer = String::new();

        stream.read_to_string(&mut output_buffer).unwrap();

        let exit_status = child_handle
            .wait()
            .context("Failed to wait for Rustorio game")?;
        if exit_status.code() != Some(0) {
            bail!(
                "Unexpected status code {:?} from Rustorio process",
                exit_status.code()
            )
        }

        output_buffer.parse().with_context(|| {
            format!("Failed to parse output from Rustorio process. Output: {output_buffer}")
        })
    }
}
