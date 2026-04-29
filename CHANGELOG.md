# Changelog

All notable changes to TetraMem-XL v12.0 are documented here.

## [12.0.0] - 2026-04-30

### Core System
- **7D Dark Universe** — Full 7-dimensional space (3 physical + 4 dark dimensions)
- **Energy Conservation** — Mathematically guaranteed zero-loss across all operations
- **MemoryCodec** — Encode/decode 1-28 dimensions with precision < 1e-14
- **BCC Lattice** — Body-centered cubic lattice emerging from 7D space
- **Hebbian Learning** — Path recording, bias, decay, pruning
- **PCNN Pulse Engine** — BFS propagation with 3 pulse types (reinforcing, exploratory, noise)
- **Dream Engine** — 3-phase cycle (replay, weaken, consolidate)
- **Crystallization** — Phase transition with super channels
- **Topology** — Betti numbers H0-H6, Euler characteristic, shortest paths
- **Reasoning** — Analogy, association, inference chains, discovery

### Infrastructure
- **REST API** — 26 endpoints (axum) with JWT auth and rate limiting
- **Dual Persistence** — JSON snapshots + SQLite with indexed schema
- **Cluster** — Raft-based consensus (openraft) with H6EnergyQuorum two-phase protocol
- **Auto-scaling** — 5 strategies + scale-to-fit-memory
- **Regulation** — Dimension pressure thermodynamics + stress response
- **Backup** — Generational rotation with scheduled backups
- **Watchdog** — 4-level watermarks + auto-recovery
- **Perception Budget** — Topology-weighted energy allocation
- **Config** — TOML configuration with full validation (NaN, zero, range checks)
- **Metrics** — Prometheus-compatible metrics

### Security Audit Fixes (Critical)
- **Deadlock Fix** — `conservation_validator` and `energy_reporter` switched from `blocking_lock()` to `try_lock()`, eliminating self-deadlock and AB/BA lock inversion between cluster and universe mutexes
- **Crystal Data Loss Fix** — Backup creation and shutdown persist now use live `state.crystal` instead of empty `CrystalEngine::new()`
- **Checksum Enforcement** — Persist checksum mismatch now rejects the snapshot (`Err`) instead of just logging a warning; checksum includes node dims, edge coordinates, and channel coordinates
- **Login Validation** — Password must be ≥ 8 characters
- **HTTP 404** — Memory not found returns proper 404 status code
- **Config Validation** — NaN detection, zero-interval rejection, manifestation threshold range [0,1], all numeric fields validated at startup

### Performance Fixes
- **Trace O(n*m) → O(n+m)** — `memory_trace` endpoint uses HashMap index instead of linear string scan
- **SQLite Indexes** — Added indexes on hebbian_edges, memories (anchor), and crystal_channels
- **SQLite Timestamps** — `created_at` column preserved in SQLite roundtrip (was silently zeroed)
- **Metrics Test** — Fixed flaky parallel test race with unique gauge value

### Correctness
- **energy_drift_tolerance** — Configurable tolerance now wired into stats, health, background check, and post-restore verification (was dead code)
- **Background Conservation** — Periodic check uses configurable `energy_drift_tolerance` instead of hardcoded 1e-10

### Test Results
- **210 unit tests** — All passing
- **38 integration tests** — All passing (full_suite)
- **8 scalability tests** — All passing (scale_bench)
- **12 stress tests** — All passing (stress_test)

### Frontend
- **Control Panel** — React 19 + TypeScript + Vite 7 + Tailwind + shadcn/ui + Three.js + GSAP
- **10 pages** — Dashboard, Universe, Memory, Pulse, Dream, Topology, Regulation, Cluster, Timeline, API
