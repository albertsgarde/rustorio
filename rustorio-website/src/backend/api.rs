use dioxus::server::axum::{
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use serde::Deserialize;

use super::server::db;

pub fn router() -> Router {
    Router::new().route("/runs", post(submit_run))
}

#[derive(Deserialize)]
struct SubmitRunRequest {
    name: String,
    tick_count: u64,
}

async fn submit_run(Json(body): Json<SubmitRunRequest>) -> impl IntoResponse {
    match sqlx::query(
        "INSERT INTO runs (name, tick_count) VALUES ($1, $2)",
    )
    .bind(&body.name)
    .bind(body.tick_count as i64)
    .execute(db())
    .await
    {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:#}")).into_response(),
    }
}
