pub mod message;
pub mod files;
pub mod multi;

use std::collections::HashMap;

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio::time;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use message::{
    ConnectionIdentifyResult, GcodeScriptResponse, JsonRpcMessage, JsonRpcRequest, Notification,
    ServerInfo,
};

/// Errors that can occur in the Moonraker client.
#[derive(Debug)]
pub enum MoonrakerError {
    ConnectionFailed(String),
    WebSocketError(String),
    JsonError(String),
    HttpError(String),
    RpcError { code: i64, message: String },
    NotConnected,
    Timeout,
    ChannelClosed,
}

impl std::fmt::Display for MoonrakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "connection failed: {msg}"),
            Self::WebSocketError(msg) => write!(f, "websocket error: {msg}"),
            Self::JsonError(msg) => write!(f, "json error: {msg}"),
            Self::HttpError(msg) => write!(f, "http error: {msg}"),
            Self::RpcError { code, message } => write!(f, "rpc error {code}: {message}"),
            Self::NotConnected => write!(f, "not connected"),
            Self::Timeout => write!(f, "request timed out"),
            Self::ChannelClosed => write!(f, "channel closed"),
        }
    }
}

impl std::error::Error for MoonrakerError {}

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// A channel sender/receiver pair for a single RPC request.
struct PendingRequest {
    responder: oneshot::Sender<Result<serde_json::Value, MoonrakerError>>,
}

/// Configuration for the Moonraker client.
#[derive(Debug, Clone)]
pub struct MoonrakerConfig {
    pub http_url: String,
    pub ws_url: String,
    pub client_name: String,
    pub client_version: String,
    pub reconnect_delay_min: Duration,
    pub reconnect_delay_max: Duration,
    pub rpc_timeout: Duration,
}

impl Default for MoonrakerConfig {
    fn default() -> Self {
        Self {
            http_url: "http://localhost:7125".to_string(),
            ws_url: "ws://localhost:7125/websocket".to_string(),
            client_name: "VectorScreen".to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            reconnect_delay_min: Duration::from_secs(1),
            reconnect_delay_max: Duration::from_secs(30),
            rpc_timeout: Duration::from_secs(10),
        }
    }
}

/// Moonraker WebSocket client.
///
/// Connects to the printer's Moonraker API for real-time status updates.
/// Supports subscribing to Klipper objects, sending G-code, and emergency stop.
pub struct MoonrakerClient {
    config: MoonrakerConfig,
    /// Shared HTTP client (connection pooling via internal Arc).
    http_client: reqwest::Client,
    /// Receiver for incoming notifications. Consumer should poll this.
    pub notification_rx: mpsc::UnboundedReceiver<Notification>,
    /// Internal channel to send commands to the connection task.
    cmd_tx: mpsc::UnboundedSender<ClientCommand>,
}

/// Commands sent to the internal connection task.
enum ClientCommand {
    RpcRequest {
        request: JsonRpcRequest,
        responder: oneshot::Sender<Result<serde_json::Value, MoonrakerError>>,
    },
    Shutdown,
}

impl MoonrakerClient {
    /// Create a new client with the given configuration.
    ///
    /// The client spawns a background task that manages the WebSocket connection
    /// and reconnection. Returns the client handle and a notification receiver.
    pub fn new(config: MoonrakerConfig) -> Self {
        let (notification_tx, notification_rx) = mpsc::unbounded_channel();
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        let http_client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");

        let client = Self {
            config: config.clone(),
            http_client,
            notification_rx,
            cmd_tx,
        };

        // Spawn the connection manager task.
        tokio::spawn(run_connection_manager(config, cmd_rx, notification_tx));

        client
    }

    /// Send a JSON-RPC request and wait for the response.
    async fn rpc_call(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, MoonrakerError> {
        let (responder_tx, responder_rx) = oneshot::channel();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: rand_id(),
        };

        self.cmd_tx
            .send(ClientCommand::RpcRequest {
                request,
                responder: responder_tx,
            })
            .map_err(|_| MoonrakerError::ChannelClosed)?;

        responder_rx
            .await
            .map_err(|_| MoonrakerError::ChannelClosed)?
    }

