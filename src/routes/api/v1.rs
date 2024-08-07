use axum::{routing::Route, Router};

use crate::app_state::AppState;

use super::health;

pub fn get_routes() -> Router<AppState> {
    Router::new().merge(health::get_routes())
}
