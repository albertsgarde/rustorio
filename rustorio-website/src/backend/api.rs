use dioxus::server::axum::{
    Extension, Json, Router, http::StatusCode, response::IntoResponse, routing::post,
};
use rustorio_common::SubmitRunRequest;
use sqlx::SqlitePool;

use crate::backend::server;

pub fn router() -> Router {
    Router::new()
        .route("/runs", post(submit_run))
        .route("/reset-db", post(reset_db))
}

async fn submit_run(
    Extension(db): Extension<SqlitePool>,
    Json(body): Json<SubmitRunRequest>,
) -> impl IntoResponse {
    match sqlx::query("INSERT INTO runs (name, gamemode, tick_count) VALUES ($1, $2, $3)")
        .bind(&body.name)
        .bind(body.gamemode)
        .bind(body.tick_count as i64)
        .execute(&db)
        .await
    {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => {
            println!("{e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:#}")).into_response()
        }
    }
}

async fn reset_db(Extension(db): Extension<SqlitePool>) -> impl IntoResponse {
    match server::reset_db(&db).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            println!("{e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:#}")).into_response()
        }
    }
}
