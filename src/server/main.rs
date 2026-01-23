use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

// Re-use the shared network protocol from the main library
use primordium_lib::model::infra::network::{NetMessage, PeerInfo};

/// Server state tracking connected peers and their info
struct AppState {
    /// Broadcast channel for room-wide messages
    tx: broadcast::Sender<String>,
    /// Connected peers with their metadata
    peers: Arc<Mutex<HashMap<Uuid, PeerInfo>>>,
    /// Total migrations processed by server
    total_migrations: Arc<Mutex<usize>>,
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
        peers: Arc::new(Mutex::new(HashMap::new())),
        total_migrations: Arc::new(Mutex::new(0)),
    });

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/api/peers", get(get_peers))
        .route("/api/stats", get(get_stats))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Primordium Relay Server listening on {}", addr);
    tracing::info!("  WebSocket: ws://{}/ws", addr);
    tracing::info!("  Peers API: http://{}/api/peers", addr);
    tracing::info!("  Stats API: http://{}/api/stats", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app).await.expect("Server error");
}

/// REST endpoint: Get list of connected peers
async fn get_peers(State(state): State<Arc<AppState>>) -> Json<Vec<PeerInfo>> {
    let peers = state.peers.lock().expect("Mutex poisoned");
    Json(peers.values().cloned().collect())
}

/// REST endpoint: Get server stats
async fn get_stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let peers = state.peers.lock().expect("Mutex poisoned");
    let total_migrations = *state.total_migrations.lock().expect("Mutex poisoned");
    Json(serde_json::json!({
        "online_count": peers.len(),
        "total_migrations": total_migrations,
        "peers": peers.values().cloned().collect::<Vec<_>>()
    }))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();
    let client_id = Uuid::new_v4();

    // Initialize peer info and get initial peer list message
    let initial_peer_list_msg = {
        let mut peers = state.peers.lock().expect("Mutex poisoned");
        peers.insert(
            client_id,
            PeerInfo {
                peer_id: client_id,
                entity_count: 0,
                migrations_sent: 0,
                migrations_received: 0,
            },
        );
        tracing::info!(
            "Client connected: {}. Total peers: {}",
            client_id,
            peers.len()
        );
        // Create peer list message
        let peer_list = NetMessage::PeerList {
            peers: peers.values().cloned().collect(),
        };
        serde_json::to_string(&peer_list).ok()
    };

    // Send Handshake with client ID
    let handshake = NetMessage::Handshake { client_id };
    if let Ok(msg_str) = serde_json::to_string(&handshake) {
        let _ = sender.send(Message::Text(msg_str)).await;
    }

    // Send initial peer list
    if let Some(peer_list_msg) = initial_peer_list_msg {
        let _ = sender.send(Message::Text(peer_list_msg)).await;
    }

    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();

    // Spawn task to forward broadcasts to this client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx = state.tx.clone();
    let peers_clone = state.peers.clone();
    let total_migrations_clone = state.total_migrations.clone();
    let id_clone = client_id;

    // Process incoming messages
    while let Some(Ok(result)) = receiver.next().await {
        if let Message::Text(text) = result {
            if let Ok(msg) = serde_json::from_str::<NetMessage>(&text) {
                match msg {
                    NetMessage::MigrateEntity { .. } => {
                        // Update migration stats
                        {
                            let mut peers = peers_clone.lock().expect("Mutex poisoned");
                            if let Some(peer) = peers.get_mut(&id_clone) {
                                peer.migrations_sent += 1;
                            }
                            let mut total = total_migrations_clone.lock().expect("Mutex poisoned");
                            *total += 1;
                        }
                        tracing::info!("Relaying migration from {}", id_clone);
                        let _ = tx.send(text);
                    }
                    NetMessage::PeerAnnounce {
                        entity_count,
                        migrations_sent,
                        migrations_received,
                    } => {
                        // Update peer info and broadcast
                        let peer_list_msg = {
                            let mut peers = peers_clone.lock().expect("Mutex poisoned");
                            if let Some(peer) = peers.get_mut(&id_clone) {
                                peer.entity_count = entity_count;
                                peer.migrations_sent = migrations_sent;
                                peer.migrations_received = migrations_received;
                            }
                            tracing::debug!(
                                "Peer {} announced: {} entities",
                                id_clone,
                                entity_count
                            );
                            let peer_list = NetMessage::PeerList {
                                peers: peers.values().cloned().collect(),
                            };
                            serde_json::to_string(&peer_list).ok()
                        };
                        if let Some(msg_str) = peer_list_msg {
                            let _ = tx.send(msg_str);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Cleanup on disconnect
    send_task.abort();
    let disconnect_peer_list_msg = {
        let mut peers = peers_clone.lock().expect("Mutex poisoned");
        peers.remove(&id_clone);
        tracing::info!(
            "Client disconnected: {}. Total peers: {}",
            id_clone,
            peers.len()
        );
        let peer_list = NetMessage::PeerList {
            peers: peers.values().cloned().collect(),
        };
        serde_json::to_string(&peer_list).ok()
    };
    if let Some(msg_str) = disconnect_peer_list_msg {
        let _ = tx.send(msg_str);
    }
}
