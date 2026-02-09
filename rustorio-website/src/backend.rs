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
pub async fn get_leaderboard() -> Result<Vec<LeaderboardEntry>, ServerFnError> {
    let entries: Vec<LeaderboardEntry> = sqlx::query_as(
        "SELECT name, MIN(tick_count) as ticks
         FROM runs
         GROUP BY name
         ORDER BY ticks ASC",
    )
    .fetch_all(server::db())
    .await
    .map_err(|e| ServerFnError::new(format!("{e:#}")))?;

    Ok(entries)
}
