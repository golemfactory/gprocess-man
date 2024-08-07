use axum::{routing::get, Router};

use crate::app_state::AppState;

pub fn get_routes() -> Router<AppState> {
    Router::new().route("/health", get(|| async { "OK" }))
}
