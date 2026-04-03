use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
const FUGLE_WS_URL: &str = "wss://api.fugle.tw/marketdata/v1.0/stock/streaming";

// ── Outgoing commands to the WS task ────────────────────────────────────────

#[derive(Debug)]
pub enum WsCommand {
    Subscribe {
        channel: String,
        symbol: String,
    },
    Unsubscribe {
        channel: String,
        symbol: String,
    },
    Disconnect,
}

// ── Payloads emitted to the frontend via Tauri events ───────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FugleTrade {
    pub symbol: String,
    pub price: Option<f64>,
    pub size: Option<f64>,
    pub volume: Option<f64>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub time: Option<String>,
    pub serial: Option<String>,
    #[serde(rename = "isOpen")]
    pub is_open: Option<bool>,
    #[serde(rename = "isClose")]
    pub is_close: Option<bool>,
    #[serde(rename = "isLimitUpPrice")]
    pub is_limit_up_price: Option<bool>,
    #[serde(rename = "isLimitDownPrice")]
    pub is_limit_down_price: Option<bool>,
    #[serde(rename = "isTrial")]
    pub is_trial: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLevel {
    pub price: Option<f64>,
    pub size: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FugleBook {
    pub symbol: String,
    pub bids: Vec<BookLevel>,
    pub asks: Vec<BookLevel>,
    pub time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FugleCandle {
    pub symbol: String,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: Option<f64>,
    pub volume: Option<f64>,
    pub average: Option<f64>,
    pub time: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FugleWsStatus {
    pub connected: bool,
    pub message: String,
}

// ── Client state managed by Tauri ───────────────────────────────────────────

pub struct FugleWsClient {
    cmd_tx: Arc<RwLock<Option<mpsc::Sender<WsCommand>>>>,
}

impl FugleWsClient {
    pub fn new() -> Self {
        Self {
            cmd_tx: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn is_connected(&self) -> bool {
        self.cmd_tx.read().await.is_some()
    }

    /// Spawn the WebSocket background task. Returns error if already connected.
    pub async fn connect(&self, api_key: String, app: AppHandle) -> Result<(), String> {
        if self.is_connected().await {
            return Err("Already connected".into());
        }

        let url = format!("{}?apikey={}", FUGLE_WS_URL, api_key);

        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| format!("WebSocket connect failed: {e}"))?;

        let (mut ws_write, mut ws_read) = ws_stream.split();
        let (tx, mut rx) = mpsc::channel::<WsCommand>(64);

        // Store the sender so commands can be sent later
        {
            let mut lock = self.cmd_tx.write().await;
            *lock = Some(tx);
        }

        let cmd_tx_clone = Arc::clone(&self.cmd_tx);

        // Emit connected status
        let _ = app.emit("fugle-ws-status", FugleWsStatus {
            connected: true,
            message: "Connected to Fugle WebSocket".into(),
        });

        // Spawn the WS event loop
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Incoming WS messages
                    msg = ws_read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                handle_ws_message(&app, &text);
                            }
                            Some(Ok(Message::Ping(data))) => {
                                let _ = ws_write.send(Message::Pong(data)).await;
                            }
                            Some(Ok(Message::Close(_))) | None => {
                                let _ = app.emit("fugle-ws-status", FugleWsStatus {
                                    connected: false,
                                    message: "WebSocket closed by server".into(),
                                });
                                break;
                            }
                            Some(Err(e)) => {
                                let _ = app.emit("fugle-ws-status", FugleWsStatus {
                                    connected: false,
                                    message: format!("WebSocket error: {e}"),
                                });
                                break;
                            }
                            _ => {}
                        }
                    }
                    // Outgoing commands from Tauri commands
                    cmd = rx.recv() => {
                        match cmd {
                            Some(WsCommand::Subscribe { channel, symbol }) => {
                                let msg = serde_json::json!({
                                    "event": "subscribe",
                                    "data": {
                                        "channel": channel,
                                        "symbol": symbol,
                                    }
                                });
                                if let Err(e) = ws_write.send(Message::Text(msg.to_string().into())).await {
                                    eprintln!("WS send error: {e}");
                                }
                            }
                            Some(WsCommand::Unsubscribe { channel, symbol }) => {
                                let msg = serde_json::json!({
                                    "event": "unsubscribe",
                                    "data": {
                                        "channel": channel,
                                        "symbol": symbol,
                                    }
                                });
                                if let Err(e) = ws_write.send(Message::Text(msg.to_string().into())).await {
                                    eprintln!("WS send error: {e}");
                                }
                            }
                            Some(WsCommand::Disconnect) | None => {
                                let _ = ws_write.send(Message::Close(None)).await;
                                let _ = app.emit("fugle-ws-status", FugleWsStatus {
                                    connected: false,
                                    message: "Disconnected".into(),
                                });
                                break;
                            }
                        }
                    }
                }
            }

            // Clear the sender so we know we're disconnected
            let mut lock = cmd_tx_clone.write().await;
            *lock = None;
        });

        Ok(())
    }

    pub async fn send_command(&self, cmd: WsCommand) -> Result<(), String> {
        let lock = self.cmd_tx.read().await;
        match lock.as_ref() {
            Some(tx) => tx.send(cmd).await.map_err(|e| format!("Send failed: {e}")),
            None => Err("Not connected".into()),
        }
    }

    pub async fn disconnect(&self) -> Result<(), String> {
        self.send_command(WsCommand::Disconnect).await
    }
}

