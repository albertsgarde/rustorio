use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use anyhow::Context;
#[cfg(feature = "server")]
use clap::Parser;

#[cfg(feature = "server")]
static DB_PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();

#[cfg(feature = "server")]
#[derive(Parser)]
struct Args {
    /// Path to the SQLite database file
    #[arg(long, default_value = "rustorio.db")]
    db_path: String,
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    #[cfg(feature = "server")]
    {
        let args = Args::parse();
        DB_PATH.set(args.db_path).expect("DB_PATH already set");
        init_db().expect("Failed to initialize database");
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

/// Represents a player's score on the leaderboard
#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct LeaderboardEntry {
    name: String,
    ticks: u64,
}

#[cfg(feature = "server")]
fn get_db() -> anyhow::Result<rusqlite::Connection> {
    let path = DB_PATH.get().context("DB_PATH not initialized")?;
    rusqlite::Connection::open(path).context("failed to open database connection")
}

#[cfg(feature = "server")]
fn init_db() -> anyhow::Result<()> {
    let conn = get_db().context("failed to get database connection")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        )",
        [],
    )
    .context("failed to create users table")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS runs (
            run_id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            tick_count INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )",
        [],
    )
    .context("failed to create runs table")?;

    // Insert sample data if tables are empty
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
        .context("failed to count users")?;
    if count == 0 {
        conn.execute(
            "INSERT INTO users (name) VALUES
                ('SpeedRunner42'),
                ('FactorioMaster'),
                ('OptimalPath'),
                ('RocketScience'),
                ('NewPlayer')",
            [],
        )
        .context("failed to insert sample users")?;

        conn.execute(
            "INSERT INTO runs (user_id, tick_count) VALUES
                (1, 12543),
                (1, 14000),
                (2, 15221),
                (2, 18000),
                (3, 18902),
                (4, 21445),
                (5, 45678)",
            [],
        )
        .context("failed to insert sample runs")?;
    }

    Ok(())
}

#[server]
async fn get_leaderboard() -> Result<Vec<LeaderboardEntry>, ServerFnError> {
    fn inner() -> anyhow::Result<Vec<LeaderboardEntry>> {
        let conn = get_db().context("failed to get database connection")?;

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

/// Home page
#[component]
fn Home() -> Element {
    let entries = use_server_future(get_leaderboard)?;

    rsx! {
        div { class: "container mx-auto p-4",
            h1 { class: "text-2xl font-bold mb-4", "Leaderboard" }
            match entries() {
                Some(Ok(entries)) => rsx! {
                    table { class: "w-full border-collapse",
                        thead {
                            tr { class: "border-b",
                                th { class: "text-left p-2", "Player" }
                                th { class: "text-right p-2", "Ticks" }
                            }
                        }
                        tbody {
                            for (i , entry) in entries.iter().enumerate() {
                                tr { class: "border-b hover:bg-gray-100",
                                    td { class: "p-2", "{i + 1}. {entry.name}" }
                                    td { class: "text-right p-2", "{entry.ticks}" }
                                }
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    p { class: "text-red-500", "Error loading leaderboard: {e}" }
                },
                None => rsx! {
                    p { "Loading..." }
                },
            }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div { id: "navbar" }

        Outlet::<Route> {}
    }
}
