use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use primordium_io::storage::StorageManager;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

// Re-use the shared network protocol from the main library
use primordium_net::{NetMessage, PeerInfo, TradeProposal};

/// Server state tracking connected peers and their info
struct AppState {
    /// Broadcast channel for room-wide messages
    tx: broadcast::Sender<String>,
    /// Connected peers with their metadata
    peers: Arc<Mutex<HashMap<Uuid, PeerInfo>>>,
    /// Total migrations processed by server
    total_migrations: Arc<Mutex<usize>>,
    active_trades: Arc<Mutex<HashMap<Uuid, Arc<TradeProposal>>>>,
    /// Persistent storage for Hall of Fame and marketplace
    storage: StorageManager,
    /// API key for write endpoints (None = open mode)
    api_key: Option<String>,
}
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "server=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    let (tx, _rx) = broadcast::channel::<String>(100);
    let storage: StorageManager = match StorageManager::new("./registry.db") {
        Ok(s) => {
            tracing::info!("Initialized persistent storage: registry.db");
            s
        }
        Err(e) => {
            tracing::error!("Failed to initialize storage: {}", e);
            std::process::exit(1);
        }
    };

    let api_key = std::env::var("PRIMORDIUM_API_KEY")
        .ok()
        .filter(|k| !k.is_empty());
    if api_key.is_some() {
        tracing::info!("API key authentication enabled for write endpoints");
    } else {
        tracing::warn!(
            "No PRIMORDIUM_API_KEY set — write endpoints are open (unsafe for production)"
        );
    }

    let app_state = Arc::new(AppState {
        tx,
        peers: Arc::new(Mutex::new(HashMap::new())),
        total_migrations: Arc::new(Mutex::new(0)),
        active_trades: Arc::new(Mutex::new(HashMap::new())),
        storage,
        api_key,
    });

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/api/peers", get(get_peers))
        .route("/api/stats", get(get_stats))
        .route("/api/registry/hall_of_fame", get(get_hall_of_fame))
        .route(
            "/api/registry/genomes",
            get(get_genomes).post(submit_genome),
        )
        .route("/api/registry/seeds", get(get_seeds).post(submit_seed))
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

/// REST endpoint: Get Hall of Fame (Global Registry)
async fn get_hall_of_fame(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let rx = match state.storage.query_hall_of_fame_async() {
        Some(r) => r,
        None => {
            return Json(serde_json::json!({
                "error": "failed to query storage"
            }))
            .into_response();
        }
    };

    match rx.recv() {
        Ok(hall_of_fame) => Json(serde_json::json!({
            "hall_of_fame": hall_of_fame.iter().map(|(id, civ_level, is_extinct)| serde_json::json!({
                "id": id.to_string(),
                "civilization_level": civ_level,
                "is_extinct": is_extinct
            })).collect::<Vec<_>>()
        })).into_response(),
        Err(e) => Json(serde_json::json!({
            "error": format!("failed to receive hall of fame: {}", e)
        })).into_response(),
    }
}

/// REST endpoint: Get genomes from marketplace
async fn get_genomes(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let rx = match state
        .storage
        .query_genomes_async(Some(100), Some("fitness".to_string()))
    {
        Some(r) => r,
        None => {
            return Json(serde_json::json!({
                "error": "failed to query genomes"
            }))
            .into_response();
        }
    };

    match rx.recv() {
        Ok(genomes) => Json(serde_json::json!({
            "genomes": genomes
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "error": format!("failed to receive genomes: {}", e)
        }))
        .into_response(),
    }
}
/// Validate API key from Authorization header.
/// Returns None if auth passes, Some(Response) with 401 if it fails.
/// When no api_key is configured (open mode), all requests are allowed.
fn check_auth(state: &AppState, headers: &HeaderMap) -> Option<axum::response::Response> {
    let expected = state.api_key.as_ref()?;

    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let token = auth_header
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.strip_prefix("bearer "));

    match token {
        Some(t) if t == expected => None,
        _ => {
            tracing::warn!("Rejected write request: invalid or missing API key");
            Some(
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({ "error": "invalid or missing API key" })),
                )
                    .into_response(),
            )
        }
    }
}

