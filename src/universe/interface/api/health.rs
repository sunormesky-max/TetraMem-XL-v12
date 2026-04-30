use axum::{extract::State, Json};

use crate::universe::metrics;
use crate::universe::observer::UniverseObserver;

use super::state::SharedState;
use super::types::*;

pub async fn get_health(State(state): State<SharedState>) -> Json<ApiResponse<HealthResponse>> {
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let mems = state.memories.read().await;

    let report = UniverseObserver::inspect(&u, &h, &mems);

    let level = report.health_level().as_str().to_string();
    if level == "WARNING" || level == "CRITICAL" {
        tracing::warn!(health = %level, nodes = report.node_count, "universe health degraded");
    }

    Json(ApiResponse::ok(HealthResponse {
        level,
        conservation_ok: report.conservation_ok,
        energy_utilization: report.energy_utilization,
        node_count: report.node_count,
        manifested_ratio: report.manifested_ratio,
        hebbian_edge_count: report.hebbian_edge_count,
        hebbian_avg_weight: report.hebbian_avg_weight,
        memory_count: report.memory_count,
        frontier_size: report.frontier_size,
    }))
}

pub async fn get_stats(State(state): State<SharedState>) -> Json<ApiResponse<StatsResponse>> {
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let mems = state.memories.read().await;
    let stats = u.stats();

    tracing::debug!(nodes = stats.active_nodes, utilization = %format!("{:.1}%", stats.utilization * 100.0), "stats requested");

    Json(ApiResponse::ok(StatsResponse {
        nodes: stats.active_nodes,
        manifested: stats.manifested_nodes,
        dark: stats.dark_nodes,
        even: stats.even_nodes,
        odd: stats.odd_nodes,
        total_energy: stats.total_energy,
        allocated_energy: stats.allocated_energy,
        available_energy: stats.available_energy,
        physical_energy: stats.physical_energy,
        dark_energy: stats.dark_energy,
        utilization: stats.utilization,
        conservation_ok: u
            .verify_conservation_with_tolerance(state.config.universe.energy_drift_tolerance),
        energy_drift: u.energy_drift(),
        memory_count: mems.len(),
        hebbian_edges: h.edge_count(),
        hebbian_total_weight: h.total_weight(),
    }))
}

pub async fn get_metrics(State(state): State<SharedState>) -> String {
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let mems = state.memories.read().await;
    let stats = u.stats();
    metrics::update_universe_metrics(
        stats.active_nodes,
        stats.manifested_nodes,
        stats.dark_nodes,
        stats.total_energy,
        stats.allocated_energy,
        stats.available_energy,
        mems.len(),
        h.edge_count(),
    );
    drop(u);
    drop(h);
    drop(mems);
    metrics::render_metrics()
}

pub async fn get_openapi() -> Json<OpenApiDoc> {
    let paths: serde_json::Value = serde_json::from_str(r#"{
        "/health": {"get":{"summary":"Health check","responses":{"200":{"description":"OK"}}}},
        "/stats": {"get":{"summary":"Universe statistics","responses":{"200":{"description":"OK"}}}},
        "/metrics": {"get":{"summary":"Prometheus metrics","responses":{"200":{"description":"OK"}}}},
        "/openapi.json": {"get":{"summary":"OpenAPI spec","responses":{"200":{"description":"OK"}}}},
        "/login": {"post":{"summary":"Authenticate","responses":{"200":{"description":"JWT token"}}}},
        "/memory/encode": {"post":{"summary":"Encode memory","responses":{"200":{"description":"OK"}}}},
        "/memory/decode": {"post":{"summary":"Decode memory","responses":{"200":{"description":"OK"}}}},
        "/memory/list": {"get":{"summary":"List memories","responses":{"200":{"description":"OK"}}}},
        "/memory/timeline": {"get":{"summary":"Memory timeline by date","responses":{"200":{"description":"OK"}}}},
        "/memory/trace": {"post":{"summary":"Trace memory associations","responses":{"200":{"description":"OK"}}}},
        "/pulse": {"post":{"summary":"Fire pulse (reinforcing/exploratory/cascade)","responses":{"200":{"description":"OK"}}}},
        "/dream": {"post":{"summary":"Run dream cycle","responses":{"200":{"description":"OK"}}}},
        "/regulate": {"post":{"summary":"Run regulation cycle","responses":{"200":{"description":"OK"}}}},
        "/scale": {"post":{"summary":"Auto-scale universe","responses":{"200":{"description":"OK"}}}},
        "/scale/frontier/{max_new}": {"post":{"summary":"Frontier expansion","responses":{"200":{"description":"OK"}}}},
        "/hebbian/neighbors/{x}/{y}/{z}": {"get":{"summary":"Get Hebbian neighbors","responses":{"200":{"description":"OK"}}}},
        "/backup/create": {"post":{"summary":"Create manual backup","responses":{"200":{"description":"OK"}}}},
        "/backup/list": {"get":{"summary":"List backups","responses":{"200":{"description":"OK"}}}},
        "/cluster/status": {"get":{"summary":"Cluster status","responses":{"200":{"description":"OK"}}}},
        "/cluster/init": {"post":{"summary":"Initialize cluster","responses":{"200":{"description":"OK"}}}},
        "/cluster/propose": {"post":{"summary":"Propose Raft command","responses":{"200":{"description":"OK"}}}},
        "/cluster/add-node": {"post":{"summary":"Add cluster node","responses":{"200":{"description":"OK"}}}},
        "/cluster/remove-node": {"post":{"summary":"Remove cluster node","responses":{"200":{"description":"OK"}}}},
        "/dark/query": {"get":{"summary":"Query node 7D energy state","responses":{"200":{"description":"OK"}}}},
        "/dark/flow": {"post":{"summary":"Transfer energy between physical/dark","responses":{"200":{"description":"OK"}}}},
        "/dark/transfer": {"post":{"summary":"Transfer energy between 7D nodes","responses":{"200":{"description":"OK"}}}},
        "/dark/materialize": {"post":{"summary":"Materialize at full 7D coords","responses":{"200":{"description":"OK"}}}},
        "/dark/dematerialize": {"post":{"summary":"Dematerialize node","responses":{"200":{"description":"OK"}}}},
        "/dark/pressure": {"get":{"summary":"7D dimension pressure/entropy","responses":{"200":{"description":"OK"}}}},
        "/phase/detect": {"get":{"summary":"Detect phase transitions","responses":{"200":{"description":"OK"}}}},
        "/phase/consensus": {"post":{"summary":"Phase consensus proposal","responses":{"200":{"description":"OK"}}}},
        "/phase/quorum/start": {"post":{"summary":"Start energy quorum","responses":{"200":{"description":"OK"}}}},
        "/phase/quorum/confirm": {"post":{"summary":"Confirm quorum entry","responses":{"200":{"description":"OK"}}}},
        "/phase/quorum/status": {"get":{"summary":"Quorum status","responses":{"200":{"description":"OK"}}}},
        "/phase/quorum/execute": {"post":{"summary":"Execute quorum decision","responses":{"200":{"description":"OK"}}}}
    }"#).unwrap_or_default();

    Json(OpenApiDoc {
        openapi: "3.0.3".to_string(),
        info: OpenApiInfo {
            title: "TetraMem-XL v12.0 API".to_string(),
            version: "12.0.0".to_string(),
            description: "7D Dark Universe Memory System REST API".to_string(),
        },
        paths,
    })
}
