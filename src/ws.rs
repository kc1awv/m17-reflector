use crate::reflector::Reflector;
use axum::extract::{
    ws::{WebSocketUpgrade, WebSocket, Message},
    State,
};
use axum::response::IntoResponse;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<Mutex<Reflector>>>,
    rx: broadcast::Receiver<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, rx))
}

async fn handle_socket(
    mut socket: WebSocket,
    state: Arc<Mutex<Reflector>>,
    mut rx: broadcast::Receiver<String>,
) {
    if send_snapshot(&mut socket, &state).await.is_err() {
        return;
    }

    while let Ok(_) = rx.recv().await {
        if send_snapshot(&mut socket, &state).await.is_err() {
            break;
        }
    }
}

async fn send_snapshot(socket: &mut WebSocket, state: &Arc<Mutex<Reflector>>) -> Result<(), ()> {
    let snapshot = {
        let guard = state.lock().await;
        guard.export_state().snapshot()
    };

    let text = serde_json::to_string(&snapshot).unwrap();
    socket.send(Message::Text(text.into())).await.map_err(|_| ())
}
