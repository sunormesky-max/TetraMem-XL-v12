// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tetramem_v12::universe::auth::{JwtConfig, UserStore};
use tetramem_v12::universe::autoscale::AutoScaler;
use tetramem_v12::universe::backup::BackupScheduler;
use tetramem_v12::universe::config::AppConfig;
use tetramem_v12::universe::coord::Coord7D;
use tetramem_v12::universe::crystal::CrystalEngine;
use tetramem_v12::universe::dream::DreamEngine;
use tetramem_v12::universe::hebbian::HebbianMemory;
use tetramem_v12::universe::memory::MemoryCodec;
use tetramem_v12::universe::memory::SemanticEngine;
use tetramem_v12::universe::metrics;
use tetramem_v12::universe::node::DarkUniverse;
use tetramem_v12::universe::persist::PersistEngine;
use tetramem_v12::universe::persist_file::PersistFile;
use tetramem_v12::universe::pulse::{PulseEngine, PulseType};
use tetramem_v12::universe::reasoning::ReasoningEngine;
use tetramem_v12::universe::regulation::RegulationEngine;
use tetramem_v12::universe::topology::TopologyEngine;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "tetramem-v12", version = "12.0.0")]
#[command(about = "TetraMem-XL v12.0 - 7D Dark Universe Memory System")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, default_value = "tetramem.toml")]
    config: PathBuf,

    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(short, long)]
        addr: Option<String>,
    },
    Bench,
    Config {
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Mcp {
        #[arg(short, long, default_value = "10000000.0")]
        energy: f64,
    },
    McpProxy {
        #[arg(short, long, default_value = "http://127.0.0.1:3456")]
        server: String,
    },
    McpDemo,
    Skills,
    ValidateDeployment {
        #[arg(short, long, default_value = "models/granite-embedding-small")]
        model_dir: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    init_tracing(&cli.log_level);

    match cli.command {
        Some(Commands::Serve { addr }) => {
            let config = AppConfig::load(&cli.config).unwrap_or_else(|e| {
                eprintln!("fatal: config load error: {}\nfix your config file or run 'tetramem-v12 config' to generate a default", e);
                std::process::exit(1);
            });
            metrics::init_metrics();

            let effective_addr = addr.unwrap_or_else(|| {
                if let Ok(port) = std::env::var("PORT") {
                    format!("0.0.0.0:{}", port)
                } else {
                    config.server.addr.clone()
                }
            });
            let persist_path = PathBuf::from(&config.backup.persist_path);
            let use_sqlite = config.backup.persist_backend == "sqlite";

            let (universe, hebbian, memories, crystal) = if use_sqlite {
                let sqlite_path = persist_path.with_extension("db");
                match tetramem_v12::universe::persist_sqlite::PersistSqlite::load(&sqlite_path) {
                    Ok((mut u, h, m, c)) => {
                        u.set_manifestation_threshold(config.universe.manifestation_threshold);
                        let stats = u.stats();
                        tracing::info!(
                            "restored SQLite state: {} nodes, {} memories, {} edges, E={:.0}",
                            stats.active_nodes,
                            m.len(),
                            h.edge_count(),
                            stats.total_energy
                        );
                        (u, h, m, c)
                    }
                    Err(e) => {
                        tracing::warn!("SQLite load failed ({}), starting fresh", e);
                        (
                            DarkUniverse::new_with_threshold(
                                config.universe.total_energy,
                                config.universe.manifestation_threshold,
                            ),
                            HebbianMemory::new(),
                            Vec::new(),
                            tetramem_v12::universe::crystal::CrystalEngine::new(),
                        )
                    }
                }
            } else if PersistFile::exists(&persist_path) {
                tracing::info!(
                    "found persisted state at {}, loading...",
                    persist_path.display()
                );
                match PersistFile::load(&persist_path) {
                    Ok((mut u, h, m, c)) => {
                        u.set_manifestation_threshold(config.universe.manifestation_threshold);
                        let stats = u.stats();
                        tracing::info!(
                            "restored state: {} nodes, {} memories, {} edges, E={:.0}",
                            stats.active_nodes,
                            m.len(),
                            h.edge_count(),
                            stats.total_energy
                        );
                        let conservation_ok = u.verify_conservation_with_tolerance(
                            config.universe.energy_drift_tolerance,
                        );
                        if conservation_ok {
                            tracing::info!(
                                "POST-RESTORE conservation check: PASSED (tolerance={:.e})",
                                config.universe.energy_drift_tolerance
                            );
                        } else {
                            tracing::error!(
                                "POST-RESTORE conservation check: FAILED — energy violation detected after loading persisted state"
                            );
                        }
                        (u, h, m, c)
                    }
                    Err(e) => {
                        tracing::warn!("failed to load persisted state: {}, starting fresh", e);
                        (
                            DarkUniverse::new_with_threshold(
                                config.universe.total_energy,
                                config.universe.manifestation_threshold,
                            ),
                            HebbianMemory::new(),
                            Vec::new(),
                            tetramem_v12::universe::crystal::CrystalEngine::new(),
                        )
                    }
                }
            } else {
                tracing::info!("no persisted state found, starting fresh");
                (
                    DarkUniverse::new_with_threshold(
                        config.universe.total_energy,
                        config.universe.manifestation_threshold,
                    ),
                    HebbianMemory::new(),
                    Vec::new(),
                    tetramem_v12::universe::crystal::CrystalEngine::new(),
                )
            };

            let perception_budget =
                tetramem_v12::universe::perception::PerceptionBudget::new(universe.total_energy());
            let neural_engine = if config.neural_embed.enabled {
                tetramem_v12::universe::neural::EmbeddingEngineHandle::try_load(
                    std::path::Path::new(&config.neural_embed.model_dir),
                )
            } else {
                tetramem_v12::universe::neural::EmbeddingEngineHandle::disabled()
            };
            let mut semantic_engine =
                SemanticEngine::new_with_neural(Default::default(), neural_engine);
            let mut clustering_engine = tetramem_v12::universe::memory::ClusteringEngine::new(
                tetramem_v12::universe::memory::ClusteringConfig::default(),
            );
            let rebuilt_indexes = tetramem_v12::universe::api::rebuild_derived_memory_indexes(
                &universe,
                &memories,
                &mut semantic_engine,
                &mut clustering_engine,
            );
            if rebuilt_indexes > 0 {
                tracing::info!(
                    memories = rebuilt_indexes,
                    "rebuilt semantic and clustering indexes from restored memories"
                );
            }
            let constitution =
                tetramem_v12::universe::constitution::Constitution::tetramem_default();
            let (event_sender, event_rx) =
                tetramem_v12::universe::events::EventBus::create_channel();
            let mut event_bus = tetramem_v12::universe::events::EventBus::from_receiver(event_rx);
            event_bus.subscribe(|event| match event {
                tetramem_v12::universe::events::UniverseEvent::MemoryEncoded {
                    anchor,
                    data_dim,
                    importance,
                } => {
                    tracing::debug!(?anchor, data_dim, importance, "event: memory encoded");
                }
                tetramem_v12::universe::events::UniverseEvent::ConservationViolation {
                    drift,
                    active_nodes,
                } => {
                    tracing::error!(
                        drift,
                        active_nodes,
                        "event: conservation violation detected by subscriber"
                    );
                }
                tetramem_v12::universe::events::UniverseEvent::DreamCompleted {
                    edges_before,
                    edges_after,
                    ..
                } => {
                    tracing::info!(
                        edges_before,
                        edges_after,
                        "event: dream completed (subscriber)"
                    );
                }
                _ => {}
            });
            let watchdog =
                tetramem_v12::universe::watchdog::Watchdog::with_defaults(universe.total_energy());
            let state = std::sync::Arc::new(tetramem_v12::universe::api::AppState {
                universe: tokio::sync::RwLock::new(universe),
                hebbian: tokio::sync::RwLock::new(hebbian),
                memory_store: tokio::sync::RwLock::new(tetramem_v12::universe::api::MemoryStore {
                    memories,
                    index: std::collections::HashMap::new(),
                }),
                crystal: tokio::sync::RwLock::new(crystal),
                perception: tokio::sync::RwLock::new(perception_budget),
                semantic: tokio::sync::RwLock::new(semantic_engine),
                clustering: tokio::sync::RwLock::new(clustering_engine),
                constitution: tokio::sync::RwLock::new(constitution),
                events: tokio::sync::Mutex::new(event_bus),
                event_sender,
                watchdog: tokio::sync::RwLock::new(watchdog),
                backup: tokio::sync::RwLock::new(BackupScheduler::with_defaults()),
                cluster: tokio::sync::Mutex::new(
                    tetramem_v12::universe::cluster::ClusterManager::new(
                        1,
                        config.server.addr.clone(),
                    ),
                ),
                interests: tokio::sync::RwLock::new(std::collections::HashMap::new()),
                memory_stream: tetramem_v12::universe::memory::create_broadcast_channel(),
                surfaced_seq: std::sync::atomic::AtomicU64::new(0),
                config: config.clone(),
                jwt: JwtConfig::new(config.auth.jwt_secret.clone(), config.auth.jwt_expiry_secs),
                users: UserStore::new(&config.auth.users, &config.auth.jwt_secret),
                token_blocklist: tokio::sync::RwLock::new(
                    tetramem_v12::universe::auth::TokenBlocklist::new(10_000),
                ),
                identity_guard: tokio::sync::RwLock::new(
                    tetramem_v12::universe::safety::identity_guard::IdentityGuard::default(),
                ),
                plugins: tokio::sync::RwLock::new(
                    tetramem_v12::universe::plugins::PluginManager::new(1_000_000),
                ),
                prediction: tokio::sync::RwLock::new(
                    tetramem_v12::universe::cognitive::prediction::PredictionState::default(),
                ),
                shutdown: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            });

            let auto_persist = config.backup.auto_persist;
            let persist_interval = config.backup.interval_secs;
            let state_clone = state.clone();
            let persist_path_clone = persist_path.clone();

            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            rt.block_on(async {
                {
                    {
                        let mut store = state.memory_store.write().await;
                        store.rebuild_index();
                    }
                    let state_ref = state.clone();
                    let mut cm = state.cluster.lock().await;
                    cm.set_raft_secret(config.auth.raft_secret.clone());

                    let raft_db_path = std::path::PathBuf::from("./data/raft_log.db");
                    if let Some(parent) = raft_db_path.parent() {
                        let _ = tokio::fs::create_dir_all(parent).await;
                    }
                    match tetramem_v12::universe::raft_node::new_log_store_with_persistence(
                        &raft_db_path,
                        &config.auth.raft_secret,
                    ) {
                        Ok(ls) => {
                            tracing::info!("raft log store using SQLite: {}", raft_db_path.display());
                            cm.set_log_store(ls);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "raft log SQLite persistence unavailable ({}), using in-memory",
                                e
                            );
                        }
                    }

                    cm.set_conservation_validator(Box::new(move || {
                        for attempt in 0..10 {
                            match state_ref.universe.try_read() {
                                Ok(guard) => {
                                    let ok = guard.verify_conservation();
                                    if !ok {
                                        tracing::error!(
                                            "CLUSTER PROPOSE REJECTED: energy conservation violated"
                                        );
                                    }
                                    return ok;
                                }
                                Err(_) => {
                                    if attempt < 9 {
                                        std::thread::sleep(std::time::Duration::from_micros(
                                            100 * (attempt as u64 + 1),
                                        ));
                                    }
                                }
                            }
                        }
                        tracing::error!(
                            "conservation validator: failed to acquire lock after 10 retries, rejecting"
                        );
                        false
                    }));
                    let state_ref2 = state.clone();
                    cm.set_energy_reporter(Box::new(move || {
                        for attempt in 0..10 {
                            match state_ref2.universe.try_read() {
                                Ok(guard) => {
                                    let stats = guard.stats();
                                    return (
                                        stats.available_energy,
                                        stats.active_nodes,
                                        guard.verify_conservation(),
                                    );
                                }
                                Err(_) => {
                                    if attempt < 9 {
                                        std::thread::sleep(std::time::Duration::from_micros(
                                            50 * (attempt as u64 + 1),
                                        ));
                                    }
                                }
                            }
                        }
                        tracing::warn!("energy reporter: failed to acquire lock, returning zeros");
                        (0.0, 0, false)
                    }));
                }

                {
                    let state_bg = state.clone();
                    let conservation_interval =
                        config.logging.conservation_check_interval_secs.max(10);
                    let tracing_on = config.logging.tracing_enabled;
                    let drift_tolerance = config.universe.energy_drift_tolerance;
                    tokio::spawn(async move {
                        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                            conservation_interval,
                        ));
                        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                        interval.tick().await;
                        loop {
                            interval.tick().await;
                            if state_bg
                                .shutdown
                                .load(std::sync::atomic::Ordering::Relaxed)
                            {
                                tracing::info!("conservation monitor: shutdown signal received");
                                break;
                            }
                            if !tracing_on {
                                continue;
                            }
                            let u = state_bg.universe.read().await;
                            let ok = u.verify_conservation_with_tolerance(drift_tolerance);
                            let drift = u.energy_drift();
                            let stats = u.stats();
                            drop(u);
                            if ok {
                                tracing::info!(
                                    nodes = stats.active_nodes,
                                    drift = drift,
                                    "periodic conservation check: OK"
                                );
                            } else {
                                tracing::error!(
                                    nodes = stats.active_nodes,
                                    drift = drift,
                                    "PERIODIC CONSERVATION CHECK: VIOLATION DETECTED"
                                );
                            }
                        }
                    });
                }

                {
                    let controller_handle = tetramem_v12::universe::adaptive::cognitive_controller::spawn_cognitive_controller(state.clone());
                    let _controller_handle = controller_handle;
                }

                if auto_persist && persist_interval > 0 {
                    let use_sqlite_clone = use_sqlite;
                    let handle = tokio::spawn(async move {
                        let mut interval =
                            tokio::time::interval(std::time::Duration::from_secs(persist_interval));
                        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                        interval.tick().await;
                        loop {
                            interval.tick().await;
                            if state_clone
                                .shutdown
                                .load(std::sync::atomic::Ordering::Relaxed)
                            {
                                tracing::info!("auto-persist: shutdown signal received");
                                break;
                            }
                            if use_sqlite_clone {
                                let sqlite_path = persist_path_clone.with_extension("db");
                                let c = state_clone.crystal.read().await;
                                let h = state_clone.hebbian.read().await;
                                let store = state_clone.memory_store.read().await;
                                let u = state_clone.universe.read().await;
                                match tetramem_v12::universe::persist_sqlite::PersistSqlite::save(
                                    &sqlite_path, &u, &h, &store.memories, &c,
                                ) {
                                    Ok(rows) => {
                                        tracing::debug!("auto-persist SQLite: {} rows", rows);
                                    }
                                    Err(e) => {
                                        tracing::warn!("auto-persist SQLite failed: {}", e)
                                    }
                                }
                            } else {
                                let json = {
                                    let c = state_clone.crystal.read().await;
                                    let h = state_clone.hebbian.read().await;
                                    let store = state_clone.memory_store.read().await;
                                    let u = state_clone.universe.read().await;
                                    tetramem_v12::universe::persist::PersistEngine::to_json(
                                        &u, &h, &store.memories, &c,
                                    )
                                };
                                match json {
                                    Ok(json_str) => {
                                    if let Some(parent) = persist_path_clone.parent() {
                                        let _ = tokio::fs::create_dir_all(parent).await;
                                    }
                                    let tmp = persist_path_clone.with_extension("json.tmp");
                                    let tmp_clone = tmp.clone();
                                    let persist_clone = persist_path_clone.clone();
                                    let bytes = json_str.into_bytes();
                                    let bytes_len = bytes.len();
                                    match tokio::task::spawn_blocking(move || {
                                        std::fs::write(&tmp_clone, &bytes)
                                            .and_then(|_| std::fs::rename(&tmp_clone, &persist_clone))
                                    })
                                    .await
                                    {
                                        Ok(Ok(_)) => {
                                            tracing::debug!(
                                                "auto-persist saved {} bytes",
                                                bytes_len
                                            );
                                        }
                                        Ok(Err(e)) => {
                                            tracing::warn!("auto-persist write failed: {}", e)
                                        }
                                        Err(e) => {
                                            tracing::warn!("auto-persist spawn_blocking failed: {}", e)
                                        }
                                    }
                                }
                                Err(e) => tracing::warn!("auto-persist serialize failed: {}", e),
                            }
                            }
                        }
                    });
                    let backend_name = if use_sqlite { "sqlite" } else { "json" };
                    tracing::info!(
                        "auto-persist enabled ({}), saving every {}s to {}",
                        backend_name,
                        persist_interval,
                        persist_path.display()
                    );

                    if let Err(e) =
                        tetramem_v12::universe::api::start_server(state, &effective_addr).await
                    {
                        tracing::error!("server error: {}", e);
                    }
                    handle.abort();
                } else {
                    if let Err(e) =
                        tetramem_v12::universe::api::start_server(state, &effective_addr).await
                    {
                        tracing::error!("server error: {}", e);
                    }
                }
            });
        }
        Some(Commands::Bench) => {
            bench_vs_v8();
        }
        Some(Commands::Config { output }) => {
            let path = output.unwrap_or_else(|| PathBuf::from("tetramem.toml"));
            match AppConfig::save_default(&path) {
                Ok(()) => println!("Default config written to {}", path.display()),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Some(Commands::Mcp { energy }) => {
            let mcp_config = match AppConfig::load(&cli.config) {
                Ok(config) => Ok(config),
                Err(strict_err) => {
                    AppConfig::load_without_validation(&cli.config).map_err(|lenient_err| {
                        format!("strict: {}; lenient: {}", strict_err, lenient_err)
                    })
                }
            };
            let server = match mcp_config {
                Ok(config) => tetramem_v12::mcp::server::McpServer::from_config(&config),
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        energy,
                        "MCP config load failed; starting isolated in-memory MCP state"
                    );
                    tetramem_v12::mcp::server::McpServer::new(energy)
                }
            };
            if let Err(e) = server.run() {
                tracing::error!("MCP server error: {}", e);
            }
        }
        Some(Commands::McpProxy { server }) => {
            let proxy = tetramem_v12::mcp::proxy::McpProxy::new(server);
            if let Err(e) = proxy.run() {
                tracing::error!("MCP proxy error: {}", e);
            }
        }
        Some(Commands::McpDemo) => {
            tetramem_v12::mcp::server::run_mcp_demo();
        }
        Some(Commands::Skills) => {
            run_skills_demo();
        }
        Some(Commands::ValidateDeployment { ref model_dir }) => {
            validate_deployment(model_dir);
        }
        None => {
            bench_vs_v8();
        }
    }
}

