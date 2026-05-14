// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::io::{BufRead, Write};
use std::path::PathBuf;

use serde_json::{json, Value};

use super::core::TetraMemCore;
use super::protocol::*;
use super::tools::TetraMemTools;
use crate::universe::api::SharedState;
use crate::universe::config::AppConfig;
use crate::universe::crystal::CrystalEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::{ClusteringEngine, SemanticEngine};
use crate::universe::node::DarkUniverse;

pub struct McpServer {
    state: SharedState,
    context_window: Vec<super::core::ContextEntry>,
    persist_path: Option<PathBuf>,
    persist_backend: String,
    auto_persist: bool,
    runtime: tokio::runtime::Runtime,
}

impl McpServer {
    pub fn new(total_energy: f64) -> Self {
        let mut config = AppConfig::default();
        config.auth.enabled = false;
        let state = crate::universe::api::build_shared_state(
            config,
            DarkUniverse::new(total_energy),
            HebbianMemory::new(),
            Vec::new(),
            CrystalEngine::new(),
            SemanticEngine::new(Default::default()),
            ClusteringEngine::with_default_config(),
        );
        Self {
            state,
            context_window: Vec::new(),
            persist_path: None,
            persist_backend: "json".to_string(),
            auto_persist: false,
            runtime: tokio::runtime::Runtime::new().expect("failed to create MCP runtime"),
        }
    }

    pub fn from_config(config: &AppConfig) -> Self {
        let persist_path = PathBuf::from(&config.backup.persist_path);
        let persist_backend = config.backup.persist_backend.clone();
        let (universe, hebbian, memories, crystal) = if persist_backend == "sqlite" {
            let sqlite_path = persist_path.with_extension("db");
            match crate::universe::persist_sqlite::PersistSqlite::load(&sqlite_path) {
                Ok((mut universe, hebbian, memories, crystal)) => {
                    universe.set_manifestation_threshold(config.universe.manifestation_threshold);
                    tracing::info!(
                        memories = memories.len(),
                        path = %sqlite_path.display(),
                        "loaded MCP state from SQLite"
                    );
                    (universe, hebbian, memories, crystal)
                }
                Err(e) => {
                    tracing::warn!(
                        path = %sqlite_path.display(),
                        error = %e,
                        "MCP SQLite load failed; starting fresh"
                    );
                    (
                        DarkUniverse::new_with_threshold(
                            config.universe.total_energy,
                            config.universe.manifestation_threshold,
                        ),
                        HebbianMemory::new(),
                        Vec::new(),
                        CrystalEngine::new(),
                    )
                }
            }
        } else if crate::universe::persist_file::PersistFile::exists(&persist_path) {
            match crate::universe::persist_file::PersistFile::load(&persist_path) {
                Ok((mut universe, hebbian, memories, crystal)) => {
                    universe.set_manifestation_threshold(config.universe.manifestation_threshold);
                    tracing::info!(
                        memories = memories.len(),
                        path = %persist_path.display(),
                        "loaded MCP state from JSON persistence"
                    );
                    (universe, hebbian, memories, crystal)
                }
                Err(e) => {
                    tracing::warn!(
                        path = %persist_path.display(),
                        error = %e,
                        "MCP JSON load failed; starting fresh"
                    );
                    (
                        DarkUniverse::new_with_threshold(
                            config.universe.total_energy,
                            config.universe.manifestation_threshold,
                        ),
                        HebbianMemory::new(),
                        Vec::new(),
                        CrystalEngine::new(),
                    )
                }
            }
        } else {
            (
                DarkUniverse::new_with_threshold(
                    config.universe.total_energy,
                    config.universe.manifestation_threshold,
                ),
                HebbianMemory::new(),
                Vec::new(),
                CrystalEngine::new(),
            )
        };
        let mut semantic = SemanticEngine::new(Default::default());
        let mut clustering = ClusteringEngine::with_default_config();
        let rebuilt = crate::universe::api::rebuild_derived_memory_indexes(
            &universe,
            &memories,
            &mut semantic,
            &mut clustering,
        );
        if rebuilt > 0 {
            tracing::info!(
                memories = rebuilt,
                "rebuilt MCP semantic and clustering indexes from persisted state"
            );
        }
        let state = crate::universe::api::build_shared_state(
            config.clone(),
            universe,
            hebbian,
            memories,
            crystal,
            semantic,
            clustering,
        );

        Self {
            state,
            context_window: Vec::new(),
            persist_path: Some(persist_path),
            persist_backend,
            auto_persist: config.backup.auto_persist,
            runtime: tokio::runtime::Runtime::new().expect("failed to create MCP runtime"),
        }
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

        self.persist_on_shutdown();
        Ok(())
    }

