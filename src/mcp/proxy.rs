// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::io::{BufRead, Write};

use serde_json::{json, Value};

use super::protocol::*;

pub struct McpProxy {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl McpProxy {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        let mut reader = stdin.lock();

        loop {
            let mut line = String::new();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(line) {
                Ok(r) => r,
                Err(_) => {
                    let resp = JsonRpcResponse::error(None, JsonRpcError::parse_error());
                    let _ = writeln!(stdout_lock, "{}", serde_json::to_string(&resp).unwrap());
                    stdout_lock.flush().ok();
                    continue;
                }
            };

            let id = request.id.clone();
            let response = self.handle_request(request);
            let json = serde_json::to_string(&response).unwrap_or_else(|_| {
                serde_json::to_string(&JsonRpcResponse::error(
                    id,
                    JsonRpcError::internal_error("serialization failed"),
                ))
                .unwrap()
            });

            writeln!(stdout_lock, "{}", json)?;
            stdout_lock.flush()?;
        }

        Ok(())
    }

    fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let id = req.id.clone();
        match req.method.as_str() {
            "initialize" => {
                let result = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": { "listChanged": false },
                        "resources": { "subscribe": false, "listChanged": false },
                    },
                    "serverInfo": {
                        "name": "tetramem-x12-mcp-proxy",
                        "version": "12.0.0",
                    }
                });
                JsonRpcResponse::success(id, result)
            }
            "notifications/initialized" => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: None,
                result: Some(Value::Null),
                error: None,
            },
            "ping" => JsonRpcResponse::success(id, json!({})),
            "tools/list" => {
                let tools = super::tools::TetraMemTools::definitions();
                JsonRpcResponse::success(id, json!({ "tools": tools }))
            }
            "tools/call" => {
                let params = match req.params {
                    Some(p) => p,
                    None => {
                        return JsonRpcResponse::error(
                            id,
                            JsonRpcError::invalid_params("missing params"),
                        )
                    }
                };
                let tool_name = match params.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n.to_string(),
                    None => {
                        return JsonRpcResponse::error(
                            id,
                            JsonRpcError::invalid_params("missing tool name"),
                        )
                    }
                };
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                let result = self.proxy_tool_call(&tool_name, &args);
                match result {
                    Ok(tool_result) => JsonRpcResponse::success(
                        id,
                        serde_json::to_value(tool_result).unwrap_or_default(),
                    ),
                    Err(e) => JsonRpcResponse::error(id, JsonRpcError::internal_error(&e)),
                }
            }
            "resources/list" => {
                let resources = super::tools::TetraMemTools::resources();
                JsonRpcResponse::success(id, json!({ "resources": resources }))
            }
            "resources/read" => JsonRpcResponse::error(
                id,
                JsonRpcError::invalid_params("resources/read not supported in proxy mode"),
            ),
            _ => JsonRpcResponse::error(id, JsonRpcError::method_not_found(&req.method)),
        }
    }

    fn proxy_tool_call(
        &self,
        tool_name: &str,
        args: &Value,
    ) -> Result<super::protocol::ToolCallResult, String> {
        let endpoint = match tool_name {
            "tetramem_stats" => "/api/stats",
            "tetramem_health" => "/api/health",
            "tetramem_encode" => "/api/memory/encode",
            "tetramem_decode" => "/api/memory/decode",
            "tetramem_list_memories" => "/api/memory/list",
            "tetramem_pulse" => "/api/pulse",
            "tetramem_dream" => "/api/dream",
            "tetramem_topology" => "/api/stats",
            "tetramem_regulate" => "/api/regulate",
            "tetramem_trace" => "/api/memory/trace",
            "tetramem_phase_detect" => "/api/phase/detect",
            "tetramem_materialize" => "/api/dark/materialize",
            "tetramem_conservation_check" => "/api/stats",
            "tetramem_remember" => "/api/memory/remember",
            "tetramem_recall" => "/api/memory/recall",
            "tetramem_associate" => "/api/memory/associate",
            "tetramem_consolidate" => "/api/dream/consolidate",
            "tetramem_context" => {
                return Ok(ToolCallResult::ok(String::from(
                    "context: proxy mode — use REST directly",
                )))
            }
            "tetramem_reason" => "/api/memory/associate",
            "tetramem_emotion" => "/api/emotion/status",
            "tetramem_scale" => "/api/scale",
            "tetramem_watchdog" => "/api/watchdog/checkup",
            "tetramem_forget" => "/api/memory/forget",
            "tetramem_cognitive_state" => "/api/cognitive/state",
            "tetramem_insights" => "/api/cognitive/insights",
            "tetramem_meta_cognitive" => "/api/cognitive/meta",
            _ => return Ok(ToolCallResult::err(format!("unknown tool: {}", tool_name))),
        };

        let url = format!("{}{}", self.base_url, endpoint);
        let is_get = matches!(
            tool_name,
            "tetramem_stats"
                | "tetramem_health"
                | "tetramem_list_memories"
                | "tetramem_topology"
                | "tetramem_conservation_check"
                | "tetramem_emotion"
                | "tetramem_cognitive_state"
                | "tetramem_insights"
                | "tetramem_meta_cognitive"
        );

        let resp = if is_get {
            self.client
                .get(&url)
                .send()
                .map_err(|e| format!("HTTP GET failed: {}", e))?
        } else {
            self.client
                .post(&url)
                .json(args)
                .send()
                .map_err(|e| format!("HTTP POST failed: {}", e))?
        };

        let status = resp.status();
        let body: Value = resp
            .json()
            .map_err(|e| format!("parse response failed: {}", e))?;

        if !status.is_success() {
            let msg = body["error"]["message"].as_str().unwrap_or("unknown error");
            return Ok(ToolCallResult::err(format!("HTTP {}: {}", status, msg)));
        }

        let data = body["data"].clone();
        let text = serde_json::to_string_pretty(&data).unwrap_or_else(|_| data.to_string());
        Ok(ToolCallResult::ok(text))
    }
}
