use crate::model::infra::network::NetMessage;
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
}

impl NetworkManager {
    pub fn new(url: &str) -> Self {
        let pending = Rc::new(RefCell::new(Vec::new()));

        let ws = match WebSocket::new(url) {
            Ok(ws) => {
                let pending_clone = pending.clone();

                // OnMessage callback
                let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                    if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                        let txt: String = txt.into();
                        if let Ok(msg) = serde_json::from_str::<NetMessage>(&txt) {
                            match msg {
                                NetMessage::MigrateEntity { .. } => {
                                    pending_clone.borrow_mut().push(msg);
                                }
                                _ => {} // Handle handshake/stats later
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
        }
    }

    pub fn send(&self, msg: &NetMessage) {
        if let Some(ws) = &self.ws {
            if ws.ready_state() == WebSocket::OPEN {
                if let Ok(txt) = serde_json::to_string(msg) {
                    let _ = ws.send_with_str(&txt);
                }
            }
        }
    }

    pub fn pop_pending(&self) -> Vec<NetMessage> {
        self.pending_migrations.borrow_mut().drain(..).collect()
    }
}