/// REST endpoint: Submit genome to marketplace
async fn submit_genome(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(resp) = check_auth(&state, &headers) {
        return resp;
    }
    let id = Uuid::new_v4();
    let genotype = payload
        .get("genotype")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let author = payload
        .get("author")
        .and_then(|v| v.as_str())
        .unwrap_or("anonymous")
        .to_string();
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed Genome")
        .to_string();
    let description = payload
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let tags = payload
        .get("tags")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let fitness_score = payload
        .get("fitness_score")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let offspring_count = payload
        .get("offspring_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let tick = payload.get("tick").and_then(|v| v.as_u64()).unwrap_or(0);

    state.storage.submit_genome(
        id,
        None,
        genotype,
        author,
        name,
        description,
        tags,
        fitness_score,
        offspring_count,
        tick,
    );

    Json(serde_json::json!({
        "success": true,
        "id": id.to_string()
    }))
    .into_response()
}

/// REST endpoint: Get seeds from marketplace
async fn get_seeds(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let rx = match state
        .storage
        .query_seeds_async(Some(100), Some("pop".to_string()))
    {
        Some(r) => r,
        None => {
            return Json(serde_json::json!({
                "error": "failed to query seeds"
            }))
            .into_response();
        }
    };

    match rx.recv() {
        Ok(seeds) => Json(serde_json::json!({
            "seeds": seeds
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "error": format!("failed to receive seeds: {}", e)
        }))
        .into_response(),
    }
}
/// REST endpoint: Submit seed to marketplace
async fn submit_seed(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(resp) = check_auth(&state, &headers) {
        return resp;
    }
    let id = Uuid::new_v4();
    let author = payload
        .get("author")
        .and_then(|v| v.as_str())
        .unwrap_or("anonymous")
        .to_string();
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed Seed")
        .to_string();
    let description = payload
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let tags = payload
        .get("tags")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let config_json = payload
        .get("config")
        .and_then(|v| v.as_str())
        .unwrap_or("{}")
        .to_string();
    let avg_tick_time = payload
        .get("avg_tick_time")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let max_pop = payload.get("max_pop").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let performance_summary = payload
        .get("performance_summary")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    state.storage.submit_seed(
        id,
        author,
        name,
        description,
        tags,
        config_json,
        avg_tick_time,
        max_pop,
        performance_summary,
    );

    Json(serde_json::json!({
        "success": true,
        "id": id.to_string()
    }))
    .into_response()
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
                            trades.insert(proposal.id, Arc::new(proposal));
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    fn create_app() -> Router {
        let (tx, _rx) = broadcast::channel::<String>(100);
        let storage = StorageManager::new(":memory:").unwrap_or_else(|e| {
            eprintln!("Failed to create in-memory storage: {}", e);
            std::process::exit(1);
        });
        let app_state = Arc::new(AppState {
            tx,
            peers: Arc::new(Mutex::new(HashMap::new())),
            total_migrations: Arc::new(Mutex::new(0)),
            active_trades: Arc::new(Mutex::new(HashMap::new())),
            storage,
            api_key: None,
        });
        Router::new()
            .route("/api/peers", get(get_peers))
            .route("/api/stats", get(get_stats))
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_get_peers_empty() {
        let app = create_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/peers")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"[]");
    }

    #[tokio::test]
    async fn test_get_stats() {
        let app = create_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/stats")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let stats: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(stats["online_count"], 0);
        assert_eq!(stats["total_migrations"], 0);
    }

    fn create_app_with_auth(key: &str) -> Router {
        let (tx, _rx) = broadcast::channel::<String>(100);
        let storage = StorageManager::new(":memory:").unwrap_or_else(|e| {
            eprintln!("Failed to create in-memory storage: {}", e);
            std::process::exit(1);
        });
        let app_state = Arc::new(AppState {
            tx,
            peers: Arc::new(Mutex::new(HashMap::new())),
            total_migrations: Arc::new(Mutex::new(0)),
            active_trades: Arc::new(Mutex::new(HashMap::new())),
            storage,
            api_key: Some(key.to_string()),
        });
        Router::new()
            .route(
                "/api/registry/genomes",
                get(get_genomes).post(submit_genome),
            )
            .route("/api/registry/seeds", get(get_seeds).post(submit_seed))
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_submit_genome_rejected_without_key() {
        let app = create_app_with_auth("test-secret-key");
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/registry/genomes")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"genotype":"AABB"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_submit_genome_accepted_with_valid_key() {
        let app = create_app_with_auth("test-secret-key");
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/registry/genomes")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer test-secret-key")
                    .body(axum::body::Body::from(r#"{"genotype":"AABB"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
    }

    #[tokio::test]
    async fn test_submit_genome_rejected_with_wrong_key() {
        let app = create_app_with_auth("test-secret-key");
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/registry/genomes")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer wrong-key")
                    .body(axum::body::Body::from(r#"{"genotype":"AABB"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_submit_seed_rejected_without_key() {
        let app = create_app_with_auth("seed-key");
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/registry/seeds")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"name":"test"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_genomes_open_without_auth() {
        let app = create_app_with_auth("any-key");
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/registry/genomes")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // GET endpoints remain public even when auth is configured
        assert_eq!(response.status(), StatusCode::OK);
    }
}