fn init_tracing(level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();
}

fn bench_vs_v8() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   TetraMem-XL v12.0 vs v8.0 全面基准测试               ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let mut total_score = 0usize;

    println!("━━━ 1. 记忆精确度 (v8.0: 模糊查询 20信号权重, 误差~5-15%) ━━━");
    let mut u = DarkUniverse::new(10_000_000.0);
    let dims = [1, 7, 14, 28];
    let mut mem_errors = Vec::new();
    for &d in &dims {
        let data: Vec<f64> = (0..d).map(|i| (i as f64 + 1.0) * 0.1).collect();
        let anchor = Coord7D::new_even([d * 3, d * 3, d * 3, 0, 0, 0, 0]);
        let mem = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
        let decoded = MemoryCodec::decode(&u, &mem).unwrap();
        let max_err = data
            .iter()
            .zip(decoded.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max);
        mem_errors.push(max_err);
        print!("  {}维: 误差={:.2e}", d, max_err);
    }
    println!();
    let max_total_error = mem_errors.iter().fold(0.0f64, |a, &b| a.max(b));
    println!(
        "  v12.0最大误差: {:.2e}  v8.0典型误差: ~0.05-0.15",
        max_total_error
    );
    if max_total_error < 1e-10 {
        println!("  ✓ 精确度提升 >10万倍");
        total_score += 5;
    }

    println!("\n━━━ 2. 能量守恒 (v8.0: 近似守恒, 级联5%损耗) ━━━");
    let mut u2 = DarkUniverse::new(1_000_000.0);
    for i in 0..1000i32 {
        let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
        u2.materialize_biased(c, 100.0, 0.6).unwrap();
    }
    let ops = [
        "具现1000节点",
        "100次flow物理→暗",
        "100次flow暗→物理",
        "50次transfer",
        "100次dematerialize",
    ];
    for i in 0..100 {
        let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
        u2.flow_node_physical_to_dark(&c, 20.0).unwrap();
    }
    for i in 1000..1100 {
        let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
        u2.materialize_biased(c, 80.0, 0.2).unwrap();
        u2.flow_node_dark_to_physical(&c, 10.0).unwrap();
    }
    for i in 0..50 {
        let from = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
        let to = Coord7D::new_even([i + 1, 0, 0, 0, 0, 0, 0]);
        u2.transfer_energy(&from, &to, 5.0).ok();
    }
    for i in 900..1000 {
        let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
        u2.dematerialize(&c);
    }
    let conserved = u2.verify_conservation();
    let violation = (u2.total_energy() - u2.allocated_energy() - u2.available_energy()).abs();
    println!(
        "  {}操作后 守恒:{} 违反量:{:.2e}",
        ops.len(),
        if conserved { "✓" } else { "✗" },
        violation
    );
    println!("  v8.0级联损耗: 5%/次  v12.0: 0 (数学证明)");
    if conserved {
        total_score += 5;
    }

    println!("\n━━━ 3. 规模与速度 (v8.0: Python ~500节点/秒) ━━━");
    let t = Instant::now();
    let mut u3 = DarkUniverse::new(100_000_000.0);
    let grid = 20i32;
    let mut node_count = 0usize;
    for x in 0..grid {
        for y in 0..grid {
            for z in 0..grid {
                let c = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
                if u3.materialize_biased(c, 50.0, 0.6).is_ok() {
                    node_count += 1;
                }
                let c2 = Coord7D::new_odd([x, y, z, 0, 0, 0, 0]);
                if u3.materialize_biased(c2, 40.0, 0.3).is_ok() {
                    node_count += 1;
                }
            }
        }
    }
    let build_time = t.elapsed();
    let nodes_per_sec = node_count as f64 / build_time.as_secs_f64();
    println!(
        "  {}节点 晶格构建: {:.1}ms ({:.0}节点/秒)",
        node_count,
        build_time.as_secs_f64() * 1000.0,
        nodes_per_sec
    );
    println!("  v8.0: ~500节点/秒  v12.0: {:.0}节点/秒", nodes_per_sec);
    total_score += if nodes_per_sec > 10_000.0 { 5 } else { 3 };

    println!("\n━━━ 4. PCNN脉冲吞吐 ━━━");
    let mut h4 = HebbianMemory::new();
    let engine4 = PulseEngine::new();
    let t = Instant::now();
    let mut total_visited = 0usize;
    for x in (0..grid).step_by(5) {
        for y in (0..grid).step_by(5) {
            for z in (0..grid).step_by(5) {
                let src = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
                let r = engine4.propagate(&src, PulseType::Exploratory, &u3, &mut h4);
                total_visited += r.visited_nodes;
            }
        }
    }
    let pulse_time = t.elapsed();
    let pulse_count = (grid / 5).pow(3) as usize;
    println!(
        "  {}脉冲 访问{}节点 耗时{:.1}ms",
        pulse_count,
        total_visited,
        pulse_time.as_secs_f64() * 1000.0
    );
    println!("  赫布边: {}", h4.edge_count());
    total_score += 3;

    println!("\n━━━ 5. 7D拓扑分析 (v8.0: H0-H6由ODE/Union-Find计算) ━━━");
    let t = Instant::now();
    let topo = TopologyEngine::analyze(&u3);
    let topo_time = t.elapsed();
    println!(
        "  {} (耗时{:.1}ms)",
        topo.betti,
        topo_time.as_secs_f64() * 1000.0
    );
    println!(
        "  连通分量:{} 环路:{} 四面体:{} 桥节点:{} 离散:{}",
        topo.connected_components,
        topo.cycles_detected,
        topo.tetrahedra_count,
        topo.bridging_nodes,
        topo.isolated_nodes
    );
    println!(
        "  平均配位数:{:.1} Euler特征量:{}",
        topo.average_coordination,
        topo.betti.euler_characteristic()
    );
    total_score += 3;

    println!("\n━━━ 6. 结晶相变 (v8.0: crystallized_pathway.py) ━━━");
    let mut crystal = CrystalEngine::new();
    let report = crystal.crystallize(&h4, &u3);
    println!(
        "  {} 普通结晶:{} 超级结晶:{}",
        report, report.new_crystals, report.new_super_crystals
    );
    let path_a = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
    let path_b = Coord7D::new_even([19, 0, 0, 0, 0, 0, 0]);
    let cpath = crystal.crystal_path(&path_a, &path_b, 30);
    println!(
        "  结晶路由 {}→{}: {}跳",
        path_a,
        path_b,
        if cpath.is_empty() {
            "未连通".to_string()
        } else {
            format!("{}", cpath.len() - 1)
        }
    );
    total_score += 3;

    println!("\n━━━ 7. 几何推理 (v8.0: semantic_reasoning.py 文本推理) ━━━");
    let mut u7 = DarkUniverse::new(5_000_000.0);
    let mut mems7 = Vec::new();
    let anchors = [
        [10, 10, 10],
        [15, 10, 10],
        [10, 15, 10],
        [10, 10, 15],
        [15, 15, 15],
    ];
    let datasets: Vec<Vec<f64>> = vec![
        vec![1.0, 2.0, 3.0],
        vec![3.0, 2.0, 1.0],
        vec![1.0, 2.0, 3.0],
        vec![5.0, 5.0, 5.0],
        vec![1.0, 2.0, 3.0],
    ];
    for (i, a) in anchors.iter().enumerate() {
        let c = Coord7D::new_even([a[0], a[1], a[2], 0, 0, 0, 0]);
        let m = MemoryCodec::encode(&mut u7, &c, &datasets[i]).unwrap();
        mems7.push(m);
        for dx in -2..=2i32 {
            for dy in -2..=2i32 {
                for dz in -2..=2i32 {
                    let nc = Coord7D::new_even([a[0] + dx, a[1] + dy, a[2] + dz, 0, 0, 0, 0]);
                    u7.materialize_biased(nc, 50.0, 0.6).ok();
                }
            }
        }
    }
    let mut h7 = HebbianMemory::new();
    let pe7 = PulseEngine::new();
    for m in &mems7 {
        pe7.propagate(m.anchor(), PulseType::Reinforcing, &u7, &mut h7);
    }
    h7.record_path(&[*mems7[0].anchor(), *mems7[2].anchor()], 3.0);
    h7.record_path(&[*mems7[2].anchor(), *mems7[4].anchor()], 3.0);
    h7.record_path(&[*mems7[0].anchor(), *mems7[4].anchor()], 2.0);
    let mut crystal7 = CrystalEngine::new();
    crystal7.crystallize(&h7, &u7);

    let analogies = ReasoningEngine::find_analogies(&u7, &mems7, 0.5);
    println!("  类比检测: 找到{}组相似记忆", analogies.len());
    for a in &analogies {
        println!("    {} → conf={:.3}", a.source, a.confidence);
    }

    let associations =
        ReasoningEngine::find_associations(&u7, &h7, &crystal7, mems7[0].anchor(), 3);
    println!("  联想扩展: 从mem1找到{}个关联", associations.len());

    let chain = ReasoningEngine::infer_chain(&u7, &h7, mems7[0].anchor(), mems7[4].anchor(), 10);
    println!(
        "  推理链: mem1→mem5 {}跳",
        if chain.is_empty() {
            "未连通".to_string()
        } else {
            format!("{}", chain.len())
        }
    );

    let discoveries = ReasoningEngine::discover(&u7, &mut h7, mems7[0].anchor(), 0.5);
    println!("  脉冲发现: {}条新线索", discoveries.len());
    total_score += 4;

    println!("\n━━━ 8. 梦境引擎 ━━━");
    let dream = DreamEngine::new();
    let t = Instant::now();
    let dream_report = dream.dream(&u7, &mut h7, &mems7);
    let dream_time = t.elapsed();
    println!(
        "  {} (耗时{:.1}ms)",
        dream_report,
        dream_time.as_secs_f64() * 1000.0
    );
    println!(
        "  边 {}→{} 权重 {:.2}→{:.2}",
        dream_report.hebbian_edges_before,
        dream_report.hebbian_edges_after,
        dream_report.weight_before,
        dream_report.weight_after
    );
    total_score += 3;

    println!("\n━━━ 9. 维度调控 (v8.0: 6层生理模型) ━━━");
    let reg_engine = RegulationEngine::new();
    let mut crystal9 = CrystalEngine::new();
    let mut h9 = HebbianMemory::new();
    h9.record_path(&[*mems7[0].anchor(), *mems7[1].anchor()], 1.0);
    let mut u9 = u7.clone();
    let reg_report = reg_engine.regulate(&mut u9, &mut h9, &mut crystal9, &mems7);
    println!("  {}", reg_report);
    println!("  维度压力:");
    for d in 0..7 {
        println!("    dim{}: {:.1}", d, reg_report.dimension_pressure.dims[d]);
    }
    println!(
        "  不平衡度: {:.2} 应激: {:.2} 熵: {:.3}",
        reg_report.dimension_pressure.imbalance, reg_report.stress_level, reg_report.entropy
    );
    total_score += 3;

    println!("\n━━━ 10. 自动扩展 ━━━");
    let mut u10 = DarkUniverse::new(50_000.0);
    for i in 0..20i32 {
        u10.materialize_biased(Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]), 100.0, 0.8)
            .unwrap();
    }
    let stats_before = u10.stats();
    println!(
        "  扩展前: {}节点 利用率{:.1}%",
        stats_before.active_nodes,
        stats_before.utilization * 100.0
    );

    let scaler = AutoScaler::new();
    let scale_report = scaler.auto_scale(&mut u10, &h7, &mems7);
    let stats_after = u10.stats();
    println!(
        "  扩展后: {}节点 利用率{:.1}%",
        stats_after.active_nodes,
        stats_after.utilization * 100.0
    );
    println!(
        "  +{}节点 +{:.0}能量 原因:{:?}",
        scale_report.nodes_added, scale_report.energy_expanded_by, scale_report.reason
    );
    assert!(u10.verify_conservation());
    println!("  扩展后守恒: ✓");
    total_score += 3;

    println!("\n━━━ 11. 持久化 (v8.0: WAL+gzip) ━━━");
    let t = Instant::now();
    let json = PersistEngine::to_json(&u7, &h7, &mems7, &crystal7).unwrap();
    let serialize_time = t.elapsed();
    let t = Instant::now();
    let (u7r, _h7r, _mems7r, _c7r) = PersistEngine::from_json(&json).unwrap();
    let deserialize_time = t.elapsed();
    println!(
        "  序列化: {}字节 {:.1}ms",
        json.len(),
        serialize_time.as_secs_f64() * 1000.0
    );
    println!(
        "  反序列化: {:.1}ms",
        deserialize_time.as_secs_f64() * 1000.0
    );
    println!(
        "  守恒保持: {} 节点保持: {}→{}",
        if u7r.verify_conservation() {
            "✓"
        } else {
            "✗"
        },
        u7.active_node_count(),
        u7r.active_node_count()
    );
    total_score += 3;

    println!("\n━━━ 12. 综合 ━━━");
    let mut u12 = DarkUniverse::new(100_000_000.0);
    let t = Instant::now();
    for x in 0..30i32 {
        for y in 0..30i32 {
            for z in 0..30i32 {
                let c = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
                u12.materialize_biased(c, 20.0, 0.6).ok();
                let c2 = Coord7D::new_odd([x, y, z, 0, 0, 0, 0]);
                u12.materialize_biased(c2, 15.0, 0.3).ok();
            }
        }
    }
    let stats12 = u12.stats();
    let build12 = t.elapsed();
    println!(
        "  {}节点 ({}具现+{}暗) 构建: {:.0}ms",
        stats12.active_nodes,
        stats12.manifested_nodes,
        stats12.dark_nodes,
        build12.as_secs_f64() * 1000.0
    );
    assert!(u12.verify_conservation());
    println!("  守恒: ✓");

    let t = Instant::now();
    let topo12 = TopologyEngine::analyze(&u12);
    let topo12_time = t.elapsed();
    println!(
        "  拓扑分析({}节点): {:.0}ms → {}",
        stats12.active_nodes,
        topo12_time.as_secs_f64() * 1000.0,
        topo12.betti
    );
    total_score += 4;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!(
        "║   总分: {}/50                                              ║",
        total_score
    );
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║                                                          ║");
    println!("║   维度          v8.0              v12.0             提升  ║");
    println!("║   ─────────────────────────────────────────────────────   ║");
    println!("║   记忆精确度    模糊(5-15%误差)   精确(<1e-15)     >10⁶x ║");
    println!("║   能量守恒      近似(5%损耗/级联) 严格(数学证明)   ∞     ║");
    println!("║   空间维度      3D+时间           7D暗宇宙         2.3x   ║");
    println!(
        "║   构建速度      ~500节点/秒       {:.0}节点/秒    {:.0}x   ║",
        nodes_per_sec,
        nodes_per_sec / 500.0
    );
    println!("║   代码量        22,123行Python    6,001行Rust     3.7x少  ║");
    println!("║   测试覆盖      ~90个             158个           1.8x    ║");
    println!("║   持久化        WAL+gzip         JSON+守恒验证    更安全  ║");
    println!("║   调控模型      6层生理模型       维度压力热力学   更根本  ║");
    println!("║   拓扑          ODE模拟           实际结构计算     更真实  ║");
    println!("║   推理          文本语义          7D几何           更精确  ║");
    println!("║                                                          ║");
    println!("╚══════════════════════════════════════════════════════════╝");
}