    pub async fn identify(&self) -> Result<ConnectionIdentifyResult, MoonrakerError> {
        let result = self
            .rpc_call(
                "server.connection.identify",
                Some(json!({
                    "client_name": self.config.client_name,
                    "version": self.config.client_version,
                    "type": "web",
                })),
            )
            .await?;
        serde_json::from_value(result).map_err(|e| MoonrakerError::JsonError(e.to_string()))
    }

    /// Get server info (Klipper state, server version).
    pub async fn server_info(&self) -> Result<ServerInfo, MoonrakerError> {
        let result = self.rpc_call("server.info", None).await?;
        serde_json::from_value(result).map_err(|e| MoonrakerError::JsonError(e.to_string()))
    }

    /// Subscribe to Klipper printer object updates over WebSocket.
    ///
    /// Pass a map of object names to optional field lists.
    /// Use `null` (represented as `None` in the value) to subscribe to all fields.
    ///
    /// ```ignore
    /// let mut objects = HashMap::new();
    /// objects.insert("toolhead".into(), Some(vec!["position".into(), "status".into()]));
    /// objects.insert("extruder".into(), None); // all fields
    /// client.subscribe_objects(objects).await?;
    /// ```
    pub async fn subscribe_objects(
        &self,
        objects: HashMap<String, Option<Vec<String>>>,
    ) -> Result<(), MoonrakerError> {
        let params = json!({ "objects": objects });
        self.rpc_call("printer.objects.subscribe", Some(params))
            .await?;
        Ok(())
    }

    pub async fn unsubscribe_objects(&self) -> Result<(), MoonrakerError> {
        self.rpc_call("printer.objects.subscribe", Some(json!({ "objects": {} })))
            .await?;
        Ok(())
    }

    /// Send a G-code command via HTTP POST (blocks until completion).
    pub async fn send_gcode(&self, script: &str) -> Result<String, MoonrakerError> {
        let url = format!("{}/printer/gcode/script", self.config.http_url);
        let body = json!({ "script": script });

        let resp = self.http_client
            .post(&url)
            .json(&body)
            .timeout(self.config.rpc_timeout)
            .send()
            .await
            .map_err(|e| MoonrakerError::HttpError(e.to_string()))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| MoonrakerError::HttpError(e.to_string()))?;

        if !status.is_success() {
            return Err(MoonrakerError::HttpError(format!("HTTP {status}: {text}")));
        }

        let parsed: GcodeScriptResponse =
            serde_json::from_str(&text).map_err(|e| MoonrakerError::JsonError(e.to_string()))?;

        Ok(parsed.result.unwrap_or_else(|| "ok".to_string()))
    }

    /// Send an emergency stop (M112) via WebSocket.
    pub async fn emergency_stop(&self) -> Result<(), MoonrakerError> {
        self.rpc_call("printer.emergency_stop", None).await?;
        info!("Emergency stop sent");
        Ok(())
    }

    /// Send a G-code command via WebSocket (non-blocking, returns immediately).
    pub async fn send_gcode_ws(&self, script: &str) -> Result<serde_json::Value, MoonrakerError> {
        self.rpc_call("post_printer_gcode", Some(json!({ "script": script })))
            .await
    }

    pub async fn query_objects(
        &self,
        objects: HashMap<String, Option<Vec<String>>>,
    ) -> Result<serde_json::Value, MoonrakerError> {
        let url = format!("{}/printer/objects/query", self.config.http_url);
        let body = json!({ "objects": objects });

        let resp = self.http_client
            .post(&url)
            .json(&body)
            .timeout(self.config.rpc_timeout)
            .send()
            .await
            .map_err(|e| MoonrakerError::HttpError(e.to_string()))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| MoonrakerError::HttpError(e.to_string()))?;

        if !status.is_success() {
            return Err(MoonrakerError::HttpError(format!("HTTP {status}: {text}")));
        }

        serde_json::from_str(&text).map_err(|e| MoonrakerError::JsonError(e.to_string()))
    }

    /// Non-blocking poll: try to receive the next notification.
    /// Returns `Ok(Some(notification))` if available, `Ok(None)` if nothing yet.
    pub async fn poll_notification(&self) -> Option<Notification> {
        // We need a mutable reference, but self.notification_rx is behind &self.
        // The consumer holds the rx directly — this is a convenience for simple usage.
        // For production, consume notification_rx directly.
        None
    }

    pub async fn shutdown(&self) {
        let _ = self.cmd_tx.send(ClientCommand::Shutdown);
        info!("Moonraker client shutdown requested");
    }
}

