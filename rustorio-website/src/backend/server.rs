use clap::Parser;
use sqlx::SqlitePool;

use crate::{App, backend::api};

#[derive(Parser)]
pub struct Args {
    /// Path to the SQLite database file
    #[arg(long, default_value = "rustorio.db")]
    pub db_path: String,
}

pub fn init() {
    let args = Args::parse();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let pool = rt.block_on(async {
        let pool = SqlitePool::connect(&format!("sqlite:{}?mode=rwc", args.db_path))
            .await
            .expect("Failed to connect to database");
        init_db(&pool).await.expect("Failed to initialize database");
        pool
    });

    dioxus::serve(move || {
        let pool = pool.clone();
        async move {
            let router = dioxus::server::router(App)
                .nest(rustorio_common::BASE_API_PATH, api::router())
                .layer(dioxus::server::axum::Extension(pool));

            Ok(router)
        }
    })
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

    Ok(())
}
