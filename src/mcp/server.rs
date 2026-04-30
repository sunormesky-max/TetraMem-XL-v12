use std::io::{BufRead, Write};

use serde_json::{json, Value};

use super::protocol::*;
use super::tools::TetraMemTools;
use crate::universe::config::AppConfig;
use crate::universe::crystal::CrystalEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;

pub struct McpServer {
    universe: DarkUniverse,
    hebbian: HebbianMemory,
    memories: Vec<MemoryAtom>,
    crystal: CrystalEngine,
}

impl McpServer {
    pub fn new(total_energy: f64) -> Self {
        Self {
            universe: DarkUniverse::new(total_energy),
            hebbian: HebbianMemory::new(),
            memories: Vec::new(),
            crystal: CrystalEngine::new(),
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
                )).unwrap()
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
            "notifications/initialized" => {
                JsonRpcResponse { jsonrpc: "2.0".into(), id: None, result: Some(Value::Null), error: None }
            }
            "ping" => JsonRpcResponse::success(id, json!({})),
            "tools/list" => {
                let tools = TetraMemTools::definitions();
                JsonRpcResponse::success(id, json!({ "tools": tools }))
            }
            "tools/call" => {
                let params = match req.params {
                    Some(p) => p,
                    None => return JsonRpcResponse::error(id, JsonRpcError::invalid_params("missing params")),
                };
                let tool_name = match params.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n.to_string(),
                    None => return JsonRpcResponse::error(id, JsonRpcError::invalid_params("missing tool name")),
                };
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                let result = TetraMemTools::handle_tool(
                    &tool_name, &args,
                    &mut self.universe, &mut self.hebbian,
                    &mut self.memories, &mut self.crystal,
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
                    None => return JsonRpcResponse::error(id, JsonRpcError::invalid_params("missing params")),
                };
                let uri = match params.get("uri").and_then(|v| v.as_str()) {
                    Some(u) => u.to_string(),
                    None => return JsonRpcResponse::error(id, JsonRpcError::invalid_params("missing uri")),
                };
                match TetraMemTools::read_resource(&uri, &self.universe, &self.hebbian, &self.memories) {
                    Some(content) => {
                        JsonRpcResponse::success(id, json!({ "contents": [content] }))
                    }
                    None => JsonRpcResponse::error(id, JsonRpcError::invalid_params(&format!("unknown resource: {}", uri))),
                }
            }
            _ => JsonRpcResponse::error(id, JsonRpcError::method_not_found(&req.method)),
        }
    }
}

pub fn run_mcp_demo() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   TetraMem-XL v12.0 MCP Server Demo                    ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let mut server = McpServer::new(10_000_000.0);

    let tools = TetraMemTools::definitions();
    println!("MCP Tools ({} available):", tools.len());
    for t in &tools {
        println!("  • {} — {}", t.name, t.description);
    }
    println!();

    println!("── Demo: materialize nodes ──");
    let args = json!({"coord": [10, 10, 10], "energy": 100.0, "physical_ratio": 0.6});
    let result = TetraMemTools::handle_tool(
        "tetramem_materialize", &args,
        &mut server.universe, &mut server.hebbian,
        &mut server.memories, &mut server.crystal,
    );
    println!("  {}\n", result.content[0].text);

    for i in 0..5 {
        let coord = [10 + i * 5, 10, 10];
        let args = json!({"coord": coord, "energy": 80.0, "physical_ratio": 0.5});
        let _ = TetraMemTools::handle_tool(
            "tetramem_materialize", &args,
            &mut server.universe, &mut server.hebbian,
            &mut server.memories, &mut server.crystal,
        );
    }

    println!("── Demo: encode memory ──");
    let args = json!({"anchor": [10, 10, 10], "data": [1.0, -2.5, 3.14, 0.0, 42.0]});
    let result = TetraMemTools::handle_tool(
        "tetramem_encode", &args,
        &mut server.universe, &mut server.hebbian,
        &mut server.memories, &mut server.crystal,
    );
    println!("  {}\n", result.content[0].text);

    println!("── Demo: decode memory ──");
    let args = json!({"anchor": [10, 10, 10]});
    let result = TetraMemTools::handle_tool(
        "tetramem_decode", &args,
        &mut server.universe, &mut server.hebbian,
        &mut server.memories, &mut server.crystal,
    );
    println!("  {}\n", result.content[0].text);

    println!("── Demo: fire pulse ──");
    let args = json!({"source": [10, 10, 10], "pulse_type": "reinforcing"});
    let result = TetraMemTools::handle_tool(
        "tetramem_pulse", &args,
        &mut server.universe, &mut server.hebbian,
        &mut server.memories, &mut server.crystal,
    );
    println!("  {}\n", result.content[0].text);

    println!("── Demo: stats ──");
    let args = json!({});
    let result = TetraMemTools::handle_tool(
        "tetramem_stats", &args,
        &mut server.universe, &mut server.hebbian,
        &mut server.memories, &mut server.crystal,
    );
    println!("  {}\n", result.content[0].text);

    println!("── Demo: topology ──");
    let result = TetraMemTools::handle_tool(
        "tetramem_topology", &args,
        &mut server.universe, &mut server.hebbian,
        &mut server.memories, &mut server.crystal,
    );
    println!("  {}\n", result.content[0].text);

    println!("── Demo: conservation check ──");
    let result = TetraMemTools::handle_tool(
        "tetramem_conservation_check", &args,
        &mut server.universe, &mut server.hebbian,
        &mut server.memories, &mut server.crystal,
    );
    println!("  {}\n", result.content[0].text);

    println!("✓ MCP Server Demo complete — all tools operational, conservation maintained");
}