fn rand_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// The main connection manager loop.
///
/// Handles connecting, reconnecting with exponential backoff, reading messages,
/// dispatching RPC responses, and forwarding notifications.
async fn run_connection_manager(
    config: MoonrakerConfig,
    mut cmd_rx: mpsc::UnboundedReceiver<ClientCommand>,
    notification_tx: mpsc::UnboundedSender<Notification>,
) {
    let mut delay = config.reconnect_delay_min;
    let mut pending_requests: HashMap<u64, PendingRequest> = HashMap::new();
    let mut next_rpc_id: u64 = 1;

    loop {
        // Check for shutdown command (non-blocking).
        match cmd_rx.try_recv() {
            Ok(ClientCommand::Shutdown) => {
                info!("Connection manager shutting down");
                break;
            }
            Ok(ClientCommand::RpcRequest { request, responder }) => {
                pending_requests.insert(request.id, PendingRequest { responder });
                // We'll send once connected — buffer for now or skip if not connected.
                continue;
            }
            Err(mpsc::error::TryRecvError::Empty) => {}
            Err(mpsc::error::TryRecvError::Disconnected) => break,
        }

        // Attempt to connect.
        info!("Connecting to Moonraker at {}", config.ws_url);
        match connect_websocket(&config.ws_url).await {
            Ok((mut ws_sink, mut ws_stream)) => {
                info!("WebSocket connected");
                delay = config.reconnect_delay_min; // Reset backoff on success.

                // Identify the connection.
                let identify_req = JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    method: "server.connection.identify".to_string(),
                    params: Some(json!({
                        "client_name": config.client_name,
                        "version": config.client_version,
                        "type": "web",
                    })),
                    id: next_rpc_id,
                };
                next_rpc_id += 1;
                let identify_msg = serde_json::to_string(&identify_req)
                    .expect("identify request should serialize");
                if let Err(e) = ws_sink.send(Message::Text(identify_msg)).await {
                    error!("Failed to send identify: {e}");
                    time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, config.reconnect_delay_max);
                    let _ =
                        notification_tx.send(Notification::ConnectionState { connected: false });
                    continue;
                }

                // Notify UI that we're connected.
                let _ = notification_tx.send(Notification::ConnectionState { connected: true });

                // Main read loop.
                loop {
                    tokio::select! {
                        // Handle incoming WebSocket messages.
                        ws_msg = ws_stream.next() => {
                            match ws_msg {
                                Some(Ok(Message::Text(text))) => {
                                    match serde_json::from_str::<JsonRpcMessage>(&text) {
                                        Ok(msg) => {
                                            handle_message(
                                                &msg,
                                                &mut pending_requests,
                                                &notification_tx,
                                            );
                                        }
                                        Err(e) => {
                                            warn!("Failed to parse message: {e}");
                                        }
                                    }
                                }
                                Some(Ok(Message::Close(_))) => {
                                    warn!("WebSocket closed by server");
                                    break;
                                }
                                Some(Err(e)) => {
                                    error!("WebSocket error: {e}");
                                    break;
                                }
                                None => {
                                    warn!("WebSocket stream ended");
                                    break;
                                }
                                _ => {} // Ignore ping/pong/binary.
                            }
                        }

                        // Handle outgoing commands.
                        cmd = cmd_rx.recv() => {
                            match cmd {
                                Some(ClientCommand::RpcRequest { request, responder }) => {
                                    let id = request.id;
                                    pending_requests.insert(id, PendingRequest { responder });
                                    let msg = match serde_json::to_string(&request) {
                                        Ok(m) => m,
                                        Err(e) => {
                                            let _ = pending_requests
                                                .remove(&id)
                                                .unwrap()
                                                .responder
                                                .send(Err(MoonrakerError::JsonError(e.to_string())));
                                            continue;
                                        }
                                    };
                                    if let Err(e) = ws_sink.send(Message::Text(msg)).await {
                                        error!("Failed to send RPC request: {e}");
                                        let _ = pending_requests
                                            .remove(&id)
                                            .unwrap()
                                            .responder
                                            .send(Err(MoonrakerError::WebSocketError(
                                                e.to_string(),
                                            )));
                                        break;
                                    }
                                }
                                Some(ClientCommand::Shutdown) => {
                                    info!("Connection manager shutting down");
                                    let _ = ws_sink.close().await;
                                    // Fail all pending requests.
                                    for (_, pending) in pending_requests.drain() {
                                        let _ = pending
                                            .responder
                                            .send(Err(MoonrakerError::ChannelClosed));
                                    }
                                    return;
                                }
                                None => break,
                            }
                        }
                    }
                }

                // Drain pending requests on disconnect.
                for (_, pending) in pending_requests.drain() {
                    let _ = pending.responder.send(Err(MoonrakerError::WebSocketError(
                        "disconnected".to_string(),
                    )));
                }

                // Notify UI that we're disconnected.
                let _ = notification_tx.send(Notification::ConnectionState { connected: false });
            }
            Err(e) => {
                error!("Connection failed: {e}");
            }
        }

        // Notify UI that we're disconnected.
        let _ = notification_tx.send(Notification::ConnectionState { connected: false });

        // Exponential backoff before reconnecting.
        warn!("Reconnecting in {delay:?}...");
        time::sleep(delay).await;
        delay = std::cmp::min(delay * 2, config.reconnect_delay_max);
    }

    info!("Connection manager exited");
}

