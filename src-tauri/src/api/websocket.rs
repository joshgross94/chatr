use axum::{
    extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
};
use tokio::sync::broadcast;
use tracing::{debug, warn};

use crate::events::AppEvent;
use crate::state::ServiceContext;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(ctx): State<ServiceContext>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, ctx.event_tx.subscribe()))
}

async fn handle_socket(mut socket: WebSocket, mut event_rx: broadcast::Receiver<AppEvent>) {
    debug!("WebSocket client connected");

    loop {
        tokio::select! {
            // Forward AppEvents to the WebSocket client as JSON
            result = event_rx.recv() => {
                match result {
                    Ok(event) => {
                        match serde_json::to_string(&event) {
                            Ok(json) => {
                                if socket.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                warn!("Failed to serialize event: {}", e);
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket client lagged, skipped {} events", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
            // Handle incoming WebSocket messages (for future use, e.g. ping/pong)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(_)) => {} // Ignore other messages for now
                    Some(Err(_)) => break,
                }
            }
        }
    }

    debug!("WebSocket client disconnected");
}