// ── Parse incoming WS messages and emit typed Tauri events ──────────────────

fn handle_ws_message(app: &AppHandle, raw: &str) {
    let parsed: serde_json::Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(_) => return,
    };

    let event = parsed.get("event").and_then(|v| v.as_str()).unwrap_or("");

    match event {
        "data" => {
            let data = match parsed.get("data") {
                Some(d) => d,
                None => return,
            };
            let channel = parsed
                .get("channel")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match channel {
                c if c.starts_with("trades") => {
                    if let Ok(trade) = serde_json::from_value::<FugleTrade>(data.clone()) {
                        let _ = app.emit("fugle-trade", trade);
                    }
                }
                c if c.starts_with("books") => {
                    if let Ok(book) = serde_json::from_value::<FugleBook>(data.clone()) {
                        let _ = app.emit("fugle-book", book);
                    }
                }
                c if c.starts_with("candles") => {
                    if let Ok(candle) = serde_json::from_value::<FugleCandle>(data.clone()) {
                        let _ = app.emit("fugle-candle", candle);
                    }
                }
                _ => {
                    // Forward unknown channels as generic event
                    let _ = app.emit("fugle-raw", raw.to_string());
                }
            }
        }
        "authenticated" | "subscribed" | "unsubscribed" => {
            let _ = app.emit("fugle-raw", raw.to_string());
        }
        "error" => {
            let msg = parsed
                .get("data")
                .and_then(|d| d.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            let _ = app.emit("fugle-ws-status", FugleWsStatus {
                connected: true,
                message: format!("Error: {msg}"),
            });
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_trade_payload() {
        let json = serde_json::json!({
            "symbol": "2330",
            "price": 605.0,
            "size": 100.0,
            "volume": 12345.0,
            "bid": 604.0,
            "ask": 605.0,
            "time": "2026-04-04T09:00:00.000000+08:00",
            "serial": "001",
            "isOpen": true,
            "isClose": false,
            "isLimitUpPrice": false,
            "isLimitDownPrice": false,
            "isTrial": false,
        });
        let trade: FugleTrade = serde_json::from_value(json).unwrap();
        assert_eq!(trade.symbol, "2330");
        assert_eq!(trade.price, Some(605.0));
        assert_eq!(trade.size, Some(100.0));
        assert!(trade.is_open.unwrap());
    }

    #[test]
    fn parse_book_payload() {
        let json = serde_json::json!({
            "symbol": "2330",
            "bids": [
                { "price": 604.0, "size": 50.0 },
                { "price": 603.0, "size": 30.0 },
            ],
            "asks": [
                { "price": 605.0, "size": 40.0 },
                { "price": 606.0, "size": 20.0 },
            ],
            "time": "2026-04-04T09:00:01.000000+08:00",
        });
        let book: FugleBook = serde_json::from_value(json).unwrap();
        assert_eq!(book.symbol, "2330");
        assert_eq!(book.bids.len(), 2);
        assert_eq!(book.asks.len(), 2);
        assert_eq!(book.bids[0].price, Some(604.0));
    }

    #[test]
    fn parse_candle_payload() {
        let json = serde_json::json!({
            "symbol": "2330",
            "open": 600.0,
            "high": 610.0,
            "low": 598.0,
            "close": 605.0,
            "volume": 5000.0,
            "average": 604.5,
            "time": "2026-04-04T09:01:00.000000+08:00",
        });
        let candle: FugleCandle = serde_json::from_value(json).unwrap();
        assert_eq!(candle.symbol, "2330");
        assert_eq!(candle.open, Some(600.0));
        assert_eq!(candle.high, Some(610.0));
        assert_eq!(candle.close, Some(605.0));
    }

    #[test]
    fn parse_partial_trade() {
        let json = serde_json::json!({
            "symbol": "2454",
            "price": 800.0,
        });
        let trade: FugleTrade = serde_json::from_value(json).unwrap();
        assert_eq!(trade.symbol, "2454");
        assert_eq!(trade.price, Some(800.0));
        assert_eq!(trade.size, None);
        assert_eq!(trade.is_open, None);
    }
}