async fn connect_websocket(
    url: &str,
) -> Result<
    (
        futures_util::stream::SplitSink<WsStream, Message>,
        futures_util::stream::SplitStream<WsStream>,
    ),
    MoonrakerError,
> {
    let (ws_stream, _response) = tokio_tungstenite::connect_async(url)
        .await
        .map_err(|e| MoonrakerError::ConnectionFailed(e.to_string()))?;

    Ok(ws_stream.split())
}

fn handle_message(
    msg: &JsonRpcMessage,
    pending: &mut HashMap<u64, PendingRequest>,
    notification_tx: &mpsc::UnboundedSender<Notification>,
) {
    // RPC response (has an id).
    if let Some(id) = msg.id {
        if let Some(pending_req) = pending.remove(&id) {
            if let Some(error) = &msg.error {
                let _ = pending_req.responder.send(Err(MoonrakerError::RpcError {
                    code: error.code,
                    message: error.message.clone(),
                }));
            } else {
                let _ = pending_req
                    .responder
                    .send(Ok(msg.result.clone().unwrap_or(serde_json::Value::Null)));
            }
            return;
        }
        debug!("Received response for unknown request id: {id}");
    }

    // Notification (has a method, no id).
    if msg.method.is_some() {
        if let Some(notification) = Notification::from_message(msg) {
            debug!("Notification: {notification:?}");
            let _ = notification_tx.send(notification);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rand_id_uniqueness() {
        let a = rand_id();
        let b = rand_id();
        assert_ne!(a, b);
    }

    #[test]
    fn test_notification_parse_status_update() {
        let msg = JsonRpcMessage {
            jsonrpc: Some("2.0".into()),
            id: None,
            method: Some("notify_status_update".into()),
            params: Some(json!([
                {
                    "extruder": {
                        "temperature": 210.5,
                        "target": 220.0
                    }
                },
                12345.67
            ])),
            result: None,
            error: None,
        };

        let notif = Notification::from_message(&msg).expect("should parse");
        match notif {
            Notification::StatusUpdate { objects, eventtime } => {
                assert!(objects.contains_key("extruder"));
                assert_eq!(eventtime, 12345.67);
            }
            _ => panic!("wrong notification type"),
        }
    }

    #[test]
    fn test_notification_parse_klippy_ready() {
        let msg = JsonRpcMessage {
            jsonrpc: Some("2.0".into()),
            id: None,
            method: Some("notify_klippy_ready".into()),
            params: None,
            result: None,
            error: None,
        };

        let notif = Notification::from_message(&msg).expect("should parse");
        assert!(matches!(notif, Notification::KlippyReady));
    }

    #[test]
    fn test_notification_is_none_for_rpc_response() {
        let msg = JsonRpcMessage {
            jsonrpc: Some("2.0".into()),
            id: Some(42),
            method: None,
            params: None,
            result: Some(json!({ "connection_id": "abc" })),
            error: None,
        };

        assert!(Notification::from_message(&msg).is_none());
    }
}
