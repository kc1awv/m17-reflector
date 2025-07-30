use crate::reflector::Reflector;
use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type SharedState = Arc<Mutex<Reflector>>;

pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/api/v1/stats", get(get_stats))
        .route("/api/v1/clients", get(get_clients))
        .route("/api/v1/modules", get(get_modules))
        .route("/api/v1/streams/active", get(get_active_streams))
        .route("/api/v1/streams/recent", get(get_recent_streams))
        .with_state(state)
}

async fn get_stats(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.lock().await;
    let snapshot = guard.export_state().snapshot();
    Json(snapshot)
}

async fn get_clients(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.lock().await;
    let snapshot = guard.export_state().snapshot();
    Json(snapshot.clients)
}

async fn get_modules(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.lock().await;
    let snapshot = guard.export_state().snapshot();
    Json(snapshot.modules)
}

async fn get_active_streams(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.lock().await;
    let snapshot = guard.export_state().snapshot();
    Json(snapshot.active_streams)
}

async fn get_recent_streams(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.lock().await;
    let snapshot = guard.export_state().snapshot();
    Json(snapshot.recent_streams)
}