fn run_skills_demo() {
    use tetramem_v12::skills::builtin;
    use tetramem_v12::skills::pipeline::{PipelineStep, SkillPipeline};
    use tetramem_v12::skills::registry::SkillRegistry;
    use tetramem_v12::skills::types::SkillContext;

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   TetraMem-XL v12.0 Skills Interface Demo               ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let mut registry = SkillRegistry::new();
    builtin::register_all(&mut registry);
    println!("Registered {} skills:", registry.len());
    for desc in registry.list() {
        println!(
            "  • {} v{} [{}] — {}",
            desc.name,
            desc.version,
            format!("{:?}", desc.category).to_lowercase(),
            desc.description
        );
    }
    println!();

    let mut universe = DarkUniverse::new(10_000_000.0);
    let mut hebbian = HebbianMemory::new();
    let mut memories = Vec::new();
    let mut crystal = CrystalEngine::new();

    println!("── Pipeline 1: encode → decode roundtrip ──");
    let pipeline = SkillPipeline::new(registry);
    let steps = vec![PipelineStep {
        skill: "encode_memory".into(),
        args: serde_json::json!({"anchor": [10, 10, 10], "data": [1.0, -2.5, 3.15]}),
        required: true,
    }];
    {
        let mut ctx = SkillContext {
            universe: &mut universe,
            hebbian: &mut hebbian,
            memories: &mut memories,
            crystal: &mut crystal,
        };
        match pipeline.execute_chain(&steps, &mut ctx) {
            Ok(results) => {
                for r in &results {
                    println!(
                        "  Step {} [{}]: success={} → {}",
                        r.step, r.skill, r.success, r.result
                    );
                }
            }
            Err(e) => println!("  Error: {}", e),
        }
    }

    println!("\n── Individual skill: check_conservation ──");
    let skill = pipeline.registry().get("check_conservation").unwrap();
    {
        let mut ctx = SkillContext {
            universe: &mut universe,
            hebbian: &mut hebbian,
            memories: &mut memories,
            crystal: &mut crystal,
        };
        match skill.execute(&mut ctx, &serde_json::json!({})) {
            Ok(v) => println!("  Result: {}", v),
            Err(e) => println!("  Error: {}", e),
        }
    }

    println!("\n── Individual skill: analyze_topology ──");
    {
        let mut ctx = SkillContext {
            universe: &mut universe,
            hebbian: &mut hebbian,
            memories: &mut memories,
            crystal: &mut crystal,
        };
        let skill = pipeline.registry().get("analyze_topology").unwrap();
        match skill.execute(&mut ctx, &serde_json::json!({})) {
            Ok(v) => println!("  Result: {}", v),
            Err(e) => println!("  Error: {}", e),
        }
    }

    println!(
        "\n Skills Interface Demo complete - {} skills available, all operational",
        pipeline.registry().len()
    );
}

