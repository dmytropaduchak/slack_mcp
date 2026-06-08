use anyhow::Result;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::info;

pub struct CdpClient {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: u64,
}

impl CdpClient {
    pub async fn connect(port: u16) -> Result<Self> {
        let ws_url = find_target(port).await?;
        info!("Connecting CDP: {}", ws_url);
        let (ws, _) = connect_async(&ws_url).await?;

        let mut client = Self { ws, next_id: 1 };
        client
            .send("Runtime.enable", serde_json::json!({}))
            .await?;

        Ok(client)
    }

    async fn send(&mut self, method: &str, params: Value) -> Result<Value> {
        use tokio_tungstenite::tungstenite::Message;
        let id = self.next_id;
        self.next_id += 1;

        let msg = serde_json::json!({
            "id": id,
            "method": method,
            "params": params,
        });

        let msg_str = serde_json::to_string(&msg)?;

        self.ws.send(Message::Text(msg_str.into())).await?;

        loop {
            match self.ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    let parsed: Value = serde_json::from_str(&text)?;
                    if parsed.get("id").and_then(|v| v.as_u64()) == Some(id) {

                        if let Some(error) = parsed.get("error") {
                            let msg = error
                                .get("message")
                                .and_then(|v| v.as_str())
                                .unwrap_or("CDP error");
                            anyhow::bail!("CDP {} error: {}", method, msg);
                        }
                        return Ok(parsed.get("result").cloned().unwrap_or(Value::Null));
                    }
                }
                Some(Ok(Message::Ping(data))) => {
                    self.ws.send(Message::Pong(data)).await?;
                }
                Some(Ok(_)) => {}
                Some(Err(e)) => anyhow::bail!("WebSocket error: {}", e),
                None => anyhow::bail!("WebSocket closed"),
            }
        }
    }

    pub async fn evaluate(&mut self, expression: &str) -> Result<Value> {

        let result = self
            .send(
                "Runtime.evaluate",
                serde_json::json!({
                    "expression": expression,
                    "returnByValue": true,
                    "awaitPromise": true,
                }),
            )
            .await?;

        if let Some(exception) = result.get("exceptionDetails") {
            let text = exception
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");

            anyhow::bail!("JS exception: {}", text);
        }

        let value = result
            .get("result")
            .and_then(|v| v.get("value"))
            .cloned()
            .unwrap_or(Value::Null);


        Ok(value)
    }
}

async fn http_get(path: &str, port: u16) -> Result<String> {
    use tokio::io::AsyncBufReadExt;
    use tokio::time::{timeout, Duration};

    let addr = format!("127.0.0.1:{}", port);
    let mut stream = timeout(Duration::from_secs(3), TcpStream::connect(&addr)).await??;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n",
        path, port
    );
    stream.write_all(request.as_bytes()).await?;

    let (reader, _) = stream.split();
    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut content_length: Option<usize> = None;

    loop {
        let mut line = String::new();
        let n = timeout(Duration::from_secs(3), buf_reader.read_line(&mut line)).await??;
        if n == 0 || line.trim().is_empty() {
            break;
        }
        if let Some(len) = line
            .strip_prefix("Content-Length:")
            .or_else(|| line.strip_prefix("content-length:"))
        {
            content_length = Some(len.trim().parse()?);
        }
    }

    let body: String = match content_length {
        Some(len) => {
            let mut buf = vec![0u8; len];
            timeout(Duration::from_secs(3), buf_reader.read_exact(&mut buf)).await??;
            String::from_utf8(buf)?
        }
        None => {
            let mut raw = Vec::new();
            timeout(Duration::from_secs(3), buf_reader.read_to_end(&mut raw)).await??;
            String::from_utf8_lossy(&raw).to_string()
        }
    };

    Ok(body)
}

async fn find_target(port: u16) -> Result<String> {
    let json = http_get("/json", port).await?;
    let targets: Vec<Value> = serde_json::from_str(&json)?;

    let matchers: &[fn(&Value) -> bool] = &[
        |t| matches_slack_webview(t),
        |t| matches_slack_title(t),
        |t| matches_page(t),
    ];

    for matcher in matchers {
        if let Some(t) = targets.iter().find(|t| matcher(t)) {
            if let Some(url) = t.get("webSocketDebuggerUrl").and_then(|u| u.as_str()) {
                return Ok(url.to_string());
            }
        }
    }

    anyhow::bail!(
        "No Slack page target found on port {}. Make sure Slack is running with --remote-debugging-port={}",
        port, port
    )
}

fn matches_slack_webview(t: &Value) -> bool {
    let url = t.get("url").and_then(|u| u.as_str()).unwrap_or("");
    let title = t.get("title").and_then(|u| u.as_str()).unwrap_or("");
    url.contains("app.slack.com") || title.contains("Slack") && url.contains("slack.com")
}

fn matches_slack_title(t: &Value) -> bool {
    let title = t.get("title").and_then(|u| u.as_str()).unwrap_or("");
    title.contains("Slack") || title.contains("slack")
}

fn matches_page(t: &Value) -> bool {
    let tp = t.get("type").and_then(|u| u.as_str()).unwrap_or("");
    tp == "page"
}

#[async_trait]
pub trait SlackAutomation: Send + Sync {
    async fn devtools_eval(&self, js: &str) -> anyhow::Result<String>;
}

pub struct CdpAutomation {
    cdp: Mutex<CdpClient>,
}

impl CdpAutomation {
    pub async fn new(port: u16) -> anyhow::Result<Self> {
        let cdp = CdpClient::connect(port).await?;
        Ok(Self {
            cdp: Mutex::new(cdp),
        })
    }

    async fn evaluate(&self, js: &str) -> anyhow::Result<String> {
        let mut guard = self.cdp.lock().await;

        let result = guard.evaluate(js).await?;

        let text = match result {
            serde_json::Value::String(s) => {
                s
            }
            other => {
                let s = serde_json::to_string_pretty(&other).unwrap_or_default();
                s
            }
        };
        Ok(text)
    }
}

#[async_trait]
impl SlackAutomation for CdpAutomation {
    async fn devtools_eval(&self, js: &str) -> anyhow::Result<String> {
        info!("devtools_eval, js_len={}", js.len());
        self.evaluate(js).await
    }
}
