# TetraMem-XL v12.0 — Benchmark Results

> Internal performance measurements on synthetic data. No external baselines included (yet). See [What's Missing](#whats-missing) for next steps.

## Test Environment

- **Build**: release (`codegen-units=1`, LTO=thin)
- **Data**: Synthetic random vectors (14-dim) + clustered data
- **Embedding**: 64-dim (statistical + histogram + frequency + meta features)
- **KNN**: Brute-force O(n), cosine similarity
- **Scale**: up to 5,000 memories

## 1. KNN Recall@K

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

Recall = 1.0 because brute-force always finds exact nearest neighbors. This is expected — the interesting question is how this compares to ANN methods (HNSW, IVF) which trade recall for speed.

Scaling: O(n). At 5K memories: 1ms/query. Projected at 100K: ~20ms, at 1M: ~200ms. Adding an ANN index is the top priority for scaling.

## 2. Multi-Hop Associative Recall (Hebbian Graph Walk)

| Config            | Memories | Hebbian edges | KNN top-k | Hebbian extra | hop-1 | hop-2 | hop-3 |
|-------------------|----------|---------------|-----------|---------------|-------|-------|-------|
| 10 clusters × 50  | 500      | 1494          | 10        | 15.8          | 105   | 208   | 316   |
| 20 clusters × 50  | 1000     | 2994          | 10        | 16.2          | 105   | 216   | 325   |
| 50 clusters × 20  | 1000     | 2994          | 10        | 15.7          | 99    | 208   | 314   |

Hebbian graph walk discovers ~16 additional memories per query beyond what KNN returns. 3-hop traversal reaches ~316 nodes — 30x the KNN top-10.

This is TetraMem's core differentiator: **self-organizing associative retrieval without external graph DB or LLM**. The Hebbian edges grow organically from usage (pulse propagation, dream consolidation), not from manual schema or external models. Each hop follows directed, weighted edges that reflect actual co-activation history.

Current test uses synthetic sequential edges. With real learned Hebbian weights (from pulse + dream cycles), the discovery pattern would reflect genuine associative structure rather than insertion order.

## 3. Throughput (Encode + Embed + Index)

| N    | encoded | time   | rate     |
|------|---------|--------|----------|
| 100  | 100     | 0.7ms  | 136K/s   |
| 500  | 500     | 3.4ms  | 149K/s   |
| 1000 | 1000    | 6.8ms  | 146K/s   |
| 5000 | 5000    | 35.1ms | 142K/s   |

~140K encode+embed+index ops/sec. Includes: BCC lattice node creation, 14-dim data encoding into tetrahedral energy field, 64-dim statistical embedding computation, embedding index insertion.

Does not include: Hebbian edge boost, clustering cycle, topology analysis. Full pipeline throughput would be lower.

## 4. Energy Conservation

| N    | drift   | conserved |
|------|---------|-----------|
| 100  | 0.00e0  | YES |
| 1000 | 0.00e0  | YES |
| 5000 | 0.00e0  | YES |

Zero drift on all workloads. The energy accounting system (Kahan-compensated sum + exact reconciliation) maintains conservation across all encode/erase operations.

Under JSON serialization + Raft consensus round-trips, serialization precision may introduce sub-epsilon drift — this needs separate testing.

## Architecture Comparison

| Aspect | TetraMem | Mainstream approach | TetraMem's tradeoff |
|--------|----------|---------------------|---------------------|
| Embedding | 64-dim statistical | 384-1536 dim neural (MiniLM, etc.) | No external model, fully local, deterministic |
| KNN index | Brute-force O(n) | HNSW / IVF O(log n) | Exact recall, but needs ANN for scale |
| Associative retrieval | Self-organizing Hebbian graph | External graph DB + vector hybrid | Built-in, no infrastructure dependency |
| Memory eviction | Energy budget accounting | LRU / TTL | Physics-inspired, tunable per-dimension |
| Spatial structure | BCC tetrahedral lattice (7D) | Flat vector space | Intrinsic topology, neighborhood relations emerge from geometry |
| External dependencies | None (pure Rust) | Typically Python + GPU + external DB | Self-contained, embeddable, offline-capable |

TetraMem is designed for a different niche than general-purpose vector databases: **self-contained, offline-capable memory systems where associative structure emerges from usage rather than being externally engineered**.

## Known Gaps

- **KNN scaling**: O(n) without ANN index. Priority: add HNSW layer.
- **Embedding validation**: 64-dim statistical features haven't been benchmarked against learned embeddings on standard NLP tasks.
- **Hebbian discovery quality**: "Extra discoveries" from graph walk are topologically reachable — their semantic relevance needs user-level evaluation.
- **Serialization drift**: Conservation verified in-memory only; JSON/SQLite round-trips not tested.
- **No concurrent benchmark**: All tests are single-threaded sequential.

## How to Run

```bash
cargo test --test honest_benchmark --release
```

## What's Missing

This benchmark measures TetraMem against itself. To be meaningful, it needs:

1. **ANN baseline**: Add `instant-distance` or `hnswlib`, compare recall@10 + latency at 100K+ scale
2. **Real datasets**: SIFT1M, GloVe-1M, or domain-specific corpora
3. **GraphRAG comparison**: TetraMem's multi-hop vs LlamaIndex PropertyGraph / Neo4j hybrid on same queries
4. **Concurrency**: Multi-client sustained load test
5. **End-to-end quality**: Human or LLM-judged relevance of Hebbian discoveries vs KNN-only results
