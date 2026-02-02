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
use primordium_lib::model::infra::network::{NetMessage, PeerInfo, TradeProposal};

/// Server state tracking connected peers and their info
struct AppState {
    /// Broadcast channel for room-wide messages
    tx: broadcast::Sender<String>,
    /// Connected peers with their metadata
    peers: Arc<Mutex<HashMap<Uuid, PeerInfo>>>,
    /// Total migrations processed by server
    total_migrations: Arc<Mutex<usize>>,
    active_trades: Arc<Mutex<HashMap<Uuid, TradeProposal>>>,
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
        active_trades: Arc::new(Mutex::new(HashMap::new())),
    });

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/api/peers", get(get_peers))
        .route("/api/stats", get(get_stats))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Primordium Relay Server listening on {}", addr);
    tracing::info!("    WebSocket: ws://{}/ws", addr);
    tracing::info!("    Peers API: http://{}/api/peers", addr);
    tracing::info!("    Stats API: http://{}/api/stats", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind to address {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}

/// REST endpoint: Get list of connected peers
async fn get_peers(State(state): State<Arc<AppState>>) -> Json<Vec<PeerInfo>> {
    match state.peers.lock() {
        Ok(peers) => Json(peers.values().cloned().collect()),
        Err(e) => {
            tracing::error!("Failed to lock peers mutex: {}", e);
            Json(vec![])
        }
    }
}

/// REST endpoint: Get server stats
async fn get_stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let (online_count, peers_data) = match state.peers.lock() {
        Ok(peers) => (peers.len(), peers.values().cloned().collect::<Vec<_>>()),
        Err(e) => {
            tracing::error!("Failed to lock peers mutex: {}", e);
            (0, vec![])
        }
    };

    let total_migrations = match state.total_migrations.lock() {
        Ok(migrations) => *migrations,
        Err(e) => {
            tracing::error!("Failed to lock migrations mutex: {}", e);
            0
        }
    };

    Json(serde_json::json!({
        "online_count": online_count,
        "total_migrations": total_migrations,
        "peers": peers_data
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
        match state.peers.lock() {
            Ok(mut peers) => {
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
            }
            Err(e) => {
                tracing::error!("Failed to lock peers mutex: {}", e);
                None
            }
        }
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
    let active_trades_clone = state.active_trades.clone();
    let id_clone = client_id;

    // Maximum message size: 100KB to prevent DoS
    const MAX_MESSAGE_SIZE: usize = 100 * 1024;

    // Process incoming messages
    while let Some(Ok(result)) = receiver.next().await {
        if let Message::Text(text) = result {
            // Check message size to prevent memory exhaustion
            if text.len() > MAX_MESSAGE_SIZE {
                tracing::warn!(
                    "Client {} sent oversized message: {} bytes (max: {})",
                    client_id,
                    text.len(),
                    MAX_MESSAGE_SIZE
                );
                continue;
            }

            if let Ok(msg) = serde_json::from_str::<NetMessage>(&text) {
                match msg {
                    NetMessage::MigrateEntity { .. } => {
                        // Update migration stats
                        {
                            if let Ok(mut peers) = peers_clone.lock() {
                                if let Some(peer) = peers.get_mut(&id_clone) {
                                    peer.migrations_sent += 1;
                                }
                            } else {
                                tracing::warn!("Failed to lock peers mutex for migration stats");
                            }
                            if let Ok(mut total) = total_migrations_clone.lock() {
                                *total += 1;
                            } else {
                                tracing::warn!("Failed to lock migrations mutex");
                            }
                        }
                        tracing::info!("Relaying migration from {}", id_clone);
                        let _ = tx.send(text);
                    }
                    NetMessage::TradeOffer(proposal) => {
                        if let Ok(mut trades) = active_trades_clone.lock() {
                            trades.insert(proposal.id, proposal.clone());
                        } else {
                            tracing::warn!("Failed to lock trades mutex for trade offer");
                        }
                        tracing::info!("Relaying trade offer from {}", id_clone);
                        let _ = tx.send(text);
                    }
                    NetMessage::TradeAccept { proposal_id, .. } => {
                        let is_valid = if let Ok(mut trades) = active_trades_clone.lock() {
                            trades.remove(&proposal_id).is_some()
                        } else {
                            tracing::warn!("Failed to lock trades mutex for trade acceptance");
                            false
                        };

                        if is_valid {
                            tracing::info!("Relaying valid trade acceptance for {}", proposal_id);
                            let _ = tx.send(text);
                        } else {
                            tracing::warn!("Blocked double-acceptance for trade {}", proposal_id);
                        }
                    }
                    NetMessage::TradeRevoke { proposal_id } => {
                        if let Ok(mut trades) = active_trades_clone.lock() {
                            trades.remove(&proposal_id);
                        } else {
                            tracing::warn!("Failed to lock trades mutex for trade revoke");
                        }
                        let _ = tx.send(text);
                    }
                    NetMessage::MigrateAck { .. } => {
                        tracing::info!("Relaying migration ACK for {}", id_clone);
                        let _ = tx.send(text);
                    }
                    NetMessage::PeerAnnounce {
                        entity_count,
                        migrations_sent,
                        migrations_received,
                    } => {
                        // Update peer info and broadcast
                        let peer_list_msg = if let Ok(mut peers) = peers_clone.lock() {
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
                        } else {
                            tracing::warn!("Failed to lock peers mutex for PeerAnnounce");
                            None
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

    let revoked_ids = if let Ok(mut trades) = state.active_trades.lock() {
        let to_remove: Vec<Uuid> = trades
            .iter()
            .filter(|(_, p)| p.sender_id == client_id)
            .map(|(id, _)| *id)
            .collect();

        for id in &to_remove {
            trades.remove(id);
        }
        to_remove
    } else {
        tracing::warn!("Failed to lock trades mutex during disconnect cleanup");
        vec![]
    };

    for id in revoked_ids {
        let revoke = NetMessage::TradeRevoke { proposal_id: id };
        if let Ok(msg_str) = serde_json::to_string(&revoke) {
            let _ = tx.send(msg_str);
        }
    }

    let disconnect_peer_list_msg = if let Ok(mut peers) = peers_clone.lock() {
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
    } else {
        tracing::warn!("Failed to lock peers mutex during disconnect");
        None
    };
    if let Some(msg_str) = disconnect_peer_list_msg {
        let _ = tx.send(msg_str);
    }
}
