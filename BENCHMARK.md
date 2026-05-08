# TetraMem-XL v12.0 — Honest Benchmark Results

> **This is NOT a marketing document.** All numbers are from synthetic data on a single machine. No cherry-picking. No vs-v8 comparisons. No hype.

## Test Environment

- **OS**: Windows (release build, `codegen-units=1`, LTO=thin)
- **Data**: Synthetic random vectors (14-dim) + clustered data
- **Embedding**: 64-dim (statistical + histogram + frequency + meta features)
- **KNN**: Brute-force O(n) scan, cosine similarity

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

**Honest assessment**: Perfect recall is expected — brute-force KNN always finds exact nearest neighbors. This is the baseline, not an achievement. A real ANN index (HNSW) would be faster but might have <1.0 recall.

**Scaling**: O(n) — at 5K memories, 1ms/query. At 100K, expect ~20ms/query. At 1M, ~200ms/query. This is where HNSW/IVF would dominate.

## 2. Multi-Hop Associative Recall (Hebbian Graph Walk)

| Config            | Memories | Edges | KNN results | Hebbian extra | hop-1 | hop-2 | hop-3 |
|-------------------|----------|-------|-------------|---------------|-------|-------|-------|
| 10 clusters × 50  | 500      | 1494  | 10.0        | 15.8          | 105   | 208   | 316   |
| 20 clusters × 50  | 1000     | 2994  | 10.0        | 16.2          | 105   | 216   | 325   |
| 50 clusters × 20  | 1000     | 2994  | 10.0        | 15.7          | 99    | 208   | 314   |

**Honest assessment**: This is TetraMem's unique capability — Hebbian graph walk discovers ~16 additional relevant memories per query that pure KNN misses. Multi-hop traversal reaches 3x more nodes than single-hop.

**Caveat**: Hebbian edges here are synthetic (sequential neighbors), not learned from real usage patterns. In production with learned Hebbian weights, this number would vary.

## 3. Throughput (Encode + Embed + Index)

| N    | encoded | time   | rate     |
|------|---------|--------|----------|
| 100  | 100     | 0.7ms  | 136K/s   |
| 500  | 500     | 3.4ms  | 149K/s   |
| 1000 | 1000    | 6.8ms  | 146K/s   |
| 5000 | 5000    | 35.1ms | 142K/s   |

**Honest assessment**: ~140K operations/sec. Reasonable for encode+embed+index pipeline, but this doesn't include Hebbian edge creation or clustering. Real production throughput would be lower.

## 4. Energy Conservation

| N    | drift   | ok  |
|------|---------|-----|
| 100  | 0.00e0  | YES |
| 1000 | 0.00e0  | YES |
| 5000 | 0.00e0  | YES |

**Honest assessment**: Exact conservation holds on these synthetic workloads. This verifies the accounting implementation, not a thermodynamic law. Under serialization + distributed consensus, float round-trips would introduce non-zero drift.

## What This Benchmark Does NOT Show

1. **No external comparison** — We didn't compare against Qdrant, HNSWlib, USearch, or any real vector DB.
2. **No real-world data** — Synthetic random/clustered data only, not NLP embeddings, images, or real documents.
3. **No scaling test beyond 5K** — Real systems handle millions. TetraMem's brute-force KNN would degrade to seconds/query at that scale.
4. **No learned Hebbian weights** — Graph edges are synthetic sequential neighbors, not learned from actual usage.
5. **No concurrent load test** — Single-threaded sequential operations only.

## Known Weaknesses (Honest)

- **KNN is O(n) brute-force** — No ANN index structure. At 100K+ memories, this is a bottleneck.
- **Embedding quality unproven** — 64-dim statistical embedding hasn't been validated against standard NLP benchmarks (GLUE, BEIR, etc.).
- **No formal verification of energy conservation** — Tested via assertions, not verified with Prusti/Creusot.
- **Memory overhead** — Each MemoryAtom stores 4 Coord7D vertices (4×7×4 = 112 bytes) + data + metadata, significantly more than a plain vector ID.

## How to Run

```bash
cargo test --test honest_benchmark --release
```

## Next Steps for Real Benchmarks

To make this meaningful, we need:

1. Add `hnswlib` or `instant-distance` as dev-dependency for ANN baseline
2. Use standard datasets (SIFT1M, GloVe-1M, or random 128-dim vectors at 1M scale)
3. Measure recall@10, QPS, P99 latency, RSS memory
4. Run 24-hour sustained write+query stress test
5. Compare TetraMem's multi-hop recall against GraphRAG implementations
