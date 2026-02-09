#[cfg(feature = "server")]
pub mod server;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents a player's score on the leaderboard
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub name: String,
    pub ticks: u64,
}

#[server]
pub async fn get_leaderboard() -> Result<Vec<LeaderboardEntry>, ServerFnError> {
    use anyhow::Context;

    fn inner() -> anyhow::Result<Vec<LeaderboardEntry>> {
        let conn = server::get_db().context("failed to get database connection")?;

        let mut stmt = conn
            .prepare(
                "SELECT users.name, MIN(runs.tick_count) as best_ticks
                 FROM runs
                 JOIN users ON runs.user_id = users.id
                 GROUP BY users.id
                 ORDER BY best_ticks ASC",
            )
            .context("failed to prepare leaderboard query")?;

        let entries = stmt
            .query_map([], |row| {
                Ok(LeaderboardEntry {
                    name: row.get(0)?,
                    ticks: u64::try_from(row.get::<_, i64>(1)?)
                        .expect("Ticks should never surpass i64::MAX"),
                })
            })
            .context("failed to execute leaderboard query")?
            .collect::<Result<Vec<_>, _>>()
            .context("failed to collect leaderboard entries")?;

        Ok(entries)
    }

    inner().map_err(|e| ServerFnError::new(format!("{e:#}")))
}