    fn persist_on_shutdown(&self) {
        if !self.auto_persist {
            return;
        }
        let Some(path) = &self.persist_path else {
            return;
        };

        if self.persist_backend == "sqlite" {
            let sqlite_path = path.with_extension("db");
            let save_result = self.runtime.block_on(async {
                let universe = self.state.universe.read().await;
                let hebbian = self.state.hebbian.read().await;
                let store = self.state.memory_store.read().await;
                let crystal = self.state.crystal.read().await;
                crate::universe::persist_sqlite::PersistSqlite::save(
                    &sqlite_path,
                    &universe,
                    &hebbian,
                    &store.memories,
                    &crystal,
                )
            });
            match save_result {
                Ok(rows) => tracing::info!(
                    rows,
                    path = %sqlite_path.display(),
                    "persisted MCP state to SQLite"
                ),
                Err(e) => tracing::warn!(
                    path = %sqlite_path.display(),
                    error = %e,
                    "failed to persist MCP state to SQLite"
                ),
            }
        } else {
            let save_result = self.runtime.block_on(async {
                let universe = self.state.universe.read().await;
                let hebbian = self.state.hebbian.read().await;
                let store = self.state.memory_store.read().await;
                let crystal = self.state.crystal.read().await;
                crate::universe::persist_file::PersistFile::save(
                    path,
                    &universe,
                    &hebbian,
                    &store.memories,
                    &crystal,
                )
            });
            match save_result {
                Ok(report) => tracing::info!(
                    report = %report,
                    path = %path.display(),
                    "persisted MCP state to JSON"
                ),
                Err(e) => tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "failed to persist MCP state to JSON"
                ),
            }
        }
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
                let result =
                    self.with_core(|core| TetraMemTools::handle_tool(&tool_name, &args, core));
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
                match self.with_core(|core| TetraMemTools::read_resource(&uri, core)) {
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

    fn with_core<R>(&mut self, f: impl FnOnce(&mut TetraMemCore) -> R) -> R {
        let state = self.state.clone();
        let context_window = &mut self.context_window;
        self.runtime.block_on(async move {
            let mut semantic = state.semantic.write().await;
            let mut crystal = state.crystal.write().await;
            let mut clustering = state.clustering.write().await;
            let mut hebbian = state.hebbian.write().await;
            let mut store = state.memory_store.write().await;
            let mut universe = state.universe.write().await;

            let total_energy = universe.total_energy();
            let mut core = TetraMemCore {
                universe: std::mem::replace(&mut *universe, DarkUniverse::new(total_energy)),
                hebbian: std::mem::take(&mut *hebbian),
                memories: std::mem::take(&mut store.memories),
                crystal: std::mem::take(&mut *crystal),
                semantic: std::mem::replace(
                    &mut *semantic,
                    SemanticEngine::new(Default::default()),
                ),
                clustering: std::mem::replace(
                    &mut *clustering,
                    ClusteringEngine::with_default_config(),
                ),
                context_window: std::mem::take(context_window),
                context_max_tokens: 4096,
            };
            store.index.clear();

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&mut core)));

            *universe = core.universe;
            *hebbian = core.hebbian;
            store.memories = core.memories;
            store.rebuild_index();
            *crystal = core.crystal;
            *semantic = core.semantic;
            *clustering = core.clustering;
            *context_window = core.context_window;

            match result {
                Ok(result) => result,
                Err(payload) => std::panic::resume_unwind(payload),
            }
        })
    }
}

macro_rules! call_tool {
    ($server:expr, $name:expr, $args:expr) => {
        $server.with_core(|core| TetraMemTools::handle_tool($name, &$args, core))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_tools_write_to_shared_state() {
        let mut server = McpServer::new(1_000_000.0);
        let args = json!({
            "anchor": [4, 0, 0],
            "data": [1.0, 2.0, 3.0],
        });

        let result =
            server.with_core(|core| TetraMemTools::handle_tool("tetramem_encode", &args, core));

        assert_ne!(result.is_error, Some(true));
        let len = server
            .runtime
            .block_on(async { server.state.memory_store.read().await.len() });
        assert_eq!(len, 1);
    }

    #[test]
    fn mcp_state_is_restored_after_tool_panic() {
        let mut server = McpServer::new(1_000_000.0);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            server.with_core(|core| {
                let anchor = crate::universe::coord::Coord7D::new_even([6, 0, 0, 0, 0, 0, 0]);
                let memory = crate::universe::memory::MemoryCodec::encode(
                    &mut core.universe,
                    &anchor,
                    &[1.0],
                )
                .unwrap();
                core.memories.push(memory);
                panic!("intentional test panic");
            });
        }));

        assert!(result.is_err());
        let len = server
            .runtime
            .block_on(async { server.state.memory_store.read().await.len() });
        assert_eq!(len, 1);
    }
}
