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
    let event_bus = tetramem_v12::universe::events::EventBus::new();
    let event_sender = event_bus.sender();
    Arc::new(AppState {
        universe: tokio::sync::RwLock::new(DarkUniverse::new(10_000_000.0)),
        hebbian: tokio::sync::RwLock::new(HebbianMemory::new()),
        memories: tokio::sync::RwLock::new(Vec::new()),
        memory_index: tokio::sync::RwLock::new(std::collections::HashMap::new()),
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
        events: std::sync::Mutex::new(event_bus),
        event_sender,
        watchdog: tokio::sync::RwLock::new(
            tetramem_v12::universe::watchdog::Watchdog::with_defaults(10_000_000.0),
        ),
        backup: tokio::sync::RwLock::new(BackupScheduler::with_defaults()),
        cluster: tokio::sync::Mutex::new(ClusterManager::new(1, "127.0.0.1:3456".to_string())),
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
    })
}

async fn body_json(body: Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn body_text(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

fn post(uri: &str, payload: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
}

fn get_with_body(uri: &str, payload: Value) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
}

#[tokio::test]
async fn test_health() {
    let state = build_state();
    let app = create_router(state);
    let resp = app.oneshot(get("/health")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["conservation_ok"].as_bool().unwrap());
    assert!(body["data"]["level"].is_string());
    assert!(body["data"]["node_count"].is_u64());
}

#[tokio::test]
async fn test_stats() {
    let state = build_state();
    let app = create_router(state);
    let resp = app.oneshot(get("/stats")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["total_energy"].as_f64().unwrap() > 0.0);
    assert_eq!(body["data"]["nodes"].as_u64().unwrap(), 0);
    assert!(body["data"]["conservation_ok"].as_bool().unwrap());
}

#[tokio::test]
async fn test_metrics() {
    let state = build_state();
    let app = create_router(state);
    let resp = app.oneshot(get("/metrics")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let text = body_text(resp.into_body()).await;
    assert!(!text.is_empty());
}

#[tokio::test]
async fn test_openapi() {
    let state = build_state();
    let app = create_router(state);
    let resp = app.oneshot(get("/openapi.json")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert_eq!(body["openapi"].as_str().unwrap(), "3.0.3");
    assert!(body["paths"].is_object());
    assert!(body["info"]["title"].as_str().unwrap().contains("TetraMem"));
}

#[tokio::test]
async fn test_encode_decode_roundtrip() {
    let state = build_state();

    let app = create_router(state.clone());
    let resp = app
        .oneshot(post(
            "/memory/encode",
            json!({"anchor": [5, 0, 0], "data": [1.0, 2.0, 3.0]}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let enc = body_json(resp.into_body()).await;
    assert!(enc["success"].as_bool().unwrap());
    assert_eq!(enc["data"]["data_dim"].as_u64().unwrap(), 3);
    assert!(enc["data"]["manifested"].as_bool().unwrap());

    let app = create_router(state);
    let resp = app
        .oneshot(post(
            "/memory/decode",
            json!({"anchor": [5, 0, 0], "data_dim": 3}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let dec = body_json(resp.into_body()).await;
    assert!(dec["success"].as_bool().unwrap());
    let data = dec["data"]["data"].as_array().unwrap();
    assert_eq!(data.len(), 3);
    assert!((data[0].as_f64().unwrap() - 1.0).abs() < 1e-10);
    assert!((data[1].as_f64().unwrap() - 2.0).abs() < 1e-10);
    assert!((data[2].as_f64().unwrap() - 3.0).abs() < 1e-10);
}

#[tokio::test]
async fn test_list_memories() {
    let state = build_state();

    let app = create_router(state.clone());
    let resp = app
        .oneshot(post(
            "/memory/encode",
            json!({"anchor": [20, 0, 0], "data": [42.0]}),
        ))
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let app = create_router(state);
    let resp = app.oneshot(get("/memory/list")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_pulse() {
    let state = build_state();
    let app = create_router(state);
    let resp = app
        .oneshot(post(
            "/pulse",
            json!({"source": [0, 0, 0], "pulse_type": "exploratory"}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["visited_nodes"].is_u64());
    assert!(body["data"]["total_activation"].is_f64());
}

#[tokio::test]
async fn test_dream() {
    let state = build_state();
    let app = create_router(state);
    let resp = app.oneshot(post("/dream", json!({}))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["edges_before"].is_u64());
    assert!(body["data"]["edges_after"].is_u64());
}

#[tokio::test]
async fn test_regulate() {
    let state = build_state();
    let app = create_router(state);
    let resp = app.oneshot(post("/regulate", json!({}))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"].is_array());
}

#[tokio::test]
async fn test_scale() {
    let state = build_state();
    let app = create_router(state);
    let resp = app.oneshot(post("/scale", json!({}))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["nodes_added"].is_u64());
    assert!(body["data"]["reason"].is_string());
}

#[tokio::test]
async fn test_backup() {
    let state = build_state();

    let app = create_router(state.clone());
    let resp = app
        .oneshot(post("/backup/create", json!({})))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["backup_id"].as_u64().unwrap() > 0);
    assert!(body["data"]["bytes"].as_u64().unwrap() > 0);

    let app = create_router(state);
    let resp = app.oneshot(get("/backup/list")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_dark_query() {
    let state = build_state();
    let app = create_router(state);
    let resp = app
        .oneshot(get_with_body(
            "/dark/query",
            json!({"coord": [99, 99, 99, 0, 0, 0, 0]}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(!body["data"]["exists"].as_bool().unwrap());
}

#[tokio::test]
async fn test_dark_materialize() {
    let state = build_state();
    let app = create_router(state);
    let resp = app
        .oneshot(post(
            "/dark/materialize",
            json!({"coord": [10, 10, 10, 0, 0, 0, 0], "energy": 100.0, "physical_ratio": 0.8}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["manifested"].as_bool().unwrap());
    assert!(body["data"]["energy"].as_f64().unwrap() > 0.0);
}

#[tokio::test]
async fn test_dark_pressure() {
    let state = build_state();
    let app = create_router(state);
    let resp = app.oneshot(get("/dark/pressure")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["dimension_spread"].is_array());
    assert_eq!(
        body["data"]["dimension_spread"].as_array().unwrap().len(),
        7
    );
    assert!(body["data"]["dark_node_count"].is_u64());
    assert!(body["data"]["physical_node_count"].is_u64());
}

#[tokio::test]
async fn test_login() {
    let state = build_state();
    let app = create_router(state);
    let resp = app
        .oneshot(post(
            "/login",
            json!({"username": "testuser", "password": "testpassword123"}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp.into_body()).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(!body["data"]["token"].as_str().unwrap().is_empty());
    assert!(body["data"]["expires_in"].as_u64().unwrap() > 0);
}
