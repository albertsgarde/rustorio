use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use rustorio_common::Username;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub username: Option<Username>,
    pub rustorio_url: String,
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
        }
    }
}
