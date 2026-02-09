use clap::Parser;
use sqlx::SqlitePool;

use crate::App;

static DB: std::sync::OnceLock<SqlitePool> = std::sync::OnceLock::new();

#[derive(Parser)]
pub struct Args {
    /// Path to the SQLite database file
    #[arg(long, default_value = "rustorio.db")]
    pub db_path: String,
}

pub fn init() {
    let args = Args::parse();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let pool = SqlitePool::connect(&format!("sqlite:{}?mode=rwc", args.db_path))
            .await
            .expect("Failed to connect to database");
        init_db(&pool).await.expect("Failed to initialize database");
        DB.set(pool).expect("DB already set");
    });

    dioxus::launch(App);
}

pub fn db() -> &'static SqlitePool {
    DB.get().expect("DB not initialized")
}

async fn init_db(pool: &SqlitePool) -> sqlx::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS runs (
            run_id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            tick_count INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )",
    )
    .execute(pool)
    .await?;

    // Insert sample data if tables are empty
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    if count.0 == 0 {
        sqlx::query(
            "INSERT INTO users (name) VALUES
                ('SpeedRunner42'),
                ('FactorioMaster'),
                ('OptimalPath'),
                ('RocketScience'),
                ('NewPlayer')",
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "INSERT INTO runs (user_id, tick_count) VALUES
                (1, 12543),
                (1, 14000),
                (2, 15221),
                (2, 18000),
                (3, 18902),
                (4, 21445),
                (5, 45678)",
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
