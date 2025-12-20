//! V8 Inspector Protocol WebSocket client.
//!
//! Implements the Chrome DevTools Protocol (CDP) transport layer
//! for communicating with V8's built-in inspector.

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, trace, warn};

/// Inspector protocol errors
#[derive(Debug, Error)]
pub enum InspectorError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Connection closed")]
    ConnectionClosed,

    #[allow(dead_code)]
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Method error: {code} - {message}")]
    MethodError { code: i32, message: String },
}

/// CDP message sent to the inspector
#[derive(Debug, Clone, Serialize)]
pub struct CdpRequest {
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// CDP response from the inspector
#[derive(Debug, Clone, Deserialize)]
pub struct CdpResponse {
    pub id: Option<u64>,
    pub result: Option<serde_json::Value>,
    pub error: Option<CdpError>,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
}

/// CDP error in response
#[derive(Debug, Clone, Deserialize)]
pub struct CdpError {
    pub code: i32,
    pub message: String,
    #[allow(dead_code)]
    pub data: Option<serde_json::Value>,
}

/// Inspector event message
#[derive(Debug, Clone)]
pub struct InspectorMessage {
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// Pending request awaiting response
struct PendingRequest {
    sender: oneshot::Sender<Result<serde_json::Value, InspectorError>>,
}

/// V8 Inspector Protocol client
pub struct InspectorClient {
    /// WebSocket write sink
    ws_tx: mpsc::Sender<Message>,
    /// Event receiver
    event_rx: Arc<Mutex<mpsc::Receiver<InspectorMessage>>>,
    /// Pending requests map
    pending: Arc<Mutex<HashMap<u64, PendingRequest>>>,
    /// Next request ID
    next_id: Arc<AtomicU64>,
    /// Connection state
    connected: Arc<Mutex<bool>>,
}

impl InspectorClient {
    /// Connect to the V8 inspector at the given WebSocket URL
    pub async fn connect(url: &str, timeout_ms: u32) -> Result<Self, InspectorError> {
        let timeout_duration = Duration::from_millis(timeout_ms as u64);

        debug!(url = %url, "Connecting to V8 inspector");

        // Connect with timeout
        let ws_stream = timeout(timeout_duration, connect_async(url))
            .await
            .map_err(|_| InspectorError::Timeout)?
            .map_err(|e| InspectorError::ConnectionFailed(e.to_string()))?
            .0;

        let (ws_write, ws_read) = ws_stream.split();

        // Create channels
        let (ws_tx, mut ws_rx) = mpsc::channel::<Message>(64);
        let (event_tx, event_rx) = mpsc::channel::<InspectorMessage>(64);

        let pending: Arc<Mutex<HashMap<u64, PendingRequest>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let connected = Arc::new(Mutex::new(true));

        let pending_clone = pending.clone();
        let connected_clone = connected.clone();

        // Spawn writer task
        let ws_write = Arc::new(Mutex::new(ws_write));
        let ws_write_clone = ws_write.clone();

        tokio::spawn(async move {
            while let Some(msg) = ws_rx.recv().await {
                let mut write = ws_write_clone.lock().await;
                if let Err(e) = write.send(msg).await {
                    error!("WebSocket write error: {}", e);
                    break;
                }
            }
        });

        // Spawn reader task
        let mut ws_read = ws_read;
        tokio::spawn(async move {
            while let Some(msg_result) = ws_read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        trace!(msg = %text, "Received inspector message");

                        match serde_json::from_str::<CdpResponse>(&text) {
                            Ok(response) => {
                                // Check if this is a response to a request (has id)
                                if let Some(id) = response.id {
                                    let mut pending = pending_clone.lock().await;
                                    if let Some(request) = pending.remove(&id) {
                                        let result = if let Some(error) = response.error {
                                            Err(InspectorError::MethodError {
                                                code: error.code,
                                                message: error.message,
                                            })
                                        } else {
                                            Ok(response.result.unwrap_or(serde_json::Value::Null))
                                        };
                                        let _ = request.sender.send(result);
                                    }
                                }
                                // Check if this is an event (has method but no id)
                                else if let Some(method) = response.method {
                                    let event = InspectorMessage {
                                        method,
                                        params: response.params,
                                    };
                                    if event_tx.send(event).await.is_err() {
                                        debug!("Event receiver dropped");
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse inspector message: {}", e);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        debug!("WebSocket closed");
                        *connected_clone.lock().await = false;
                        break;
                    }
                    Ok(Message::Ping(_data)) => {
                        // Pong is handled automatically by tungstenite
                        trace!("Received ping");
                    }
                    Ok(_) => {
                        // Ignore binary, pong, frame messages
                    }
                    Err(e) => {
                        error!("WebSocket read error: {}", e);
                        *connected_clone.lock().await = false;
                        break;
                    }
                }
            }

            // Mark as disconnected
            *connected_clone.lock().await = false;

            // Cancel all pending requests
            let mut pending = pending_clone.lock().await;
            for (_, request) in pending.drain() {
                let _ = request.sender.send(Err(InspectorError::ConnectionClosed));
            }
        });

        debug!("Connected to V8 inspector");

        Ok(Self {
            ws_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
            pending,
            next_id: Arc::new(AtomicU64::new(1)),
            connected,
        })
    }

    /// Send a CDP method call and wait for response
    pub async fn send_method(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, InspectorError> {
        if !*self.connected.lock().await {
            return Err(InspectorError::ConnectionClosed);
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let request = CdpRequest {
            id,
            method: method.to_string(),
            params: if params.is_null() || params == serde_json::json!({}) {
                None
            } else {
                Some(params)
            },
        };

        let json = serde_json::to_string(&request)
            .map_err(|e| InspectorError::ProtocolError(e.to_string()))?;

        trace!(id = id, method = %method, "Sending inspector request");

        // Create response channel
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.insert(id, PendingRequest { sender: tx });
        }

        // Send the message
        self.ws_tx
            .send(Message::Text(json))
            .await
            .map_err(|e| InspectorError::WebSocketError(e.to_string()))?;

        // Wait for response with timeout
        let response = timeout(Duration::from_secs(30), rx)
            .await
            .map_err(|_| {
                // Remove from pending on timeout
                let pending = self.pending.clone();
                tokio::spawn(async move {
                    pending.lock().await.remove(&id);
                });
                InspectorError::Timeout
            })?
            .map_err(|_| InspectorError::ConnectionClosed)??;

        trace!(id = id, "Received inspector response");

        Ok(response)
    }

    /// Receive the next inspector event
    pub async fn receive_event(&self) -> Result<Option<InspectorMessage>, InspectorError> {
        let mut rx = self.event_rx.lock().await;

        // Use a very short timeout to make this non-blocking friendly
        match timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Some(event)) => Ok(Some(event)),
            Ok(None) => {
                // Channel closed
                if !*self.connected.lock().await {
                    Err(InspectorError::ConnectionClosed)
                } else {
                    Ok(None)
                }
            }
            Err(_) => {
                // Timeout - no events available
                Ok(None)
            }
        }
    }

