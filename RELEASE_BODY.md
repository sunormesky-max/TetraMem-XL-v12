<p align="center">
  <img src="https://img.shields.io/badge/tests-676-brightgreen?style=for-the-badge" />
  <img src="https://img.shields.io/badge/REST-62%20endpoints-blue?style=for-the-badge" />
  <img src="https://img.shields.io/badge/Rust-1.88+-orange?style=for-the-badge" />
  <img src="https://img.shields.io/badge/embedding-ONNX-9cf?style=for-the-badge" />
  <img src="https://img.shields.io/badge/license-AGPL--3.0-blue?style=for-the-badge" />
  <img src="https://img.shields.io/github/stars/sunormesky-max/TetraMem-XL-v12?style=social" />
</p>

# TetraMem-XL v12.0 — 7D Dark Universe Memory System

> Memory is not a database. Memory is physics.

TetraMem-XL stores memories as tetrahedra of energy in a 7-dimensional dark universe. The 3D physical world emerges as a cross-section when nodes reach manifestation threshold. Pure Rust, energy conservation enforced as mathematical invariant.

---

## What's New in v12.0.0

### Neural Embedding Engine (Optional)

Integrated ONNX Runtime neural embedding for true semantic understanding:

- **Model**: Granite Embedding Small (int8 quantized, ~50MB)
- **Runtime**: ONNX Runtime 1.24.2 via `ort` 2.0 (dynamic loading)
- **384-dim neural** fused with **64-dim hand-crafted** features (0.3 HC + 0.7 neural)
- **Pure Rust BPE tokenizer** — no external tokenizer dependencies
- **Graceful fallback** to hand-crafted only when model unavailable
- **Safety**: ONNX header validation, 512 token limit, NaN/finite guard

```toml
[neural_embed]
enabled = true
model_dir = "models/granite-embedding-small"
```

### Frontend Bug Fixes (8 items)

- Universe: dematerialize now calls API, view button fills coord input
- Dream: re-entry guard prevents double-click, action buttons wired to API
- Regulation: backup rotation button functional, unique IDs via Date.now()
- Pulse: unique IDs fix duplicate key warnings
- Memory: label fix (decode -> detail)
- Api: template hoisted outside callback, "load example" button wired
- Plugins: removed unused formatBytes function

### Other

- Clippy 0 warnings, `cargo fmt` clean
- `ort` 2.0.0-rc.12 (`load-dynamic`) + `ndarray` 0.16 added
- ONNX Runtime 1.24.2 DLL bundled in `lib/`
- Granite embedding int8 model in `models/`

---

## By The Numbers

| Metric | Value |
|---|---|
| Rust source | ~27,000 lines |
| Modules | 62 across 8 layers |
| Unit tests | 555 (all passing) |
| API integration | 15 (all passing) |
| E2E HTTP tests | 50 (all passing) |
| Full suite | 38 (all passing) |
| Stress tests | 12 (all passing) |
| Proptests | 6 (all passing) |
| **Total tests** | **676** |
| REST endpoints | 62 |
| MCP tools | 26 + 8 built-in skills |
| Frontend pages | 15 (React SPA) |
| Security audits | 12 rounds |
| MSRV | Rust 1.88 |
| License | AGPL-3.0-or-later |

---

## Architecture

```
                        TetraMem-XL v12.0
                              |
    +---------+----------+----------+----------+----------+----------+----------+----------+
    |  Core   |  Memory  | Cognitive| Adaptive | Storage  |Consensus |Interface |  Safety  |
    | (7D)    | (6 mod)  | (6 mod)  | (4 mod)  | (4 mod)  | (3 mod)  | (14 mod) | (3 mod)  |
    +----+----+----+-----+----+-----+----+-----+----+-----+----+-----+----+-----+----+-----+
         |         |          |          |          |          |          |          |
    coord/energy  memory/    crystal/   autoscale/ persist/   cluster/   api/       constitution/
    node/lattice  hebbian/   topology/  observer/  persist_   raft_node/ auth/      events/
    config/       pulse/     reasoning  regulation persist_   network/   metrics/
    physics       dream/     emotion/   watchdog   sqlite/               14 route
                  semantic/  perception backup     persist_file          handlers
                  nlp/       agent/                backup
                  clustering functional_
                             emotion
                  neural/
                  (ONNX)
```

---

## Core Features

### Energy Conservation as First Principle

Finite energy budget. Every operation preserves total energy exactly. Not approximated — enforced with zero drift.

```
Initial: 10,000,000.0
After 1M encode/decode: 10,000,000.0 (drift: 0.0)
```

### 7D Dark Universe

3 physical (x, y, z) + 4 dark (E, S, T, mu) dimensions. Nodes manifest into observable BCC lattice when physical energy ratio > 0.5.

### Semantic Engine (6 Layers)

| Layer | Mechanism |
|---|---|
| S1 — NLP | TF-IDF 128-dim, Chinese bigram, synonym buckets, contradiction detection |
| S2 — Embedding | 64-dim: statistical + histogram + DFT frequency + NLP text features |
| S3 — Neural | 384-dim ONNX embeddings (Granite, int8), fused 0.3 HC + 0.7 neural |
| S4 — Knowledge Graph | 8 relation types with automatic linking |
| S5 — Concepts | Dream-driven prototype extraction |
| S6 — Query | Composable filters + KNN + multi-hop Hebbian expansion |

### 4-Layer Clustering

