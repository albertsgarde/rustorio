use clap::Parser;
use sqlx::SqlitePool;

use crate::{App, backend::api};

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

    dioxus::serve(|| async move {
        let router =
            dioxus::server::router(App).nest(rustorio_common::BASE_API_PATH, api::router());

        Ok(router)
    })
}

pub fn db() -> &'static SqlitePool {
    DB.get().expect("DB not initialized")
}

async fn init_db(pool: &SqlitePool) -> sqlx::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS runs (
            run_id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            gamemode TEXT NOT NULL,
            tick_count INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    // Insert sample data if table is empty
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM runs")
        .fetch_one(pool)
        .await?;

    if count.0 == 0 {
        sqlx::query(
            "INSERT INTO runs (name, gamemode, tick_count) VALUES
                ('SpeedRunner42', 'tutorial', 12543),
                ('SpeedRunner42', 'tutorial', 14000),
                ('FactorioMaster', 'tutorial', 15221),
                ('FactorioMaster', 'tutorial', 18000),
                ('OptimalPath', 'tutorial', 18902),
                ('RocketScience', 'tutorial', 21445),
                ('NewPlayer', 'tutorial', 45678)",
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
