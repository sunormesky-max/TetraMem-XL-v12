# TetraMem-XL v12.0

**7D Dark Universe Memory System** — Pure Rust implementation with strict energy conservation as the first principle.

[![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.95.0-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-248-brightgreen.svg)]()

## What is TetraMem-XL?

TetraMem-XL v12.0 is a next-generation memory system built on a **7-dimensional dark universe** architecture. The physical 3D BCC lattice emerges as a cross-section/crystallization of this 7D space, governed by a strict energy conservation law that serves as the foundation of the entire system.

### Core Principles

- **Energy Conservation** — Mathematically proven, zero loss across all operations
- **7D Dark Universe** — Independent space where 7 = 3 physical (x, y, z) + 4 dark (E, S, T, μ)
- **Tetrahedron as Minimal Unit** — Each memory is stored in a tetrahedron of 4 nodes in 7D space
- **Offset Encoding** — Arbitrary real numbers (including negatives) stored with precision < 1e-14
- **Manifestation** — Nodes with physical energy ratio > 0.5 crystallize into the physical lattice
- **H6 Phase Transition** — Two-phase energy quorum consensus for distributed crystallization

## Architecture

```
                    TetraMem-XL v12.0
                         |
        ┌────────────────┼────────────────┐
        |                |                |
    Core Layer      Memory Layer     Cognitive Layer
   ┌──┼──┬──┐    ┌────┼────┐     ┌──────┼──────┐
   coord  energy  memory  hebbian  crystal  reasoning
   node   lattice  pulse  dream   topology
   config                        perception
                         |
               ┌─────────┼─────────┐
               |         |         |
          Adaptive    Safety     Interface
          ┌──┼──┐   ┌──┼──┐      |
        autoscale  observer backup watchdog   API (axum REST)
        regulation persist  raft_node         26 endpoints
                   cluster
```

## Modules (22 total)

| Module | Description |
|--------|-------------|
| `coord` | 7D coordinate system with Even/Odd parity |
| `energy` | 7D energy fields, pool management, flow/split, drift measurement |
| `lattice` | BCC lattice, tetrahedra, 3-layer neighbor shells |
| `memory` | MemoryCodec — encode/decode 1-28 dimensions with timestamps |
| `node` | DarkUniverse core — materialize, protect, conservation |
| `hebbian` | Hebbian learning — path recording, bias, decay, prune |
| `pulse` | PCNN pulse engine — BFS with 3 pulse types |
| `observer` | Universe health monitor (12 metrics) + self-regulator |
| `dream` | 3-phase dream engine — replay, weaken, consolidate |
| `autoscale` | 5 auto-scaling strategies + scale-to-fit-memory |
| `crystal` | Phase transition crystallization + crystal path routing |
| `topology` | Betti numbers H0-H6, Euler characteristic, BFS paths |
| `reasoning` | Analogy, association, inference chains, discovery |
| `perception` | Perception budget with topology weighting |
| `persist` | JSON serialization with enhanced checksum verification |
| `persist_file` | Atomic file persistence with temp-file write |
| `persist_sqlite` | SQLite storage with indexed schema |
| `regulation` | Dimension pressure thermodynamics + stress response |
| `backup` | Scheduled backups with generational rotation |
| `watchdog` | 4-level watermarks + auto-recovery + backup integration |
| `config` | TOML configuration with full validation |
| `api` | axum REST API with 26 endpoints, JWT auth, rate limiting |
| `cluster` | Raft-based cluster management + H6EnergyQuorum consensus |
| `raft_node` | openraft integration (log store, state machine, network) |
| `auth` | JWT token creation and validation |
| `metrics` | Prometheus-compatible metrics |

## Quick Start

### Build & Test

```bash
# Build
cargo build --release

# Run all 210 unit tests + 38 integration tests
cargo test

# Run specific test suites
cargo test --test full_suite    # 38 integration tests
cargo test --test scale_bench   # 8 scalability tests
cargo test --test stress_test   # 12 extreme stress tests
```

### REST API Server

```bash
# Start API server on default address (127.0.0.1:3456)
cargo run --release serve

# Custom address
cargo run --release serve 0.0.0.0:8080

# With config file
cargo run --release -- --config config.toml serve
```

#### API Endpoints (26 total)

