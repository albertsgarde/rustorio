use anyhow::Context;
use clap::Parser;

use crate::App;

static DB_PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();

#[derive(Parser)]
pub struct Args {
    /// Path to the SQLite database file
    #[arg(long, default_value = "rustorio.db")]
    pub db_path: String,
}

pub fn init() {
    let args = Args::parse();
    set_db_path(args.db_path);
    init_db().expect("Failed to initialize database");
    dioxus::launch(App);
}

pub fn set_db_path(path: String) {
    DB_PATH.set(path).expect("DB_PATH already set");
}

pub fn get_db() -> anyhow::Result<rusqlite::Connection> {
    let path = DB_PATH.get().context("DB_PATH not initialized")?;
    rusqlite::Connection::open(path).context("failed to open database connection")
}

pub fn init_db() -> anyhow::Result<()> {
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
