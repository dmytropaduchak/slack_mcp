use std::io::BufRead;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tracing::{error, info, warn};

use crate::cdp::SlackAutomation;
use crate::tools;

#[derive(Deserialize)]
struct JsonRpcMessage {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: Option<String>,
    params: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

pub struct McpServer {
    automation: Box<dyn SlackAutomation>,
}

impl McpServer {
    pub fn new(automation: Box<dyn SlackAutomation>) -> Self {
        Self { automation }
    }

    pub async fn run(&self) -> Result<()> {
        info!("slack_mcp server starting");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        tokio::task::spawn_blocking(move || {
            let stdin = std::io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(l) => {
                        if tx.send(l).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("stdin read error: {}", e);
                        break;
                    }
                }
            }
        });

        while let Some(line) = rx.recv().await {
            if line.trim().is_empty() {
                continue;
            }

            let msg: JsonRpcMessage = match serde_json::from_str(&line) {
                Ok(m) => m,
                Err(e) => {
                    error!("Failed to parse JSON-RPC message: {}", e);
                    continue;
                }
            };

            let response = self.handle_message(&msg).await;

            if let Some(resp) = response {
                let json = serde_json::to_string(&resp)?;
                let mut stdout = tokio::io::stdout();
                stdout.write_all(json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }

        info!("slack_mcp server stopped");
        Ok(())
    }

    async fn handle_message(&self, msg: &JsonRpcMessage) -> Option<JsonRpcResponse> {
        let id = msg.id.clone();

        if id.is_none() {
            return None;
        }

        let id = id.unwrap();

        let method = match &msg.method {
            Some(m) => m.as_str(),
            None => {
                return Some(Self::error_response(
                    id,
                    -32600,
                    "Invalid Request: missing method".to_string(),
                    None,
                ))
            }
        };

        match method {
            "initialize" => Some(self.handle_initialize(id, msg.params.as_ref())),
            "notifications/initialized" | "initialized" => None,
            "ping" => Some(Self::success_response(id, serde_json::json!({}))),
            "tools/list" => Some(self.handle_tools_list(id)),
            "tools/call" => Some(self.handle_tools_call(id, msg.params.as_ref()).await),
            "resources/list" => Some(Self::success_response(id, serde_json::json!({"resources": []}))),
            "prompts/list" => Some(Self::success_response(id, serde_json::json!({"prompts": []}))),
            _ => {
                warn!("Unknown method: {}", method);
                Some(Self::error_response(
                    id,
                    -32601,
                    format!("Method not found: {}", method),
                    None,
                ))
            }
        }
    }

    fn handle_initialize(&self, id: Value, params: Option<&Value>) -> JsonRpcResponse {
        info!("MCP initialize request");
        let protocol_version = params
            .and_then(|p| p.get("protocolVersion"))
            .and_then(|v| v.as_str())
            .unwrap_or("2024-11-05")
            .to_string();
        Self::success_response(
            id,
            serde_json::json!({
                "protocolVersion": protocol_version,
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "slack_mcp",
                    "version": "0.1.0"
                }
            }),
        )
    }

    fn handle_tools_list(&self, id: Value) -> JsonRpcResponse {
        info!("MCP tools/list request");
        let definitions = tools::tool_definitions();
        let tools: Vec<Value> = definitions
            .into_iter()
            .map(|def| {
                serde_json::json!({
                    "name": def.name,
                    "description": def.description,
                    "inputSchema": def.input_schema
                })
            })
            .collect();

        Self::success_response(id, serde_json::json!({ "tools": tools }))
    }

    async fn handle_tools_call(&self, id: Value, params: Option<&Value>) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return Self::error_response(
                    id,
                    -32602,
                    "Invalid params: missing".to_string(),
                    None,
                )
            }
        };

        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => {
                return Self::error_response(
                    id,
                    -32602,
                    "Invalid params: missing tool name".to_string(),
                    None,
                )
            }
        };

        let empty = serde_json::json!({});
        let arguments = params.get("arguments").unwrap_or(&empty);

        info!("Executing tool: {} with arguments: {}", tool_name, arguments);

        let result = tools::handle_tool_call(tool_name, arguments, &*self.automation).await;

        let is_error = result.is_error();
        let content = result.into_content();

        Self::success_response(
            id,
            serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string(&content).unwrap_or_else(|_| "{}".to_string())
                    }
                ],
                "isError": is_error
            }),
        )
    }

    fn success_response(id: Value, result: Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            result: Some(result),
            error: None,
        }
    }

    fn error_response(id: Value, code: i32, message: String, data: Option<Value>) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            result: None,
            error: Some(JsonRpcError { code, message, data }),
        }
    }
}
