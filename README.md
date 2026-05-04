# TetraMem-XL v12.0

**7D Dark Universe Memory System** — A next-generation memory architecture where physical 3D space emerges as a cross-section of a 7-dimensional dark universe, governed by strict energy conservation as its foundational invariant. Pure Rust, production-grade.

[![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88+-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-399-brightgreen.svg)]()
[![Endpoints](https://img.shields.io/badge/REST-62-blue.svg)]()
[![Stars](https://img.shields.io/github/stars/sunormesky-max/TetraMem-XL-v12?style=social)](https://github.com/sunormesky-max/TetraMem-XL-v12/stargazers)

## Why TetraMem-XL?

Traditional memory systems store data in flat key-value pairs or vector databases. TetraMem-XL takes a fundamentally different approach: **memory is physical structure in a 7-dimensional dark universe**.

Each memory atom occupies a tetrahedron of 4 nodes in 7D space. The physical 3D world we interact with is a crystallized cross-section — nodes whose physical energy ratio exceeds a threshold "manifest" into observable reality. This creates a memory system with:

- **Structural depth** — memories have intrinsic geometric relationships (distance, angle, neighborhood)
- **Emergent topology** — Betti numbers and Euler characteristics arise naturally from memory placement
- **Thermodynamic grounding** — energy conservation is enforced as a resource pool invariant, not approximated
- **Dimensional richness** — 4 dark dimensions (E, S, T, μ) carry latent information invisible to 3D queries

## Architecture

```
                        TetraMem-XL v12.0
                              │
    ┌─────────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┐
    │  Core   │  Memory  │ Cognitive│ Adaptive │ Storage  │Consensus │Interface │  Safety  │
    │ (7D)    │ (6 mod)  │ (6 mod)  │ (4 mod)  │ (4 mod)  │ (3 mod)  │ (14 mod) │ (3 mod)  │
    └────┬────┴────┬─────┴────┬─────┴────┬─────┴────┬─────┴────┬─────┴────┬─────┴────┬─────┘
         │         │          │          │          │          │          │          │
    coord/energy  memory/    crystal/   autoscale/ persist/   cluster/   api/       constitution/
    node/lattice  hebbian/   topology/  observer/  persist_   raft_node/ auth/      events/
    config/       pulse/     reasoning  regulation persist_   network/   metrics/
    physics       dream/     emotion/   watchdog   sqlite/               14 route
                  semantic/  perception backup    persist_file          handlers
                  nlp/       agent/               backup
                  clustering functional_
                             emotion
```

**62 modules across 8 layers**, ~26K lines of Rust.

## Core Innovations

### 1. Energy Conservation as a First Principle

The entire system operates on a finite energy budget. Every operation — materialization, flow, transfer — preserves total energy exactly. This isn't approximated: it's enforced at the type level with zero drift across millions of operations.

```
Total energy: 10,000,000.0
After 1M encode/decode cycles: 10,000,000.0 (drift: 0.0)
```

### 2. 7D Dark Universe with Physical Emergence

The dark universe has 7 dimensions: 3 physical (x, y, z) + 4 dark (E, S, T, μ). Nodes exist in full 7D space. When a node's physical energy ratio exceeds 0.5, it "manifests" into the observable BCC lattice — the physical world crystallizes from the dark universe.

### 3. Offset Encoding with Arbitrary Precision

Real numbers (including negatives, zero, and extremes like 1e-10) are encoded as energy offsets relative to a node's base energy. Decoding recovers the original values with tested precision to 1e-10 tolerance across 28 dimensions.

### 4. Tetrahedron as Minimal Memory Unit

Each memory atom occupies a tetrahedron of 4 nodes in 7D space. This gives every memory an intrinsic geometric structure — volume, orientation, and neighborhood relationships that enable spatial reasoning without external indexing.

### 5. Semantic Understanding Without External Models

A 5-layer semantic engine (S1–S5) provides text understanding using TetraMem's own spatial architecture:

| Layer | Name | Mechanism |
|-------|------|-----------|
| S1 | NLP | TF-IDF (128-dim), Chinese bigram, synonym buckets, contradiction detection |
| S2 | Embedding | 64-dim statistical + histogram + DFT frequency + NLP text features |
| S3 | Knowledge Graph | Typed relations (IsA, Causes, SimilarTo...) with automatic linking |
| S4 | Concept Abstraction | Dream-driven prototype extraction with incremental centroid updates |
| S5 | Semantic Query | Unified query language with filter composition and KNN search |

Text search uses TF-IDF embeddings projected into the same 64-dim space, enabling KNN + multi-hop Hebbian expansion for associative retrieval — no external ML model required.

### 6. Hebbian Learning & Pulse Propagation

Memories form connections through Hebbian reinforcement. Pulse propagation (reinforcing, exploratory, cascade) traverses the lattice, strengthening paths between related memories. Dream consolidation replays and prunes connections during idle cycles.

### 7. Multi-Hop Semantic Search

Text queries go through TF-IDF embedding → KNN search → Hebbian edge expansion (configurable 2–3 hops), discovering associations that direct similarity would miss.

### 8. 4-Layer Memory Clustering Engine

| Layer | Strategy | Purpose |
|-------|----------|---------|
| L1 | Semantic Placement | Position new memories near similar existing ones |
| L2 | Dark Gravity | Attract distant similar memories via dark dimension fields |
| L3 | Resonance Tunnel | Bridge disconnected components with tunnel edges |
| L4 | Topology Bridge | Connect isolated clusters via Betti number analysis |

### 9. Raft Consensus with Energy Quorum

Distributed clustering via Raft protocol, with a novel energy quorum: state changes require proof that energy conservation is maintained across nodes before execution.

## REST API (62 endpoints)

### Memory Operations
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/memory/encode` | Encode memory at 3D anchor |
| POST | `/api/memory/decode` | Decode memory from 3D anchor |
| GET | `/api/memory/list` | List all memories |
| POST | `/api/memory/annotate` | Add tags, category, description |
| POST | `/api/memory/trace` | Trace associations via Hebbian graph |
| GET | `/api/memory/timeline` | Memory timeline by date |
| POST | `/api/memory/remember` | AI agent: store natural language |
| POST | `/api/memory/recall` | AI agent: retrieve by context |
| POST | `/api/memory/associate` | AI agent: find associations |
| POST | `/api/memory/forget` | Remove memory, recover energy |

### Semantic Engine
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/semantic/status` | Engine status and statistics |
| POST | `/api/semantic/search` | Search by data vector (KNN) |
| POST | `/api/semantic/query` | Search by text (TF-IDF + KNN) |
| POST | `/api/semantic/relations` | Get knowledge graph relations |
| POST | `/api/semantic/index-all` | Re-index all memories |

### 7D Dark Dimensions
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/dark/query` | Query full 7D energy state |
| POST | `/api/dark/flow` | Transfer energy between physical/dark dims |
| POST | `/api/dark/transfer` | Transfer energy between 7D nodes |
| POST | `/api/dark/materialize` | Materialize node at 7D coordinates |
| POST | `/api/dark/dematerialize` | Dematerialize, recover energy |
| GET | `/api/dark/pressure` | Dimension pressure and entropy |

### Physics Engine
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/physics/status` | Current physics configuration |
| GET | `/api/physics/profile` | Metric tensor profile |
| POST | `/api/physics/distance` | Compute 7D distance |
| POST | `/api/physics/project` | Project coordinates |
| POST | `/api/physics/configure` | Update physics parameters |

### Cognitive Systems
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/pulse` | Fire pulse (reinforcing/exploratory/cascade) |
| POST | `/api/dream` | Run dream cycle |
| POST | `/api/dream/consolidate` | Dream consolidation |
| POST | `/api/context` | Context management for AI agents |

### Emotion System
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/emotion/status` | PAD emotion state |
| POST | `/api/emotion/pulse` | Emotion-modulated pulse |
| POST | `/api/emotion/dream` | Emotion-influenced dream |
| POST | `/api/emotion/crystallize` | Emotion-weighted crystallization |

### Agents
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/agent/observer` | Observer agent |
| GET | `/api/agent/emotion` | Emotion agent |
| POST | `/api/agent/crystal` | Crystal agent |

### Cluster & Consensus
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/cluster/status` | Cluster status |
| POST | `/api/cluster/init` | Initialize Raft cluster |
| POST | `/api/cluster/propose` | Propose command |
| POST | `/api/cluster/add-node` | Add node |
| POST | `/api/cluster/remove-node` | Remove node |

### Phase Transition & Quorum
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/phase/detect` | Detect phase transitions |
| POST | `/api/phase/consensus` | Phase consensus proposal |
| POST | `/api/phase/quorum/start` | Start energy quorum |
| POST | `/api/phase/quorum/confirm` | Confirm quorum |
| GET | `/api/phase/quorum/status` | Quorum status |
| POST | `/api/phase/quorum/execute` | Execute quorum decision |

### Scaling & Maintenance
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/scale` | Auto-scale universe |
| POST | `/api/scale/frontier/:n` | Frontier expansion |
| POST | `/api/regulate` | Regulation cycle |
| GET | `/api/constitution/status` | Constitutional rules |
| GET | `/api/watchdog/status` | Watchdog state |
| POST | `/api/watchdog/checkup` | Force health check |
| GET | `/api/clustering/status` | Clustering engine status |
| POST | `/api/clustering/maintenance` | Run clustering maintenance |
| GET | `/api/perception/status` | Perception budget |
| POST | `/api/perception/replenish` | Replenish perception budget |

### Infrastructure
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| POST | `/login` | JWT authentication |
| GET | `/api/stats` | Universe statistics |
| GET | `/api/metrics` | Prometheus metrics |
| GET | `/api/openapi.json` | OpenAPI 3.0 spec |
| POST | `/api/backup/create` | Create backup |
| GET | `/api/backup/list` | List backups |
| GET | `/api/events/status` | Event bus status |

## Quick Start

### Build & Test

```bash
cargo build --release

cargo test                          # 343 unit tests
cargo test --test api_integration   # 15 HTTP integration tests
cargo test --test e2e_api           # 41 E2E HTTP tests
```

### Start Server

```bash
cargo run --release serve                    # default: 127.0.0.1:3456
cargo run --release serve 0.0.0.0:8080       # custom address
cargo run --release -- --config config.toml serve
```

The server serves both the REST API (at `/api/*`) and the built-in web panel (SPA at `/`).

### Programmatic Usage

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

## Test Coverage

| Category | Count | Description |
|----------|-------|-------------|
| Unit tests | 343 | Per-module correctness across all 8 layers |
| API integration | 15 | Full endpoint coverage via axum test harness |
| E2E HTTP tests | 41 | Real HTTP server lifecycle tests |
| **Total** | **399** | |

### Stress Test Results

| Test | Operation | Result |
|------|-----------|--------|
| S1 | 10K nodes × 100K operations | 12ms, 0 violations |
| S2 | 10K memories flood | precision within 1e-10 |
| S3 | 1M decode operations | 189ms |
| S11 | 50K chaotic operations | 0 corruption |
| S12 | 20K longevity rounds | 0 degradation |

## Security

12 rounds of security audit completed:

- **Auth**: JWT (HS256, iss/aud/nbf/jti UUID v4, 24h expiry), Argon2id passwords, timing-safe dummy hash
- **RBAC**: 4-tier (public → raft → user → admin)
- **Rate Limiting**: SeqCst atomic, per-IP login (10/5min, 10K cap)
- **Headers**: CSP, X-Frame-Options DENY, X-Content-Type-Options, Referrer-Policy, Permissions-Policy
- **CORS**: Specific methods + headers, no `Any`
- **Constant-time**: `subtle` crate for Raft secret comparison
- **Integrity**: HMAC-SHA256 for Raft log, SHA-256 for backups
- **Persistence**: SQLite synchronous=FULL, atomic writes, file permissions 0600
- **Concurrency**: Fine-grained RwLock per resource, O(1) HashMap index
- **Docker**: Non-root, no-new-privileges, cap_drop ALL, read-only fs
- **CI**: SHA-pinned Actions, `--locked` on all cargo commands

## MCP Tool Integration

23 MCP-compatible tools for AI agent integration, including:

- Memory CRUD: `encode`, `decode`, `annotate`, `forget`
- Search: `semantic_search`, `text_query`, `trace`, `recall`
- Cognitive: `pulse`, `dream`, `reason`, `emotion`, `scale`
- System: `watchdog`, `health`, `stats`

Plus 8 built-in skills and pipeline chain execution.

## Frontend Panel

Built-in React web panel with 15 pages for real-time monitoring and control:

- Universe dashboard (energy, nodes, conservation)
- Memory browser with timeline
- Semantic search interface
- Hebbian graph visualization
- Clustering status
- Emotion state monitor
- Phase transition detector
- Cluster management
- And more...

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| [axum](https://crates.io/crates/axum) | 0.7 | HTTP framework |
| [tokio](https://crates.io/crates/tokio) | 1 | Async runtime |
| [serde](https://crates.io/crates/serde) | 1 | Serialization |
| [rusqlite](https://crates.io/crates/rusqlite) | 0.31 | SQLite (bundled) |
| [openraft](https://crates.io/crates/openraft) | 0.10 | Raft consensus |
| [tower-http](https://crates.io/crates/tower-http) | 0.6 | Middleware stack |
| [jsonwebtoken](https://crates.io/crates/jsonwebtoken) | 9 | JWT auth |
| [subtle](https://crates.io/crates/subtle) | 2 | Constant-time ops |
| [prometheus](https://crates.io/crates/prometheus) | 0.13 | Metrics export |

## License

[GNU Affero General Public License v3.0 or later](LICENSE)

## Author

**sunormesky-max (Liu Qihang)** — sunormesky@gmail.com