    /// Check if still connected
    #[allow(dead_code)]
    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    /// Close the connection
    #[allow(dead_code)]
    pub async fn close(&self) {
        *self.connected.lock().await = false;
        let _ = self.ws_tx.send(Message::Close(None)).await;
    }
}

impl Drop for InspectorClient {
    fn drop(&mut self) {
        // Connection will be cleaned up by the spawned tasks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdp_request_serialization() {
        let request = CdpRequest {
            id: 1,
            method: "Debugger.enable".to_string(),
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"method\":\"Debugger.enable\""));
        assert!(!json.contains("params"));
    }

    #[test]
    fn test_cdp_request_with_params() {
        let request = CdpRequest {
            id: 2,
            method: "Debugger.setBreakpointByUrl".to_string(),
            params: Some(serde_json::json!({
                "url": "file:///test.js",
                "lineNumber": 10
            })),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"params\""));
        assert!(json.contains("\"url\""));
        assert!(json.contains("\"lineNumber\""));
    }

    #[test]
    fn test_cdp_response_parsing() {
        let json = r#"{"id":1,"result":{"debuggerId":"abc"}}"#;
        let response: CdpResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, Some(1));
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_cdp_event_parsing() {
        let json = r#"{"method":"Debugger.paused","params":{"reason":"breakpoint"}}"#;
        let response: CdpResponse = serde_json::from_str(json).unwrap();
        assert!(response.id.is_none());
        assert_eq!(response.method.as_deref(), Some("Debugger.paused"));
        assert!(response.params.is_some());
    }

    #[test]
    fn test_cdp_error_parsing() {
        let json = r#"{"id":1,"error":{"code":-32601,"message":"Method not found"}}"#;
        let response: CdpResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, Some(1));
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
    }
}
