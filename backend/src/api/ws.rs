// WebSocket handler for game state streaming.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};

use super::AppState;

/// WebSocket upgrade handler for game state streaming.
pub async fn ws_game(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut rx = state.game_server.subscribe();

    // Send cached world snapshot so late joiners see the map immediately.
    if let Some(world_json) = state.game_server.world_json() {
        if socket.send(Message::Text(world_json.into())).await.is_err() {
            return;
        }
    }

    // Forward all broadcast messages to the WebSocket client.
    // When the client disconnects or the broadcast channel closes, we stop.
    loop {
        tokio::select! {
            // Game message from broadcast channel
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if socket.send(Message::Text(msg.into())).await.is_err() {
                            // Client disconnected
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        // Channel closed, game ended
                        break;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("WebSocket client lagged, skipped {n} messages");
                        // Continue receiving
                    }
                }
            }
            // Client message (we mostly ignore, but detect disconnect)
            result = socket.recv() => {
                match result {
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    _ => {
                        // Ignore other client messages for now
                    }
                }
            }
        }
    }
}
