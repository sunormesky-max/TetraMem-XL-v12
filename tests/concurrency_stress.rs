use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use tetramem_v12::universe::api::{create_router, AppState};
use tetramem_v12::universe::auth::{JwtConfig, UserConfig, UserStore};
use tetramem_v12::universe::backup::BackupScheduler;
use tetramem_v12::universe::cluster::ClusterManager;
use tetramem_v12::universe::config::AppConfig;
use tetramem_v12::universe::crystal::CrystalEngine;
use tetramem_v12::universe::hebbian::HebbianMemory;
use tetramem_v12::universe::metrics;
use tetramem_v12::universe::node::DarkUniverse;

fn build_state() -> Arc<AppState> {
    std::env::set_var("TETRAMEM_ALLOW_NO_AUTH_ADMIN", "1");
    metrics::init_metrics();
    let mut config = AppConfig::default();
    config.auth.enabled = false;
    config.server.timeout_secs = 600;
    let (event_sender, event_rx) = tetramem_v12::universe::events::EventBus::create_channel();
    let event_bus = tetramem_v12::universe::events::EventBus::from_receiver(event_rx);
    Arc::new(AppState {
        universe: tokio::sync::RwLock::new(DarkUniverse::new(10_000_000.0)),
        hebbian: tokio::sync::RwLock::new(HebbianMemory::new()),
        memory_store: tokio::sync::RwLock::new(tetramem_v12::universe::api::MemoryStore::new()),
        crystal: tokio::sync::RwLock::new(CrystalEngine::new()),
        perception: tokio::sync::RwLock::new(
            tetramem_v12::universe::perception::PerceptionBudget::new(10_000_000.0),
        ),
        semantic: tokio::sync::RwLock::new(tetramem_v12::universe::memory::SemanticEngine::new(
            Default::default(),
        )),
        clustering: tokio::sync::RwLock::new(
            tetramem_v12::universe::memory::ClusteringEngine::new(
                tetramem_v12::universe::memory::ClusteringConfig::default(),
            ),
        ),
        constitution: tokio::sync::RwLock::new(
            tetramem_v12::universe::constitution::Constitution::tetramem_default(),
        ),
        events: tokio::sync::Mutex::new(event_bus),
        event_sender,
        watchdog: tokio::sync::RwLock::new(
            tetramem_v12::universe::watchdog::Watchdog::with_defaults(10_000_000.0),
        ),
        backup: tokio::sync::RwLock::new(BackupScheduler::with_defaults()),
        cluster: tokio::sync::Mutex::new(ClusterManager::new(1, "127.0.0.1:3456".to_string())),
        interests: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        memory_stream: tetramem_v12::universe::memory::create_broadcast_channel(),
        surfaced_seq: AtomicU64::new(0),
        config,
        jwt: JwtConfig::new("test-secret".to_string(), 3600),
        users: UserStore::new(
            &[UserConfig {
                username: "testuser".to_string(),
                password: "testpassword123".to_string(),
                password_hash: String::new(),
                role: "admin".to_string(),
            }],
            "test-secret",
        ),
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
        shutdown: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    })
}

fn encode_req(offset: i32, data: &[f64], desc: &str, tags: &[&str]) -> Request<Body> {
    Request::builder()
        .uri("/api/memory/encode")
        .method("POST")
        .header("content-type", "application/json")
        .body(
            json!({
                "anchor": [offset, 0, 0],
                "data": data,
                "description": desc,
                "tags": tags,
            })
            .to_string()
            .into(),
        )
        .unwrap()
}

#[tokio::test]
async fn concurrent_encode_no_data_loss() {
    let state = build_state();
    let app = create_router(state.clone());

    let n: usize = 2;
    let mut handles = Vec::new();

    for i in 0..n {
        let app_clone = app.clone();
        handles.push(tokio::spawn(async move {
            let data: Vec<f64> = (0..3).map(|d| (i * 3 + d) as f64).collect();
            let req = encode_req(i as i32 * 20, &data, &format!("item_{}", i), &["stress"]);
            let resp = app_clone.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let list_req = Request::builder()
        .uri("/api/memory/list?limit=100")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(list_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let list: Value = serde_json::from_slice(&body).unwrap();
    let memories = list["data"].as_array().unwrap();
    assert_eq!(memories.len(), n);
}

#[tokio::test]
async fn concurrent_encode_energy_conserved() {
    let state = build_state();
    let app = create_router(state.clone());

    let energy_before = {
        let u = state.universe.read().await;
        u.total_energy()
    };

    let n = 2;
    let mut handles = Vec::new();

    for i in 0..n {
        let app_clone = app.clone();
        handles.push(tokio::spawn(async move {
            let data: Vec<f64> = (0..3).map(|d| (i * 3 + d) as f64 * 0.1).collect();
            let req = encode_req(i * 20, &data, &format!("energy_{}", i), &["energy"]);
            let resp = app_clone.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let energy_after = {
        let u = state.universe.read().await;
        assert!(
            u.verify_conservation(),
            "Energy conservation violated after concurrent writes"
        );
        u.total_energy()
    };

    let diff = (energy_after - energy_before).abs();
    let tolerance = energy_before * 1e-10;
    assert!(
        diff <= tolerance || diff < 1e-6,
        "Total energy drift: before={}, after={}, diff={}",
        energy_before,
        energy_after,
        diff
    );
}

#[tokio::test]
async fn concurrent_read_write_no_deadlock() {
    let state = build_state();
    let app = create_router(state.clone());

    for i in 0..2i32 {
        let app_clone = app.clone();
        let data = vec![i as f64, 2.0, 3.0];
        let req = encode_req(i * 20, &data, &format!("seed_{}", i), &["setup"]);
        let resp = app_clone.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    let barrier = Arc::new(tokio::sync::Barrier::new(4));
    let mut handles = Vec::new();

    for i in 0..2i32 {
        let app_clone = app.clone();
        let barrier_clone = barrier.clone();
        handles.push(tokio::spawn(async move {
            barrier_clone.wait().await;
            let data = vec![i as f64, 1.0, 2.0];
            let req = encode_req(i * 20 + 100, &data, &format!("rw_{}", i), &["rw"]);
            let resp = app_clone.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }));
    }

    for _ in 0..2 {
        let app_clone = app.clone();
        let barrier_clone = barrier.clone();
        handles.push(tokio::spawn(async move {
            barrier_clone.wait().await;
            let req = Request::builder()
                .uri("/api/stats")
                .body(Body::empty())
                .unwrap();
            let resp = app_clone.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }));
    }

    let results = futures::future::join_all(handles).await;
    for r in results {
        r.unwrap();
    }

    let u = state.universe.read().await;
    assert!(
        u.verify_conservation(),
        "Conservation violated after r/w stress"
    );
}

#[tokio::test]
async fn concurrent_hebbian_consistent() {
    let state = build_state();
    let app = create_router(state.clone());

    let n = 2i32;
    let mut handles = Vec::new();

    for i in 0..n {
        let app_clone = app.clone();
        handles.push(tokio::spawn(async move {
            let data = vec![i as f64 * 0.5, 1.0, 2.0];
            let req = encode_req(i * 20, &data, &format!("hebbian_{}", i), &["hebbian"]);
            let resp = app_clone.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let h = state.hebbian.read().await;
    let total_weight: f64 = h.total_weight();
    assert!(
        total_weight.is_finite(),
        "Total Hebbian weight must be finite, got {}",
        total_weight
    );
    assert!(
        total_weight >= 0.0,
        "Total Hebbian weight must be non-negative"
    );
    drop(h);

    let u = state.universe.read().await;
    assert!(u.verify_conservation());
}
