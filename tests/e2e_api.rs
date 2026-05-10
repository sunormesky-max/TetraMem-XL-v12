use std::net::SocketAddr;
use std::sync::Arc;

use reqwest::StatusCode;
use serde_json::{json, Value};
use tokio::net::TcpListener;

use tetramem_v12::universe::api::{create_router, AppState};
use tetramem_v12::universe::auth::{JwtConfig, UserStore};
use tetramem_v12::universe::backup::{BackupConfig, BackupScheduler};
use tetramem_v12::universe::cluster::ClusterManager;
use tetramem_v12::universe::config::AppConfig;
use tetramem_v12::universe::constitution::{Constitution, ImmutableRule};
use tetramem_v12::universe::crystal::CrystalEngine;
use tetramem_v12::universe::events::EventBus;
use tetramem_v12::universe::hebbian::HebbianMemory;
use tetramem_v12::universe::memory::semantic::{SemanticConfig, SemanticEngine};
use tetramem_v12::universe::memory::ClusteringEngine;
use tetramem_v12::universe::node::DarkUniverse;
use tetramem_v12::universe::perception::PerceptionBudget;
use tetramem_v12::universe::watchdog::Watchdog;

fn make_test_state() -> Arc<AppState> {
    let mut config = AppConfig::default();
    config.auth.enabled = false;
    config.server.cors_origins = vec!["*".to_string()];
    config.server.static_dir = Some("./panel/dist".to_string());

    let (event_sender, event_rx) = EventBus::create_channel();
    let event_bus = EventBus::from_receiver(event_rx);

    Arc::new(AppState {
        universe: tokio::sync::RwLock::new(DarkUniverse::new(config.universe.total_energy)),
        hebbian: tokio::sync::RwLock::new(HebbianMemory::new()),
        memory_store: tokio::sync::RwLock::new(tetramem_v12::universe::api::MemoryStore::new()),
        crystal: tokio::sync::RwLock::new(CrystalEngine::new()),
        perception: tokio::sync::RwLock::new(PerceptionBudget::new(1000.0)),
        semantic: tokio::sync::RwLock::new(SemanticEngine::new(SemanticConfig::default())),
        clustering: tokio::sync::RwLock::new(ClusteringEngine::with_default_config()),
        constitution: tokio::sync::RwLock::new(Constitution::new(
            vec![ImmutableRule {
                id: "manifestation_threshold".to_string(),
                description: "memory encoding threshold".to_string(),
            }],
            vec![],
        )),
        events: tokio::sync::Mutex::new(event_bus),
        event_sender,
        watchdog: tokio::sync::RwLock::new(Watchdog::with_defaults(10000.0)),
        backup: tokio::sync::RwLock::new(BackupScheduler::new(BackupConfig::default())),
        cluster: tokio::sync::Mutex::new(ClusterManager::new(1, "127.0.0.1:3456".to_string())),
        interests: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        memory_stream: tetramem_v12::universe::memory::create_broadcast_channel(),
        surfaced_seq: std::sync::atomic::AtomicU64::new(0),
        jwt: JwtConfig::new("test-secret".to_string(), 86400),
        users: UserStore::new(&[], "test-secret"),
        config,
        token_blocklist: tokio::sync::RwLock::new(
            tetramem_v12::universe::auth::TokenBlocklist::new(10_000),
        ),
        identity_guard: tokio::sync::RwLock::new(
            tetramem_v12::universe::safety::identity_guard::IdentityGuard::default(),
        ),
        plugins: tokio::sync::RwLock::new(tetramem_v12::universe::plugins::PluginManager::new(
            1_000_000,
        )),
        prediction: tokio::sync::RwLock::new(
            tetramem_v12::universe::cognitive::prediction::PredictionState::default(),
        ),
        shutdown: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
    })
}

