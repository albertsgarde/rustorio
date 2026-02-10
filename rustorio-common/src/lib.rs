pub mod cli;

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const BASE_API_PATH: &str = "/api/v1";

#[derive(Clone, Serialize, Deserialize)]
pub struct SubmitRunRequest {
    pub name: String,
    pub gamemode: String,
    pub tick_count: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Username(String);

#[derive(Clone, Debug, Error)]
pub struct InvalidUsernameError {
    username: String,
}

impl Display for InvalidUsernameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Username '{}' not valid. A username must not contain control characters",
            self.username
        )
    }
}

impl Username {
    pub fn try_from_str(str: impl AsRef<str>) -> Result<Self, InvalidUsernameError> {
        let username = str.as_ref().to_owned();
        if username.chars().any(|c| c.is_control()) {
            Err(InvalidUsernameError { username })
        } else {
            Ok(Username(username))
        }
    }
}

impl FromStr for Username {
    type Err = InvalidUsernameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let username = s.to_owned();
        if username.chars().any(|c| c.is_control()) {
            Err(InvalidUsernameError { username })
        } else {
            Ok(Username(username))
        }
    }
}

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
