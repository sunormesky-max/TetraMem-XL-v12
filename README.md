# TetraMem-XL v12.0

**7D Dark Universe Memory System** — Pure Rust implementation with strict energy conservation as the first principle.

[![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-270-brightgreen.svg)]()
[![Stars](https://img.shields.io/github/stars/sunormesky-max/TetraMem-XL-v12?style=social)](https://github.com/sunormesky-max/TetraMem-XL-v12/stargazers)

## What is TetraMem-XL?

TetraMem-XL v12.0 is a next-generation memory system built on a **7-dimensional dark universe** architecture. The physical 3D BCC lattice emerges as a cross-section/crystallization of this 7D space, governed by a strict energy conservation law that serves as the foundation of the entire system.

### Core Principles

- **Energy Conservation** — Enforced as a resource pool invariant: total energy is strictly tracked across all operations
- **7D Dark Universe** — Independent space where 7 = 3 physical (x, y, z) + 4 dark (E, S, T, μ)
- **Tetrahedron as Minimal Unit** — Each memory is stored in a tetrahedron of 4 nodes in 7D space
- **Offset Encoding** — Arbitrary real numbers (including negatives) stored with high precision (tested to 1e-10 tolerance)
- **Manifestation** — Nodes with physical energy ratio > 0.5 crystallize into the physical lattice
- **H6 Phase Transition** — Two-phase energy quorum consensus for distributed crystallization

## Architecture

```
                        TetraMem-XL v12.0
                              |
    ┌──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┐
    Core      Memory    Cognitive   Adaptive   Storage   Consensus  Interface   Safety
 ┌──┼──┬──┐  ┌──┼──┐   ┌───┼───┐   ┌──┼──┐    ┌──┼──┐    ┌──┼──┐    ┌──┼──┐    ┌──┼──┐
coord energy  memory hebbian crystal topology autoscale observer persist cluster  api auth  consti events
node lattice  pulse  dream   reasoning perception regulation watchdog persist_file raft_node metrics
config                       emotion  agent                persist_sqlite backup
                             perception
```

## Module Layers (8 layers, 38 modules)

| Layer | Modules | Description |
|-------|---------|-------------|
| **Core** | `coord`, `energy`, `node`, `lattice`, `config` | 7D coordinates, energy fields, BCC lattice, conservation |
| **Memory** | `memory`, `hebbian`, `pulse`, `dream` | Encoding, Hebbian learning, pulse propagation, dream consolidation |
| **Cognitive** | `crystal`, `topology`, `reasoning`, `perception`, `emotion`, `agent` | Crystallization, topology analysis, analogy, emotion mapping, agents |
| **Adaptive** | `autoscale`, `observer`, `regulation`, `watchdog` | Auto-scaling, health monitoring, regulation, recovery |
| **Storage** | `persist`, `persist_file`, `persist_sqlite`, `backup` | JSON/SQLite persistence, generational backup |
| **Consensus** | `cluster`, `raft_node` | Raft-based cluster management, energy quorum |
| **Interface** | `api`, `auth`, `metrics` | REST API (35 endpoints), JWT auth, Prometheus metrics |
| **Safety** | `constitution`, `events` | Behavioral rules, event bus |

## Quick Start

### Build & Test

```bash
cargo build --release

cargo test                          # 255 unit tests
cargo test --test api_integration   # 15 HTTP integration tests
cargo test --test full_suite        # integration suite
cargo test --test stress_test       # extreme stress tests
```

### REST API Server

```bash
cargo run --release serve                    # default: 127.0.0.1:3456
cargo run --release serve 0.0.0.0:8080       # custom address
cargo run --release -- --config config.toml serve
```

#### API Endpoints (35 total)

**Health & Stats**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Universe health check |
| GET | `/stats` | Universe statistics (energy, drift, nodes) |
| GET | `/metrics` | Prometheus metrics export |
| GET | `/openapi.json` | OpenAPI 3.0 spec |
| POST | `/login` | Obtain JWT token |

**Memory Operations**
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/memory/encode` | Encode memory at 3D anchor |
| POST | `/memory/decode` | Decode memory from 3D anchor |
| GET | `/memory/list` | List all memories |
| GET | `/memory/timeline` | Memory timeline by date |
| POST | `/memory/trace` | Trace memory associations via Hebbian |

**Cognitive Operations**
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/pulse` | Fire pulse (3 types: reinforcing, exploratory, cascade) |
| POST | `/dream` | Run dream cycle (replay, weaken, consolidate) |
| POST | `/regulate` | Run regulation cycle |

**7D Dark Dimension** (full 7D coordinate operations)
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/dark/query` | Query node's full 7D energy state |
| POST | `/dark/flow` | Transfer energy between physical/dark dimensions |
| POST | `/dark/transfer` | Transfer energy between two 7D nodes |
| POST | `/dark/materialize` | Materialize node at full 7D coordinates |
| POST | `/dark/dematerialize` | Dematerialize node, recover energy |
| GET | `/dark/pressure` | Dimension pressure/entropy across all 7 dims |

**Scaling**
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/scale` | Auto-scale universe |
| POST | `/scale/frontier/{max_new}` | Frontier expansion |
| GET | `/hebbian/neighbors/{x}/{y}/{z}` | Get Hebbian neighbors |

**Backup**
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/backup/create` | Create manual backup |
| GET | `/backup/list` | List backups |

**Cluster (Raft)**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/cluster/status` | Cluster status |
| POST | `/cluster/init` | Initialize cluster |
| POST | `/cluster/propose` | Propose Raft command |
| POST | `/cluster/add-node` | Add cluster node |
| POST | `/cluster/remove-node` | Remove cluster node |

**Phase Transition & Quorum**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/phase/detect` | Detect phase transitions |
| POST | `/phase/consensus` | Phase consensus proposal |
| POST | `/phase/quorum/start` | Start energy quorum |
| POST | `/phase/quorum/confirm` | Confirm quorum entry |
| GET | `/phase/quorum/status` | Quorum status |
| POST | `/phase/quorum/execute` | Execute quorum decision |

### Usage Example

```rust
use tetramem_v12::universe::*;

let mut universe = DarkUniverse::new(10_000_000.0);

let anchor = Coord7D::new_even([100, 100, 100, 0, 0, 0, 0]);
let data = vec![1.0, -2.5, 3.14, 0.0, 42.0, -1e-10, 999.999];
let memory = MemoryCodec::encode(&mut universe, &anchor, &data).unwrap();

let decoded = MemoryCodec::decode(&universe, &memory).unwrap();
assert_eq!(data.len(), decoded.len());

assert!(universe.verify_conservation());
```

## v12.0 vs v8.0 Comparison

| Metric | v8.0 (Python) | v12.0 (Rust) | Improvement |
|--------|---------------|--------------|-------------|
| Memory Precision | 5-15% error | < 1e-10 drift | > 10^9x |
| Build Speed | ~500 nodes/s | ~4.5M nodes/s | 8,916x |
| Energy Conservation | ~5% loss per cascade | Resource pool invariant (enforced) | ∞ |
| Dimensions | 3D + time | 7D dark universe | 2.3x |
| Code Size | 22,123 lines | ~9,000 lines | 2.5x less |
| Tests | ~90 | 270 (255 unit + 15 integration) | 3x |

## Test Coverage

- **255 unit tests** — per-module correctness across all 8 layers
- **15 HTTP integration tests** — full endpoint coverage via axum test harness
- **3 stress/scale test suites** — 10K+ nodes, million ops, extreme conditions

### Notable Stress Test Results

| Test | Operation | Result |
|------|-----------|--------|
| S1 | 10K nodes × 100K operations | 12ms, 0 violations |
| S2 | 10K memories flood | precision within 1e-10 |
| S3 | 1M decode operations | 189ms |
| S11 | 50K chaotic operations | 0 corruption |
| S12 | 20K longevity rounds | 0 degradation |

## Security Hardening

12 rounds of security audit and hardening completed:

- **Authentication**: JWT (HS256 strict, iss/aud/nbf/jti UUID v4, 24h expiry), Argon2id passwords, timing-safe dummy hash
- **Authorization**: 4-tier RBAC (public → raft → user → admin), Claims with private fields + accessors
- **Rate Limiting**: SeqCst atomic counter, per-IP login rate limit (10 attempts/5min, 10K entry cap)
- **Security Headers**: CSP, X-Frame-Options DENY, X-Content-Type-Options, Referrer-Policy, Permissions-Policy, Cache-Control
- **CORS**: Specific methods (GET/POST/OPTIONS) + specific headers, no `Any` for methods/headers
- **Constant-time**: `subtle` crate for Raft secret comparison
- **Input Validation**: Coordinate bounds, data magnitude limits, path traversal protection
- **Integrity**: HMAC-SHA256 for Raft log entries, SHA-256 for backup snapshots
- **Persistence**: SQLite synchronous=FULL, file permissions 0600, atomic writes with temp file cleanup
- **Concurrency**: Fine-grained RwLock per resource (no global write lock), O(1) memory lookup via HashMap index
- **Docker**: Non-root user, no-new-privileges, cap_drop ALL, read-only filesystem, resource limits
- **CI**: GitHub Actions SHA-pinned, permissions: contents:read, --locked on all cargo commands

## Dependencies

- [axum](https://crates.io/crates/axum) 0.7 — HTTP framework
- [tokio](https://crates.io/crates/tokio) 1 — Async runtime
- [serde](https://crates.io/crates/serde) 1 — Serialization
- [rusqlite](https://crates.io/crates/rusqlite) 0.31 — SQLite (bundled)
- [openraft](https://crates.io/crates/openraft) 0.10 — Raft consensus
- [tower-http](https://crates.io/crates/tower-http) 0.6 — Middleware
- [jsonwebtoken](https://crates.io/crates/jsonwebtoken) 9 — JWT auth
- [subtle](https://crates.io/crates/subtle) 2 — Constant-time comparisons
- [prometheus](https://crates.io/crates/prometheus) 0.13 — Metrics

## License

[GNU Affero General Public License v3.0 or later](LICENSE)

## Author

**sunormesky-max (Liu Qihang)** — sunormesky@gmail.com