async fn spawn_server() -> (SocketAddr, Arc<AppState>) {
    let state = make_test_state();
    let app = create_router(state.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (addr, state)
}

async fn get(client: &reqwest::Client, addr: SocketAddr, path: &str) -> (StatusCode, Value) {
    let url = format!("http://{}{}", addr, path);
    let res = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .unwrap();
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    let body: Value = serde_json::from_str(&text).unwrap_or_else(|_| json!({"raw": text}));
    (status, body)
}

async fn post(
    client: &reqwest::Client,
    addr: SocketAddr,
    path: &str,
    body: Value,
) -> (StatusCode, Value) {
    let url = format!("http://{}{}", addr, path);
    let res = client.post(&url).json(&body).send().await.unwrap();
    let status = res.status();
    let resp_body: Value = res.json().await.unwrap_or_default();
    (status, resp_body)
}

fn count(results: &[(&str, bool)]) -> (usize, usize) {
    let passed = results.iter().filter(|(_, ok)| *ok).count();
    (passed, results.len())
}

#[tokio::main]
async fn main() {
    let (addr, _state) = spawn_server().await;
    let client = reqwest::Client::new();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut results: Vec<(&str, bool)> = Vec::new();

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║    TetraMem-XL v12.0 — E2E HTTP API 集成测试           ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // ── 1. PUBLIC ROUTES ──
    println!("── 1. 公开路由 ──");

    let (status, body) = get(&client, addr, "/health").await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!("  {} /health → {}", if ok { "✓" } else { "✗" }, status);
    results.push(("GET /health", ok));

    // ── 2. STATS & METRICS ──
    println!("── 2. 统计 & 指标 ──");

    let (status, body) = get(&client, addr, "/api/stats").await;
    let ok = status == StatusCode::OK
        && body["success"].as_bool() == Some(true)
        && body["data"]["total_energy"].as_f64().unwrap_or(0.0) > 0.0;
    println!(
        "  {} /api/stats → {} nodes={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["nodes"]
    );
    results.push(("GET /api/stats", ok));

    let (status, _body) = get(&client, addr, "/api/metrics").await;
    let ok = status == StatusCode::OK;
    println!("  {} /api/metrics → {}", if ok { "✓" } else { "✗" }, status);
    results.push(("GET /api/metrics", ok));

    // ── 3. MEMORY CRUD ──
    println!("── 3. 记忆 CRUD ──");

    let (status, body) = post(
        &client,
        addr,
        "/api/memory/encode",
        json!({
            "anchor": [10, 20, 30],
            "data": [1.0, -2.5, 3.0, 0.0, 2.71],
            "tags": ["test", "e2e"],
            "category": "test",
            "description": "e2e test memory",
            "importance": 0.8,
        }),
    )
    .await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/memory/encode → {} anchor={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["anchor"]
    );
    results.push(("POST /api/memory/encode", ok));

    let (status, body) = post(
        &client,
        addr,
        "/api/memory/decode",
        json!({
            "anchor": [10, 20, 30],
            "data_dim": 5,
        }),
    )
    .await;
    let ok = status == StatusCode::OK
        && body["success"].as_bool() == Some(true)
        && body["data"]["data"]
            .as_array()
            .map(|a| a.len() == 5)
            .unwrap_or(false);
    println!(
        "  {} POST /api/memory/decode → {} dims={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["data"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0)
    );
    results.push(("POST /api/memory/decode", ok));

    let (status, body) = get(&client, addr, "/api/memory/list").await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} GET /api/memory/list → {} count={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"].as_array().map(|a| a.len()).unwrap_or(0)
    );
    results.push(("GET /api/memory/list", ok));

    let (status, body) = post(
        &client,
        addr,
        "/api/memory/annotate",
        json!({
            "anchor": [10, 20, 30],
            "tags": ["annotated"],
            "description": "updated by e2e",
        }),
    )
    .await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/memory/annotate → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("POST /api/memory/annotate", ok));

    // ── 4. AI AGENT MEMORY ──
    println!("── 4. AI 智能体记忆 ──");

    let (status, body) = post(
        &client,
        addr,
        "/api/memory/remember",
        json!({
            "content": "这是一条测试记忆，用于验证remember端点",
            "tags": ["agent", "test"],
            "category": "test",
            "importance": 0.7,
            "source": "e2e",
        }),
    )
    .await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/memory/remember → {} anchor={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["anchor"]
    );
    results.push(("POST /api/memory/remember", ok));

    let (status, body) = post(
        &client,
        addr,
        "/api/memory/recall",
        json!({
            "query": "测试记忆",
            "limit": 5,
        }),
    )
    .await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/memory/recall → {} results={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["total"]
    );
    results.push(("POST /api/memory/recall", ok));

    let (status, body) = post(
        &client,
        addr,
        "/api/memory/associate",
        json!({
            "topic": "测试",
            "depth": 3,
            "limit": 5,
        }),
    )
    .await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/memory/associate → {} total={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["total"]
    );
    results.push(("POST /api/memory/associate", ok));

    // ── 5. PULSE & DREAM ──
    println!("── 5. 脉冲 & 梦境 ──");

    let (status, body) = post(
        &client,
        addr,
        "/api/pulse",
        json!({
            "source": [10, 20, 30],
            "pulse_type": "exploratory",
        }),
    )
    .await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/pulse → {} visited={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["visited_nodes"]
    );
    results.push(("POST /api/pulse", ok));

    let (status, body) = post(&client, addr, "/api/dream", json!({})).await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/dream → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("POST /api/dream", ok));

    let (status, body) = post(
        &client,
        addr,
        "/api/dream/consolidate",
        json!({
            "importance_threshold": 0.3,
        }),
    )
    .await;
    let ok = status == StatusCode::OK || status == StatusCode::REQUEST_TIMEOUT;
    println!(
        "  {} POST /api/dream/consolidate → {} conservation={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["conservation_ok"]
    );
    results.push(("POST /api/dream/consolidate", ok));

    // ── 6. CONTEXT ──
    println!("── 6. 上下文管理 ──");

    let (status, body) = post(
        &client,
        addr,
        "/api/context",
        json!({
            "action": "status",
        }),
    )
    .await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/context (status) → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("POST /api/context status", ok));

    let (status, body) = post(
        &client,
        addr,
        "/api/context",
        json!({
            "action": "pre_work",
            "content": "测试",
        }),
    )
    .await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/context (pre_work) → {} results={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["total"]
    );
    results.push(("POST /api/context pre_work", ok));

    // ── 7. HEBBIAN ──
    println!("── 7. 赫布连接 ──");

    let (status, body) = get(&client, addr, "/api/hebbian/neighbors/10/20/30").await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} GET /api/hebbian/neighbors → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("GET /api/hebbian/neighbors", ok));

    // ── 8. DARK DIMENSION ──
    println!("── 8. 暗维度 ──");

    let (status, body) = post(
        &client,
        addr,
        "/api/dark/query",
        json!({
            "coord": [10, 20, 30, 0, 0, 0, 0],
        }),
    )
    .await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/dark/query → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("POST /api/dark/query", ok));

    let (status, body) = post(
        &client,
        addr,
        "/api/dark/materialize",
        json!({
            "coord": [50, 60, 70, 0, 0, 0, 0],
            "energy": 100.0,
            "physical_ratio": 0.5,
        }),
    )
    .await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/dark/materialize → {} conservation={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["conservation_ok"]
    );
    results.push(("POST /api/dark/materialize", ok));

    let (status, body) = get(&client, addr, "/api/dark/pressure").await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} GET /api/dark/pressure → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("GET /api/dark/pressure", ok));

    // ── 9. PHYSICS ──
    println!("── 9. 物理引擎 ──");

    let (status, body) = get(&client, addr, "/api/physics/status").await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} GET /api/physics/status → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("GET /api/physics/status", ok));

    let (status, body) = get(&client, addr, "/api/physics/profile").await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} GET /api/physics/profile → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("GET /api/physics/profile", ok));

    let (status, body) = post(
        &client,
        addr,
        "/api/physics/distance",
        json!({
            "from": [0, 0, 0, 0, 0, 0, 0],
            "to": [10, 20, 30, 0, 0, 0, 0],
        }),
    )
    .await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/physics/distance → {} d7d={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["distance_7d"]
    );
    results.push(("POST /api/physics/distance", ok));

    // ── 10. SEMANTIC ──
    println!("── 10. 语义引擎 ──");

    let (status, body) = get(&client, addr, "/api/semantic/status").await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} GET /api/semantic/status → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("GET /api/semantic/status", ok));

    let (status, body) = post(
        &client,
        addr,
        "/api/semantic/query",
        json!({
            "text": "测试",
            "k": 5,
        }),
    )
    .await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/semantic/query → {} results={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["results"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0)
    );
    results.push(("POST /api/semantic/query", ok));

    // ── 11. PHASE / CRYSTAL ──
    println!("── 11. 相变 / 晶体 ──");

    let (status, body) = get(&client, addr, "/api/phase/detect").await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} GET /api/phase/detect → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("GET /api/phase/detect", ok));

    // ── 12. EMOTION ──
    println!("── 12. 情绪系统 ──");

    let (status, body) = get(&client, addr, "/api/emotion/status").await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} GET /api/emotion/status → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("GET /api/emotion/status", ok));

    // ── 13. PERCEPTION ──
    println!("── 13. 感知预算 ──");

    let (status, body) = get(&client, addr, "/api/perception/status").await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} GET /api/perception/status → {} util={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["utilization"]
    );
    results.push(("GET /api/perception/status", ok));

    // ── 14. SUBSYSTEM STATUS ──
    println!("── 14. 子系统状态 ──");

    for (_name, path) in [
        ("clustering", "/api/clustering/status"),
        ("constitution", "/api/constitution/status"),
        ("events", "/api/events/status"),
        ("watchdog", "/api/watchdog/status"),
    ] {
        let (status, body) = get(&client, addr, path).await;
        let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
        println!("  {} GET {} → {}", if ok { "✓" } else { "✗" }, path, status);
        results.push((path, ok));
    }

    // ── 15. CLUSTER ──
    println!("── 15. 集群 ──");

    let (status, body) = get(&client, addr, "/api/cluster/status").await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} GET /api/cluster/status → {} role={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["role"]
    );
    results.push(("GET /api/cluster/status", ok));

    // ── 16. TIMELINE & TRACE ──
    println!("── 16. 时间轴 & 追踪 ──");

    let (status, body) = get(&client, addr, "/api/memory/timeline").await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} GET /api/memory/timeline → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("GET /api/memory/timeline", ok));

    let (status, body) = post(
        &client,
        addr,
        "/api/memory/trace",
        json!({
            "anchor": [10, 20, 30],
            "max_hops": 5,
        }),
    )
    .await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/memory/trace → {} hops={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"].as_array().map(|a| a.len()).unwrap_or(0)
    );
    results.push(("POST /api/memory/trace", ok));

    // ── 17. ADMIN ROUTES ──
    println!("── 17. 管理路由 ──");

    let (status, body) = post(&client, addr, "/api/regulate", json!({})).await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/regulate → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("POST /api/regulate", ok));

    let (status, body) = post(&client, addr, "/api/scale", json!({})).await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/scale → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("POST /api/scale", ok));

    let (status, body) = post(&client, addr, "/api/watchdog/checkup", json!({})).await;
    let ok = status == StatusCode::OK && body["success"].as_bool() == Some(true);
    println!(
        "  {} POST /api/watchdog/checkup → {} level={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["level"]
    );
    results.push(("POST /api/watchdog/checkup", ok));

    // ── 18. FORGET ──
    println!("── 18. 记忆删除 ──");

    let (status, body) = post(
        &client,
        addr,
        "/api/memory/forget",
        json!({
            "anchor": [0, 0, 0],
        }),
    )
    .await;
    let ok = status == StatusCode::OK || status == StatusCode::NOT_FOUND;
    println!(
        "  {} POST /api/memory/forget → {} remaining={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["remaining_memories"]
    );
    results.push(("POST /api/memory/forget", ok));

    let (status, _body) = post(
        &client,
        addr,
        "/api/memory/forget",
        json!({
            "anchor": [99, 99, 99],
        }),
    )
    .await;
    let ok = status == StatusCode::NOT_FOUND;
    println!(
        "  {} POST /api/memory/forget (404) → {}",
        if ok { "✓" } else { "✗" },
        status
    );
    results.push(("POST /api/memory/forget 404", ok));

    // ── 19. CONSERVATION VERIFY ──
    println!("── 19. 能量守恒验证 ──");

    let (status, body) = get(&client, addr, "/api/stats").await;
    let conservation_ok = body["data"]["conservation_ok"].as_bool().unwrap_or(false);
    let drift = body["data"]["energy_drift"].as_f64().unwrap_or(999.0);
    let ok = status == StatusCode::OK && conservation_ok && drift.abs() < 1e-6;
    println!(
        "  {} conservation_ok={} drift={:.2e}",
        if ok { "✓" } else { "✗" },
        conservation_ok,
        drift
    );
    results.push(("conservation_check", ok));

    // ── 21. NOVELTY ASSESSMENT ──
    println!("── 21. 新颖性评估 ──");

    let (status, body) = post(
        &client,
        addr,
        "/api/perception/assess",
        json!({"data": [1.0, 2.0, 3.0]}),
    )
    .await;
    let has_score = body["data"]["score"].is_number();
    let ok = status == StatusCode::OK && has_score;
    println!(
        "  {} POST /api/perception/assess → {} score={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["score"]
    );
    results.push(("POST /api/perception/assess", ok));

    // ── 22. INTEREST MANAGEMENT ──
    println!("── 22. 兴趣管理 ──");

    let (status, body) = post(
        &client,
        addr,
        "/api/interest/register",
        json!({"agent_id": "test-bot", "tags": ["science", "ai"], "min_importance": 0.3}),
    )
    .await;
    let registered = body["data"]["registered"].as_bool().unwrap_or(false);
    let ok = status == StatusCode::OK && registered;
    println!(
        "  {} POST /api/interest/register → {} registered={}",
        if ok { "✓" } else { "✗" },
        status,
        registered
    );
    results.push(("POST /api/interest/register", ok));

    let (status, body) = get(&client, addr, "/api/interest/list").await;
    let has_list = body["data"]
        .as_array()
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    let ok = status == StatusCode::OK && has_list;
    println!(
        "  {} GET /api/interest/list → {} agents={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"].as_array().map(|a| a.len()).unwrap_or(0)
    );
    results.push(("GET /api/interest/list", ok));

    let (status, body) = get(&client, addr, "/api/surface/status").await;
    let has_agents = body["data"]["registered_agents"].is_number();
    let ok = status == StatusCode::OK && has_agents;
    println!(
        "  {} GET /api/surface/status → {} agents={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["registered_agents"]
    );
    results.push(("GET /api/surface/status", ok));

    let (status, body) = post(
        &client,
        addr,
        "/api/interest/unregister",
        json!({"agent_id": "test-bot"}),
    )
    .await;
    let unregistered = body["data"]["unregistered"].as_bool().unwrap_or(false);
    let ok = status == StatusCode::OK && unregistered;
    println!(
        "  {} POST /api/interest/unregister → {} unregistered={}",
        if ok { "✓" } else { "✗" },
        status,
        unregistered
    );
    results.push(("POST /api/interest/unregister", ok));

    // ── 23. MEMORY AGING ──
    println!("── 23. 记忆老化 ──");

    let (status, body) = post(
        &client,
        addr,
        "/api/memory/aging",
        json!({"accessed_anchors": []}),
    )
    .await;
    let has_aged = body["data"]["aged_count"].is_number();
    let ok = status == StatusCode::OK && has_aged;
    println!(
        "  {} POST /api/memory/aging → {} aged={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["aged_count"]
    );
    results.push(("POST /api/memory/aging", ok));

    // ── 24. CONTRADICTION DETECTION ──
    println!("── 24. 矛盾检测 ──");

    let (status, body) = get(&client, addr, "/api/memory/contradictions").await;
    let has_contra = body["data"]["contradictions"].is_number();
    let ok = status == StatusCode::OK && has_contra;
    println!(
        "  {} GET /api/memory/contradictions → {} contradictions={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["contradictions"]
    );
    results.push(("GET /api/memory/contradictions", ok));

    // ── 25. TEMPORAL PREDICTION ──
    println!("── 25. 时序预测 ──");

    let (status, body) = post(
        &client,
        addr,
        "/api/memory/predict",
        json!({"anchor": [10, 20, 30], "max_steps": 3}),
    )
    .await;
    let has_pred = body["data"]["predictions"].is_array();
    let ok = status == StatusCode::OK && has_pred;
    println!(
        "  {} POST /api/memory/predict → {} predictions={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["predictions"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0)
    );
    results.push(("POST /api/memory/predict", ok));

    // ── 26. MEMORY RECONSTRUCTION ──
    println!("── 26. 记忆重建 ──");

    let (status, body) = post(
        &client,
        addr,
        "/api/memory/reconstruct",
        json!({"anchor": [10, 20, 30], "max_hops": 3}),
    )
    .await;
    let has_recon = body["data"]["reconstructed"].is_array();
    let ok = status == StatusCode::OK && has_recon;
    println!(
        "  {} POST /api/memory/reconstruct → {} reconstructed={}",
        if ok { "✓" } else { "✗" },
        status,
        body["data"]["reconstructed"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0)
    );
    results.push(("POST /api/memory/reconstruct", ok));

    // ── 20. STATIC FRONTEND ──
    println!("── 20. 静态前端 ──");

    let (status, _body) = get(&client, addr, "/").await;
    let has_html = status == StatusCode::OK
        || status == StatusCode::NOT_FOUND
        || status == StatusCode::UNAUTHORIZED;
    println!(
        "  {} GET / → {} (static serving {})",
        if has_html { "✓" } else { "✗" },
        status,
        if status == StatusCode::OK {
            "active"
        } else {
            "no dist/ or auth blocked"
        }
    );
    results.push(("GET / (static)", has_html));

    // ── SUMMARY ──
    println!();
    let (passed, total) = count(&results);
    println!("══════════════════════════════════════════");
    println!(
        "  结果: {}/{} 通过 ({:.0}%)",
        passed,
        total,
        passed as f64 / total as f64 * 100.0
    );
    println!("══════════════════════════════════════════");

    if passed < total {
        println!("\n  失败项:");
        for (name, ok) in &results {
            if !ok {
                println!("    ✗ {}", name);
            }
        }
    }

    std::process::exit(if passed == total { 0 } else { 1 });
}