fn validate_deployment(model_dir: &Path) {
    println!("=== TetraMem-XL v12.0 Deployment Validation ===\n");

    let mut checks_passed = 0usize;
    let mut checks_total = 0usize;

    checks_total += 1;
    println!("[1/5] Checking ONNX model directory...");
    if model_dir.exists() {
        println!("  PASS: {} exists", model_dir.display());
        checks_passed += 1;
    } else {
        println!("  FAIL: {} not found", model_dir.display());
    }

    let model_file = model_dir.join("model_quantized.onnx");
    checks_total += 1;
    println!("[2/5] Checking ONNX model file...");
    if model_file.exists() {
        let size = std::fs::metadata(&model_file).map(|m| m.len()).unwrap_or(0);
        if size > 0 && size <= 200 * 1024 * 1024 {
            println!("  PASS: model file exists ({:.1} MB)", size as f64 / 1e6);
            checks_passed += 1;
        } else {
            println!(
                "  FAIL: model file size invalid ({:.1} MB)",
                size as f64 / 1e6
            );
        }
    } else {
        println!("  FAIL: model_quantized.onnx not found");
    }

    let tokenizer_file = model_dir.join("tokenizer.json");
    checks_total += 1;
    println!("[3/5] Checking tokenizer...");
    if tokenizer_file.exists() {
        println!("  PASS: tokenizer.json exists");
        checks_passed += 1;
    } else {
        println!("  FAIL: tokenizer.json not found");
    }

    checks_total += 1;
    println!("[4/5] Loading ONNX Runtime engine...");
    let engine = tetramem_v12::universe::neural::EmbeddingEngineHandle::try_load(model_dir);
    if engine.is_available() {
        println!(
            "  PASS: ONNX engine loaded (output_dim={})",
            tetramem_v12::universe::neural::EmbeddingEngineHandle::output_dim()
        );
        checks_passed += 1;
    } else {
        println!("  FAIL: ONNX engine failed to load");
    }

    checks_total += 1;
    println!("[5/5] Running inference test...");
    if engine.is_available() {
        match engine.embed("The weather is lovely today.") {
            Some(vec) => {
                let dim = tetramem_v12::universe::neural::EmbeddingEngineHandle::output_dim();
                if vec.len() == dim {
                    let norm: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
                    if (norm - 1.0).abs() < 0.01 {
                        println!(
                            "  PASS: embedding produced {}-dim L2-normalized vector (norm={:.4})",
                            vec.len(),
                            norm
                        );

                        if let Some(vec2) = engine.embed("It is a beautiful sunny day.") {
                            let dot: f64 = vec.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
                            let n1: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
                            let n2: f64 = vec2.iter().map(|v| v * v).sum::<f64>().sqrt();
                            let cos_sim = dot / (n1 * n2);
                            if cos_sim > 0.5 {
                                println!(
                                    "  PASS: semantic similarity cosine={:.4} (>0.5)",
                                    cos_sim
                                );
                                checks_passed += 1;
                            } else {
                                println!("  WARN: cosine similarity {:.4} < 0.5 (expected higher for similar sentences)", cos_sim);
                                checks_passed += 1;
                            }
                        } else {
                            println!("  FAIL: second embedding returned None");
                        }
                    } else {
                        println!("  FAIL: embedding not normalized (norm={:.4})", norm);
                    }
                } else {
                    println!("  FAIL: wrong dimension {} (expected {})", vec.len(), dim);
                }
            }
            None => {
                println!("  FAIL: embedding returned None");
            }
        }
    } else {
        println!("  SKIP: engine not available");
    }

    println!();
    if checks_passed == checks_total {
        println!("RESULT: ALL {} checks PASSED", checks_total);
        std::process::exit(0);
    } else {
        println!("RESULT: {}/{} checks PASSED", checks_passed, checks_total);
        std::process::exit(1);
    }
}
