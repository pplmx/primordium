use crate::model::infra::network::{NetMessage, NetworkState, PeerInfo};
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

#[derive(Clone)]
pub struct NetworkManager {
    ws: Option<WebSocket>,
    pending_migrations: Rc<RefCell<Vec<NetMessage>>>,
    state: Rc<RefCell<NetworkState>>,
}

impl NetworkManager {
    pub fn new(url: &str) -> Self {
        let pending = Rc::new(RefCell::new(Vec::new()));
        let state = Rc::new(RefCell::new(NetworkState::default()));

        let ws = match WebSocket::new(url) {
            Ok(ws) => {
                let pending_clone = pending.clone();
                let state_clone = state.clone();

                // OnMessage callback
                let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                    if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                        let txt: String = txt.into();
                        if let Ok(msg) = serde_json::from_str::<NetMessage>(&txt) {
                            match msg {
                                NetMessage::MigrateEntity { .. } => {
                                    state_clone.borrow_mut().migrations_received += 1;
                                    pending_clone.borrow_mut().push(msg);
                                }
                                NetMessage::Handshake { client_id } => {
                                    state_clone.borrow_mut().client_id = Some(client_id);
                                    web_sys::console::log_1(&JsValue::from_str(&format!(
                                        "Connected as peer: {}",
                                        client_id
                                    )));
                                }
                                NetMessage::PeerList { peers } => {
                                    state_clone.borrow_mut().peers = peers;
                                }
                                NetMessage::StatsUpdate { online_count, .. } => {
                                    web_sys::console::log_1(&JsValue::from_str(&format!(
                                        "Network stats: {} peers online",
                                        online_count
                                    )));
                                }
                                _ => {}
                            }
                        }
                    }
                })
                    as Box<dyn FnMut(MessageEvent)>);

                ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                onmessage_callback.forget(); // Keep alive

                // OnError
                let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                    web_sys::console::error_1(&e);
                })
                    as Box<dyn FnMut(ErrorEvent)>);
                ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                onerror_callback.forget();

                Some(ws)
            }
            Err(_) => {
                web_sys::console::error_1(&JsValue::from_str("WebSocket connection failed"));
                None
            }
        };

        Self {
            ws,
            pending_migrations: pending,
            state,
        }
    }

    pub fn send(&self, msg: &NetMessage) {
        if let Some(ws) = &self.ws {
            if ws.ready_state() == WebSocket::OPEN {
                if let Ok(txt) = serde_json::to_string(msg) {
                    let _ = ws.send_with_str(&txt);

                    // Track migrations sent
                    if matches!(msg, NetMessage::MigrateEntity { .. }) {
                        self.state.borrow_mut().migrations_sent += 1;
                    }
                }
            }
        }
    }

    /// Announce this peer's current state to the server
    pub fn announce(&self, entity_count: usize) {
        let state = self.state.borrow();
        let msg = NetMessage::PeerAnnounce {
            entity_count,
            migrations_sent: state.migrations_sent,
            migrations_received: state.migrations_received,
        };
        drop(state);
        self.send(&msg);
    }

    pub fn pop_pending(&self) -> Vec<NetMessage> {
        self.pending_migrations.borrow_mut().drain(..).collect()
    }

    pub fn pop_pending_limited(&self, limit: usize) -> Vec<NetMessage> {
        let mut pending = self.pending_migrations.borrow_mut();
        let count = pending.len().min(limit);
        pending.drain(..count).collect()
    }

    /// Get current network state (peers, stats)
    pub fn get_state(&self) -> NetworkState {
        self.state.borrow().clone()
    }

    /// Get number of connected peers
    pub fn peer_count(&self) -> usize {
        self.state.borrow().peers.len()
    }
}
