use crate::model::infra::network::{NetMessage, NetworkState};
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

#[cfg(not(target_arch = "wasm32"))]
use futures::StreamExt;
#[cfg(not(target_arch = "wasm32"))]
use futures_util::sink::SinkExt;
#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Clone)]
pub struct NetworkManager {
    #[cfg(target_arch = "wasm32")]
    ws: Option<WebSocket>,
    #[cfg(not(target_arch = "wasm32"))]
    tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,

    pending_migrations: Arc<Mutex<Vec<NetMessage>>>,
    state: Arc<Mutex<NetworkState>>,
}

impl NetworkManager {
    #[cfg(target_arch = "wasm32")]
    pub fn new(url: &str) -> Self {
        let pending = Arc::new(Mutex::new(Vec::new()));
        let state = Arc::new(Mutex::new(NetworkState::default()));

        let ws = match WebSocket::new(url) {
            Ok(ws) => {
                let pending_clone = pending.clone();
                let state_clone = state.clone();

                let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                    if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                        let txt: String = txt.into();
                        if let Ok(msg) = serde_json::from_str::<NetMessage>(&txt) {
                            Self::handle_incoming_message(&state_clone, &pending_clone, msg);
                        }
                    }
                })
                    as Box<dyn FnMut(MessageEvent)>);

                ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                onmessage_callback.forget();

                let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                    web_sys::console::error_1(&e);
                })
                    as Box<dyn FnMut(ErrorEvent)>);
                ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                onerror_callback.forget();

                Some(ws)
            }
            Err(_) => None,
        };

        Self {
            ws,
            pending_migrations: pending,
            state,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(url: &str) -> Self {
        let pending = Arc::new(Mutex::new(Vec::new()));
        let state = Arc::new(Mutex::new(NetworkState::default()));
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        let pending_clone = pending.clone();
        let state_clone = state.clone();
        let url_string = url.to_string();

        tokio::spawn(async move {
            let (ws_stream, _) = match connect_async(&url_string).await {
                Ok(v) => v,
                Err(_) => return,
            };
            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            loop {
                tokio::select! {
                    Some(msg) = rx.recv() => {
                        if ws_sender.send(Message::Text(msg)).await.is_err() { break; }
                    }
                Some(Ok(msg)) = ws_receiver.next() => {
                    if let Message::Text(txt) = msg {
                        // Security: Limit message size to prevent DoS (100KB max)
                        const MAX_MESSAGE_SIZE: usize = 100 * 1024;
                        if txt.len() > MAX_MESSAGE_SIZE {
                            #[cfg(target_arch = "wasm32")]
                            web_sys::console::warn_1(&format!("Oversized message received: {} bytes", txt.len()).into());
                            #[cfg(not(target_arch = "wasm32"))]
                            eprintln!("Warning: Oversized message received: {} bytes", txt.len());
                            continue;
                        }

                        if let Ok(net_msg) = serde_json::from_str::<NetMessage>(&txt) {
                            Self::handle_incoming_message(&state_clone, &pending_clone, net_msg);
                        }
                    }
                }
                    else => break,
                }
            }
        });

        Self {
            tx: Some(tx),
            pending_migrations: pending,
            state,
        }
    }

    fn handle_incoming_message(
        state: &Arc<Mutex<NetworkState>>,
        pending: &Arc<Mutex<Vec<NetMessage>>>,
        msg: NetMessage,
    ) {
        let mut s = match state.lock() {
            Ok(s) => s,
            Err(e) => {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::error_1(&format!("Failed to lock state mutex: {}", e).into());
                #[cfg(not(target_arch = "wasm32"))]
                eprintln!("Failed to lock state mutex: {}", e);
                return;
            }
        };
        match msg {
            NetMessage::MigrateEntity { .. } => {
                s.migrations_received += 1;
                if let Ok(mut p) = pending.lock() {
                    p.push(msg);
                }
            }
            NetMessage::Handshake { client_id } => {
                s.client_id = Some(client_id);
            }
            NetMessage::PeerList { peers } => {
                s.peers = peers;
            }
            NetMessage::TradeOffer(proposal) => {
                s.trade_offers.push(proposal);
            }
            NetMessage::TradeAccept { proposal_id, .. } => {
                s.trade_offers.retain(|o| o.id != proposal_id);
            }
            NetMessage::TradeRevoke { proposal_id } => {
                s.trade_offers.retain(|o| o.id != proposal_id);
            }
            NetMessage::Relief { .. } => {
                if let Ok(mut p) = pending.lock() {
                    p.push(msg);
                }
            }
            _ => {}
        }
    }

    pub fn send(&self, msg: &NetMessage) {
        let txt = match serde_json::to_string(msg) {
            Ok(t) => t,
            Err(e) => {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::error_1(&format!("Failed to serialize message: {}", e).into());
                #[cfg(not(target_arch = "wasm32"))]
                eprintln!("Failed to serialize message: {}", e);
                return;
            }
        };

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(ws) = &self.ws {
                if ws.ready_state() == 1 {
                    let _ = ws.send_with_str(&txt);
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(tx) = &self.tx {
                let _ = tx.send(txt);
            }
        }

        if matches!(msg, NetMessage::MigrateEntity { .. }) {
            if let Ok(mut s) = self.state.lock() {
                s.migrations_sent += 1;
            }
        }
    }

    pub fn announce(&self, entity_count: usize) {
        let (migrations_sent, migrations_received) = if let Ok(s) = self.state.lock() {
            (s.migrations_sent, s.migrations_received)
        } else {
            (0, 0)
        };
        let msg = NetMessage::PeerAnnounce {
            entity_count,
            migrations_sent,
            migrations_received,
        };
        self.send(&msg);
    }

    pub fn pop_pending_limited(&self, limit: usize) -> Vec<NetMessage> {
        match self.pending_migrations.lock() {
            Ok(mut p) => {
                let count = p.len().min(limit);
                p.drain(..count).collect()
            }
            Err(_) => vec![],
        }
    }

    pub fn get_state(&self) -> NetworkState {
        self.state.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_handle_incoming_handshake() {
        let state = Arc::new(Mutex::new(NetworkState::default()));
        let pending = Arc::new(Mutex::new(Vec::new()));
        let client_id = Uuid::new_v4();

        NetworkManager::handle_incoming_message(
            &state,
            &pending,
            NetMessage::Handshake { client_id },
        );

        let s = state.lock().unwrap();
        assert_eq!(s.client_id, Some(client_id));
    }

    #[test]
    fn test_handle_incoming_peer_list() {
        let state = Arc::new(Mutex::new(NetworkState::default()));
        let pending = Arc::new(Mutex::new(Vec::new()));
        let mut peers = Vec::new();
        let peer_id = Uuid::new_v4();
        peers.push(crate::model::infra::network::PeerInfo {
            peer_id,
            entity_count: 10,
            migrations_sent: 0,
            migrations_received: 0,
        });

        NetworkManager::handle_incoming_message(
            &state,
            &pending,
            NetMessage::PeerList {
                peers: peers.clone(),
            },
        );

        let s = state.lock().unwrap();
        assert_eq!(s.peers.len(), 1);
        assert_eq!(s.peers[0].entity_count, 10);
    }

    #[test]
    fn test_handle_incoming_migrate_entity() {
        let state = Arc::new(Mutex::new(NetworkState::default()));
        let pending = Arc::new(Mutex::new(Vec::new()));

        let msg = NetMessage::MigrateEntity {
            migration_id: Uuid::new_v4(),
            dna: "DNA".to_string(),
            energy: 100.0,
            generation: 1,
            species_name: "Name".to_string(),
            fingerprint: "Fingerprint".to_string(),
            checksum: "Checksum".to_string(),
        };

        NetworkManager::handle_incoming_message(&state, &pending, msg.clone());

        let s = state.lock().unwrap();
        assert_eq!(s.migrations_received, 1);

        let p = pending.lock().unwrap();
        assert_eq!(p.len(), 1);
        if let NetMessage::MigrateEntity { dna, .. } = &p[0] {
            assert_eq!(dna, "DNA");
        } else {
            panic!("Wrong message type in pending");
        }
    }
}
