use dioxus::server::axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::post};
use rustorio_common::SubmitRunRequest;

use super::server::db;

pub fn router() -> Router {
    Router::new().route("/runs", post(submit_run))
}

async fn submit_run(Json(body): Json<SubmitRunRequest>) -> impl IntoResponse {
    match sqlx::query("INSERT INTO runs (name, tick_count) VALUES ($1, $2)")
        .bind(&body.name)
        .bind(body.tick_count as i64)
        .execute(db())
        .await
    {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => {
            println!("{e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:#}")).into_response()
        }
    }
}
