use crate::server::run_server::AppState;
use axum::{extract::State, response::IntoResponse};

pub async fn items(State(app_state): State<AppState>) -> impl IntoResponse {}