| Layer | Strategy |
|---|---|
| L1 | Semantic placement near similar memories |
| L2 | Dark gravity via dark dimension fields |
| L3 | Resonance tunnels bridging disconnected components |
| L4 | Topology bridges via Betti number analysis |

### Cognitive Systems

- Hebbian learning with directed edges, STDP, temporal context
- Pulse propagation (reinforcing, exploratory, cascade)
- Dream consolidation (replay, weaken, consolidate)
- Emotion system (PAD model)
- Reasoning engine (analogy, association, inference)
- Novelty detection + Haar wavelet perception
- Active push (activation spreading + interest subscription + SSE)

---

## Honest Benchmarks

### KNN Recall

| N | k | Recall@k | Avg Query |
|---|---|----------|-----------|
| 100 | 10 | 1.000 | 192 us |
| 500 | 10 | 1.000 | 953 us |
| 1,000 | 10 | 1.000 | 1.93 ms |
| 5,000 | 10 | 1.000 | 10.9 ms |

> KNN is brute-force O(n). No ANN index (HNSW etc). This is the baseline — an ANN layer would improve query time at scale.

### Multi-Hop Associative Recall

| Config | KNN Results | Hebbian Extra | Time |
|---|---|---|---|
| 10 clusters x 50 (500 mem) | 10.0/query | +15.8/query | 15 ms |
| 20 clusters x 50 (1000 mem) | 10.0/query | +16.2/query | 27 ms |
| 50 clusters x 20 (1000 mem) | 10.0/query | +15.7/query | 27 ms |

> Hebbian multi-hop discovers memories KNN misses. Edge counts are synthetic (sequential neighbors), not learned from real usage.

### Throughput

| N | Encoded | Time | Rate |
|---|---------|------|------|
| 100 | 100 | 13 ms | 7,747/s |
| 500 | 500 | 66 ms | 7,625/s |
| 1,000 | 1,000 | 135 ms | 7,437/s |
| 5,000 | 5,000 | 653 ms | 7,659/s |

### Stress Tests

| Test | What | Result |
|---|---|---|
| S1 | 10K nodes x 100K ops | 100ms, 0 violations |
| S2 | 10K memories flood | 1.5s, precision 1.4e-14 |
| S3 | 1K mem x 1K decode = 1M ops | 2.1s, precision 1.4e-14 |
| S4 | 5K erase-rewrite cycles | 341ms, precision 1.4e-14 |
| S5 | 50K inter-node energy transfers | 60ms, 0 drift |
| S7 | 50-round full pipeline | 9.9s, precision 1.4e-14 |
| S8 | 100x persist-restore cycles | 1.1s, precision 2.1e-14 |
| S10 | 2K mixed-dim memories (1-28D) | 240ms, 0 corruption |
| S12 | 20K longevity encode+decode | 2.4s, 0 degradation |

### Energy Conservation

| N | Initial | After | Drift |
|---|---------|-------|-------|
| 100 | 10B | 10B | 0 |
| 1,000 | 10B | 10B | 0 |
| 5,000 | 10B | 10B | 0 |

### Ablation Study

| Variant | Hop Recall | Forget Rate |
|---|---|---|
| Full system | 100% | 0% |
| No Hebbian | 0% | 0% |
| No Topology | 100% | 0% |
| No Energy eviction | 99.66% | 100% |
| Bare minimum | 0% | 100% |

> Hebbian edges are essential for hop recall. Topology adds no value at 1K scale. Energy eviction prevents important memory loss.

---

## Minimum Hardware Requirements

| Component | Without Neural | With Neural |
|---|---|---|
| CPU | x86_64, 2 cores | x86_64, 4 cores recommended |
| RAM | 512 MB | 1 GB (model ~200 MB) |
| Disk | 50 MB | 150 MB (binary + model + ORT DLL) |
| OS | Win / Linux / macOS | Win / Linux / macOS |

Neural engine can be disabled (`neural_embed.enabled = false`) for zero overhead.

---

## Quick Start

```bash
cargo build --release

# All tests
cargo test                          # 555 unit tests
cargo test --test api_integration   # 15 API integration tests
cargo test --test e2e_api           # 50 E2E HTTP tests

# Run server
cargo run --release serve                    # http://127.0.0.1:3456
cargo run --release serve 0.0.0.0:8080       # custom address
```

### Code Demo

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

---

## Security

12 rounds of audit: JWT HS256 + Argon2id, 4-tier RBAC, rate limiting, CSP/CORS, constant-time secrets, HMAC-SHA256 Raft logs, SQLite synchronous=FULL, atomic writes, non-root Docker, SHA-pinned CI.

---

## Known Limitations

- **KNN is O(n) brute-force** — no ANN index (HNSW, etc). Query time scales linearly with memory count.
- **Raft transport is plain HTTP** — needs TLS for production cluster deployments.
- **openraft is alpha** (`0.10.0-alpha.18`) — pinned, migration required for stable release.
- **Neural embedding is English-only** (Granite model) — multilingual support would require a different model.
- **SemanticEngine is in-memory only** — rebuilt from stored memories on restart, not persisted to disk.
- **Single-node deployment tested** — multi-node Raft cluster has no production deployment data yet.

---

## Connect

- **Repository**: [github.com/sunormesky-max/TetraMem-XL-v12](https://github.com/sunormesky-max/TetraMem-XL-v12)
- **Issues**: [Report a bug](https://github.com/sunormesky-max/TetraMem-XL-v12/issues)
- **Author**: sunormesky-max (Liu Qihang) — sunormesky@gmail.com

---

<p align="center">
  If this project interests you, a star means a lot!
</p>
