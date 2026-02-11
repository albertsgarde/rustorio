#[cfg(feature = "server")]
mod api;
#[cfg(feature = "server")]
pub mod server;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents a player's score on the leaderboard
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct LeaderboardEntry {
    pub name: String,
    pub ticks: u64,
}

#[server]
pub async fn get_leaderboard(gamemode: String) -> Result<Vec<LeaderboardEntry>, ServerFnError> {
    let entries: Vec<LeaderboardEntry> = sqlx::query_as(
        "SELECT name, MIN(tick_count) as ticks
         FROM runs
         WHERE gamemode = $1
         GROUP BY name
         ORDER BY ticks ASC",
    )
    .bind(gamemode)
    .fetch_all(server::db())
    .await
    .map_err(|e| ServerFnError::new(format!("{e:#}")))?;

    Ok(entries)
}

#[server]
pub async fn get_gamemodes() -> Result<Vec<String>, ServerFnError> {
    sqlx::query_scalar(
        "SELECT DISTINCT gamemode
         FROM runs",
    )
    .fetch_all(server::db())
    .await
    .map_err(|e| ServerFnError::new(format!("{e:#}")))
}
