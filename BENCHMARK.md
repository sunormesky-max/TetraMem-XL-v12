# TetraMem-XL v12.0 — Honest Benchmark Results

> **This is NOT a marketing document.** All numbers are from synthetic data on a single machine. No cherry-picking. No vs-v8 comparisons. No hype.

> **This benchmark is conducted on synthetic data with at most 5K memories. No public dataset (SIFT1M / GloVe / CORe50) was used. Results do not reflect production-scale performance.**

## Test Environment

- **OS**: Windows (release build, `codegen-units=1`, LTO=thin)
- **Data**: Synthetic random vectors (14-dim) + clustered data
- **Embedding**: 64-dim (statistical + histogram + frequency + meta features)
- **KNN**: Brute-force O(n) scan, cosine similarity
- **Max N**: 5,000 — well below any real-world memory system

## 1. KNN Recall@K (vs Ground Truth)

| N     | k  | recall@k | avg query | total  |
|-------|----|----------|-----------|--------|
| 100   | 1  | 1.0000   | 17µs      | 0.9ms  |
| 100   | 10 | 1.0000   | 17µs      | 0.9ms  |
| 500   | 1  | 1.0000   | 122µs     | 6.1ms  |
| 500   | 10 | 1.0000   | 116µs     | 5.8ms  |
| 1000  | 1  | 1.0000   | 182µs     | 9.1ms  |
| 1000  | 10 | 1.0000   | 167µs     | 8.3ms  |
| 5000  | 1  | 1.0000   | 1047µs    | 52.4ms |
| 5000  | 10 | 1.0000   | 1011µs    | 50.6ms |

**Assessment**: Perfect recall is expected — brute-force KNN always finds exact nearest neighbors. This is the baseline, not an achievement. A real ANN index (HNSW) would be faster but might have <1.0 recall.

**Scaling prediction**: O(n) — at 5K, 1ms/query. At 100K, ~20ms/query. At 1M, ~200ms/query. HNSW/IVF would be O(log n) at ~1-5ms regardless of scale. **TetraMem loses badly at scale.**

## 2. Multi-Hop Associative Recall (Hebbian Graph Walk)

| Config            | Memories | Edges | KNN results | Hebbian extra | hop-1 | hop-2 | hop-3 |
|-------------------|----------|-------|-------------|---------------|-------|-------|-------|
| 10 clusters × 50  | 500      | 1494  | 10.0        | 15.8          | 105   | 208   | 316   |
| 20 clusters × 50  | 1000     | 2994  | 10.0        | 16.2          | 105   | 216   | 325   |
| 50 clusters × 20  | 1000     | 2994  | 10.0        | 15.7          | 99    | 208   | 314   |

**Assessment**: Hebbian graph walk discovers ~16 additional relevant memories per query that pure KNN misses. Multi-hop traversal reaches 3x more nodes than single-hop.

**Why this may not matter**: Any GraphRAG / multi-hop retrieval system (Neo4j + vector hybrid, LlamaIndex PropertyGraph, even simple BM25 + expansion) can achieve similar or better results with far less implementation complexity. The Hebbian edges here are synthetic (sequential neighbors), not learned from real usage patterns. This is a *structural capability*, not a proven advantage.

## 3. Throughput (Encode + Embed + Index)

| N    | encoded | time   | rate     |
|------|---------|--------|----------|
| 100  | 100     | 0.7ms  | 136K/s   |
| 500  | 500     | 3.4ms  | 149K/s   |
| 1000 | 1000    | 6.8ms  | 146K/s   |
| 5000 | 5000    | 35.1ms | 142K/s   |

**Assessment**: ~140K operations/sec for the encode+embed+index pipeline. Does not include Hebbian edge creation, clustering, or topology computation. Real production throughput with all subsystems active would be significantly lower. For comparison, Qdrant handles 100K+ inserts/sec on real hardware with full indexing.

## 4. Energy Conservation

| N    | drift   | ok  |
|------|---------|-----|
| 100  | 0.00e0  | YES |
| 1000 | 0.00e0  | YES |
| 5000 | 0.00e0  | YES |

**Assessment**: Exact conservation on synthetic workloads. This verifies the f64 accounting implementation, not a thermodynamic law. Under serialization (JSON/SQLite) + distributed consensus (Raft round-trips), float serialization would introduce non-zero drift.

## What This Benchmark Does NOT Show

1. **No external comparison** — Not compared against Qdrant, HNSWlib, USearch, Neo4j, or any GraphRAG system.
2. **No real-world data** — Synthetic random/clustered data only. Not NLP embeddings, images, or real documents.
3. **No scaling test beyond 5K** — Real systems handle millions. TetraMem's brute-force KNN would degrade to seconds/query.
4. **No learned Hebbian weights** — Graph edges are synthetic sequential neighbors, not learned from actual usage.
5. **No concurrent load test** — Single-threaded sequential operations only.
6. **No recall quality evaluation** — "Extra discoveries" from Hebbian walk were not verified as actually relevant. They're topologically reachable, not semantically correct.

## Limitations & Why It May Not Matter

| TetraMem feature | Mainstream equivalent | TetraMem advantage | Honest verdict |
|---|---|---|---|
| 64-dim statistical embedding | OpenAI/MiniLM 384-1536 dim embeddings | No ML model needed | **Disadvantage**: statistical features << learned embeddings for semantic quality |
| Brute-force KNN | HNSW / IVF-PQ / ScaNN | Exact recall = 1.0 | **Disadvantage**: O(n) vs O(log n), loses at scale |
| Hebbian graph walk | GraphRAG / Neo4j + vector hybrid | Built-in, no external graph DB | **Marginal**: same capability, 10x more implementation complexity |
| Energy conservation accounting | LRU / TTL eviction in any cache | Physics-inspired budgeting | **Marginal**: different framing of resource constraints, no proven advantage |
| BCC tetrahedral lattice | Flat vector space | Intrinsic geometric structure | **Unproven**: no evidence geometric structure improves retrieval over flat vectors |

**Bottom line**: TetraMem's only structural differentiator (multi-hop Hebbian walk) is available in simpler, battle-tested systems. The 7D tetrahedral lattice adds complexity without proven retrieval benefit. This is an experimental toy, not a competitive memory system.

## Known Weaknesses

- **KNN is O(n) brute-force** — No ANN index. At 100K+ memories, unusable.
- **Embedding quality unproven** — 64-dim statistical features vs industry-standard 384+ dim learned embeddings.
- **No formal verification** — Energy conservation tested by assertions, not Prusti/Creusot.
- **Memory overhead** — Each MemoryAtom: 4×Coord7D (112 bytes) + data + metadata vs a plain vector ID (8 bytes).
- **BCC neighbor enumeration** — O(128) per node, fixed. Doesn't scale with topology.
- **No real user validation** — Zero production deployments, zero external users.

## How to Run

```bash
cargo test --test honest_benchmark --release
```

## Next Steps (Required Before Any Claims)

1. Add `instant-distance` or `hnswlib` as dev-dependency for ANN baseline comparison
2. Use standard datasets (SIFT1M, GloVe-1M) at 100K+ scale
3. Measure recall@10, QPS, P99 latency, RSS memory — publish raw JSON
4. Compare multi-hop recall against LlamaIndex PropertyGraph on same data
5. 24-hour sustained write+query stress test with concurrent clients
6. If results show TetraMem losing on all metrics (likely), document honestly and refocus project scope
