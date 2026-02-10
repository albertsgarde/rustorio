use std::{num::ParseIntError, str::FromStr};

pub const PORT_ENV_NAME: &str = "OUTPUT_PORT";

pub struct PlayOutput {
    pub ticks: u64,
}

impl PlayOutput {
    pub fn serialize(&self) -> String {
        format!("{}", self.ticks)
    }
}

impl FromStr for PlayOutput {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PlayOutput { ticks: s.parse()? })
    }
}
