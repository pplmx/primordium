use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

// Re-use the shared network protocol
#[path = "../model/network.rs"]
mod network;
use network::NetMessage;

struct AppState {
    // We'll use a simple broadcast channel for room-wide messages
    tx: broadcast::Sender<String>,
    // Track active clients (simple count for now)
    client_count: Arc<Mutex<usize>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "server=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (tx, _rx) = broadcast::channel(100);
    let app_state = Arc::new(AppState {
        tx,
        client_count: Arc::new(Mutex::new(0)),
    });

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();
    let client_id = Uuid::new_v4();

    // Increment count
    {
        let mut count = state.client_count.lock().unwrap();
        *count += 1;
        tracing::info!("Client connected: {}. Total: {}", client_id, *count);
    }

    // Send Handshake
    let handshake = NetMessage::Handshake { client_id };
    if let Ok(msg_str) = serde_json::to_string(&handshake) {
        let _ = sender.send(Message::Text(msg_str)).await;
    }

    // Subscribe to broadcast
    let mut rx = state.tx.subscribe();

    // Loop
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In a real app we'd filter messages so we don't send back to self if needed
            // For now, relay everything (simplified)
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx = state.tx.clone();
    let client_count_clone = state.client_count.clone();
    let id_clone = client_id;

    // Receive messages
    while let Some(Ok(result)) = receiver.next().await {
        if let Message::Text(text) = result {
            // Attempt to parse
            if let Ok(NetMessage::MigrateEntity { .. }) = serde_json::from_str::<NetMessage>(&text)
            {
                // Forward migration to ALL other clients (simplified broadcast)
                // Ideally we pick a random target or neighbour
                tracing::info!("Relaying migration from {}", id_clone);
                let _ = tx.send(text); // Broadcast the raw wrapper
            }
        }
    }

    // Cleanup
    send_task.abort();
    {
        let mut count = client_count_clone.lock().unwrap();
        *count = count.saturating_sub(1);
        tracing::info!("Client disconnected: {}. Total: {}", id_clone, *count);
    }
}
