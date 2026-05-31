use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    pub id: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcMessage {
    pub jsonrpc: Option<String>,
    pub id: Option<u64>,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Parsed notification types from Moonraker WebSocket.
#[derive(Debug, Clone)]
pub enum Notification {
    /// `notify_status_update` — delta of changed printer object fields.
    StatusUpdate {
        objects: HashMap<String, serde_json::Value>,
        eventtime: f64,
    },
    KlippyReady,
    KlippyDisconnected,
    KlippyShutdown,
    KlippyError(String),
    GcodeResponse(String),
    Unknown {
        method: String,
        params: serde_json::Value,
    },
    /// WebSocket connection state changed (internal, not from Moonraker).
    ConnectionState { connected: bool },
}
impl Notification {
    /// Attempt to parse a raw `JsonRpcMessage` into a typed `Notification`.
    /// Returns `None` for RPC responses (messages with `id`/`result`).
    pub fn from_message(msg: &JsonRpcMessage) -> Option<Self> {
        let method = msg.method.as_deref()?;
        let params = msg.params.clone().unwrap_or(serde_json::Value::Null);

        match method {
            "notify_status_update" => {
                let arr = params.as_array()?;
                let objects = arr
                    .first()
                    .and_then(|v| v.as_object())
                    .map(|m| m.clone().into_iter().collect())
                    .unwrap_or_default();
                let eventtime = arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
                Some(Notification::StatusUpdate { objects, eventtime })
            }
            "notify_klippy_ready" => Some(Notification::KlippyReady),
            "notify_klippy_disconnected" => Some(Notification::KlippyDisconnected),
            "notify_klippy_shutdown" => Some(Notification::KlippyShutdown),
            "notify_klippy_error" => {
                let msg_str = params
                    .as_str()
                    .map(String::from)
                    .unwrap_or_else(|| params.to_string());
                Some(Notification::KlippyError(msg_str))
            }
            "notify_gcode_response" => {
                let resp = params
                    .as_str()
                    .map(String::from)
                    .unwrap_or_else(|| params.to_string());
                Some(Notification::GcodeResponse(resp))
            }
            _ => Some(Notification::Unknown {
                method: method.to_string(),
                params,
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionIdentifyResult {
    pub connection_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerInfo {
    pub klippy_state: Option<String>,
    pub klippy_connected: Option<bool>,
    pub server_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObjectsQueryResponse {
    pub eventtime: Option<f64>,
    pub status: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GcodeScriptResponse {
    pub result: Option<String>,
}
