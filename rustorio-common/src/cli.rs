use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub const PORT_ENV_NAME: &str = "OUTPUT_PORT";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayOutput {
    pub ticks: u64,
    pub gamemode: String,
}

impl PlayOutput {
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl FromStr for PlayOutput {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