**Core CRUD**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Universe health check |
| GET | `/stats` | Universe statistics (incl. energy_drift) |
| POST | `/nodes` | Materialize a node |
| DELETE | `/nodes/{id}` | Dematerialize a node |
| POST | `/memory` | Encode memory |
| GET | `/memory/{id}` | Decode memory |
| DELETE | `/memory/{id}` | Erase memory |
| GET | `/memories` | List all memories |

**Cognitive**
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/pulse` | Fire a pulse |
| POST | `/dream` | Run dream cycle |
| POST | `/topology` | Compute topology (Betti numbers) |
| POST | `/regulate` | Run regulation cycle |

**Backup**
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/backup/create` | Create manual backup |
| GET | `/backup/list` | List backups |

**Cluster**
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/cluster/init` | Initialize cluster |
| GET | `/cluster/status` | Cluster status |
| POST | `/cluster/propose` | Propose command |
| POST | `/cluster/add-node` | Add cluster node |
| POST | `/cluster/remove-node` | Remove cluster node |

**Timeline & Trace**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/memory/timeline` | Memory timeline by date |
| POST | `/memory/trace` | Trace memory associations |

**Phase Transition**
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/phase/detect` | Detect phase transitions |
| POST | `/phase/consensus` | Phase consensus proposal |
| POST | `/phase/quorum/start` | Start energy quorum |
| POST | `/phase/quorum/confirm` | Confirm quorum entry |
| GET | `/phase/quorum/status` | Quorum status |
| POST | `/phase/quorum/execute` | Execute quorum decision |

**Auth & Metrics**
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/login` | Obtain JWT token |
| GET | `/metrics` | Prometheus metrics |

### Usage Example

```rust
use tetramem_v12::universe::*;

let mut universe = DarkUniverse::new(10_000_000.0);

// Encode a 7-dimensional memory
let anchor = Coord7D::new_even([100, 100, 100, 0, 0, 0, 0]);
let data = vec![1.0, -2.5, 3.14, 0.0, 42.0, -1e-10, 999.999];
let memory = MemoryCodec::encode(&mut universe, &anchor, &data).unwrap();

// Decode with precision < 1e-14
let decoded = MemoryCodec::decode(&universe, &memory).unwrap();
assert_eq!(data.len(), decoded.len());

// Energy conservation is mathematically guaranteed
assert!(universe.verify_conservation());
```

## v12.0 vs v8.0 Comparison

| Metric | v8.0 (Python) | v12.0 (Rust) | Improvement |
|--------|---------------|--------------|-------------|
| Memory Precision | 5-15% error | < 1e-14 | > 10^13x |
| Build Speed | ~500 nodes/s | ~4.5M nodes/s | 8,916x |
| Energy Conservation | ~5% loss per cascade | 0 (mathematically proven) | ∞ |
| Dimensions | 3D + time | 7D dark universe | 2.3x |
| Code Size | 22,123 lines | ~9,000 lines | 2.5x less |
| Tests | ~90 | 248 | 2.8x |

## Test Coverage

- **210 unit tests** — per-module correctness
- **38 integration tests** — full pipeline verification
- **8 scalability tests** — 10K+ nodes, 100K+ operations
- **12 stress tests** — extreme conditions (million ops, 20K rounds)

### Notable Stress Test Results

| Test | Operation | Result |
|------|-----------|--------|
| S1 | 10K nodes × 100K operations | 12ms, 0 violations |
| S2 | 10K memories flood | precision 1.42e-14 |
| S3 | 1M decode operations | 189ms |
| S11 | 50K chaotic operations | 0 corruption |
| S12 | 20K longevity rounds | 0 degradation |

## Dependencies

- [axum](https://crates.io/crates/axum) 0.7 — HTTP framework
- [tokio](https://crates.io/crates/tokio) 1 — Async runtime
- [serde](https://crates.io/crates/serde) 1 — Serialization
- [serde_json](https://crates.io/crates/serde_json) 1 — JSON support
- [rusqlite](https://crates.io/crates/rusqlite) 0.31 — SQLite (bundled)
- [openraft](https://crates.io/crates/openraft) 0.10 — Raft consensus
- [tower-http](https://crates.io/crates/tower-http) 0.6 — Middleware
- [jsonwebtoken](https://crates.io/crates/jsonwebtoken) 9 — JWT auth
- [prometheus](https://crates.io/crates/prometheus) 0.13 — Metrics

## License

[GNU Affero General Public License v3.0 or later](LICENSE)

## Author

**sunormesky-max (Liu Qihang)** — sunormesky@gmail.com
