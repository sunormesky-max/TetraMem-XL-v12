// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::io::{BufRead, Write};

use serde_json::{json, Value};

use super::protocol::*;
use super::tools::TetraMemTools;
use crate::universe::config::AppConfig;
use crate::universe::crystal::CrystalEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::clustering::ClusteringEngine;
use crate::universe::memory::semantic::SemanticConfig;
use crate::universe::memory::semantic::SemanticEngine;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;

pub struct McpServer {
    universe: DarkUniverse,
    hebbian: HebbianMemory,
    memories: Vec<MemoryAtom>,
    crystal: CrystalEngine,
    semantic: SemanticEngine,
    clustering: ClusteringEngine,
    context_window: Vec<super::tools::ContextEntry>,
    context_max_tokens: usize,
}

impl McpServer {
    pub fn new(total_energy: f64) -> Self {
        Self {
            universe: DarkUniverse::new(total_energy),
            hebbian: HebbianMemory::new(),
            memories: Vec::new(),
            crystal: CrystalEngine::new(),
            semantic: SemanticEngine::new(SemanticConfig::default()),
            clustering: ClusteringEngine::with_default_config(),
            context_window: Vec::new(),
            context_max_tokens: 4096,
        }
    }

    pub fn from_config(config: &AppConfig) -> Self {
        Self::new(config.universe.total_energy)
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
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

    fn handle_request(&mut self, req: JsonRpcRequest) -> JsonRpcResponse {
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
                        "name": "tetramem-x12-mcp",
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
                let tools = TetraMemTools::definitions();
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
                let result = TetraMemTools::handle_tool(
                    &tool_name,
                    &args,
                    &mut self.universe,
                    &mut self.hebbian,
                    &mut self.memories,
                    &mut self.crystal,
                    &mut self.semantic,
                    &mut self.clustering,
                    &mut self.context_window,
                    self.context_max_tokens,
                );
                JsonRpcResponse::success(id, serde_json::to_value(result).unwrap_or_default())
            }
            "resources/list" => {
                let resources = TetraMemTools::resources();
                JsonRpcResponse::success(id, json!({ "resources": resources }))
            }
            "resources/read" => {
                let params = match req.params {
                    Some(p) => p,
                    None => {
                        return JsonRpcResponse::error(
                            id,
                            JsonRpcError::invalid_params("missing params"),
                        )
                    }
                };
                let uri = match params.get("uri").and_then(|v| v.as_str()) {
                    Some(u) => u.to_string(),
                    None => {
                        return JsonRpcResponse::error(
                            id,
                            JsonRpcError::invalid_params("missing uri"),
                        )
                    }
                };
                match TetraMemTools::read_resource(
                    &uri,
                    &self.universe,
                    &self.hebbian,
                    &self.memories,
                ) {
                    Some(content) => JsonRpcResponse::success(id, json!({ "contents": [content] })),
                    None => JsonRpcResponse::error(
                        id,
                        JsonRpcError::invalid_params(&format!("unknown resource: {}", uri)),
                    ),
                }
            }
            _ => JsonRpcResponse::error(id, JsonRpcError::method_not_found(&req.method)),
        }
    }
}

macro_rules! call_tool {
    ($server:expr, $name:expr, $args:expr) => {
        TetraMemTools::handle_tool(
            $name,
            &$args,
            &mut $server.universe,
            &mut $server.hebbian,
            &mut $server.memories,
            &mut $server.crystal,
            &mut $server.semantic,
            &mut $server.clustering,
            &mut $server.context_window,
            $server.context_max_tokens,
        )
    };
}

pub fn run_mcp_demo() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   TetraMem-XL v12.0 MCP Server Demo                    ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let mut server = McpServer::new(10_000_000.0);

    let tools = TetraMemTools::definitions();
    println!("MCP Tools ({} available):", tools.len());
    for t in &tools {
        println!("  * {} -- {}", t.name, t.description);
    }
    println!();

    println!("-- Demo: remember (Agent semantic memory) --");
    let args = json!({"content": "User prefers dark mode in all applications", "tags": ["preference", "ui"], "category": "user_preference", "importance": 0.8});
    let result = call_tool!(server, "tetramem_remember", args);
    println!("  {}\n", result.content[0].text);

    println!("-- Demo: remember more --");
    let args = json!({"content": "System uses Rust for backend services", "tags": ["tech", "architecture"], "category": "technical", "importance": 0.6});
    let result = call_tool!(server, "tetramem_remember", args);
    println!("  {}\n", result.content[0].text);

    println!("-- Demo: recall (semantic retrieval) --");
    let args = json!({"query": "user interface preferences", "limit": 5});
    let result = call_tool!(server, "tetramem_recall", args);
    println!("  {}\n", result.content[0].text);

    println!("-- Demo: stats --");
    let args = json!({});
    let result = call_tool!(server, "tetramem_stats", args);
    println!("  {}\n", result.content[0].text);

    println!("-- Demo: context overflow management --");
    let args = json!({"role": "user", "content": "This is a long conversation message that would normally overflow the context window. The user is discussing various topics including memory systems, AI agents, and the importance of persistent storage for long-running conversations."});
    let result = call_tool!(server, "tetramem_context", args);
    println!("  {}\n", result.content[0].text);

    println!("-- Demo: consolidate (dream cycle) --");
    let args = json!({});
    let result = call_tool!(server, "tetramem_consolidate", args);
    println!("  {}\n", result.content[0].text);

    println!("-- Demo: conservation check --");
    let result = call_tool!(server, "tetramem_conservation_check", args);
    println!("  {}\n", result.content[0].text);

    println!("OK MCP Server Demo complete -- all tools operational, conservation maintained");
}
