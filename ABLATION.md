# TetraMem-XL v12.0 — Ablation Study Results

> Does the combination of Hebbian graph + BCC topology + energy eviction provide measurable advantages over any subset?

## Experiment Design

**Dataset**: 20 clusters × 50 points = 1000 memories (14-dim synthetic data)
**Eviction pressure**: Keep 400 of 1000 (evict 600, 60% removal)
**Important memories**: Clusters 0-4 marked importance=0.8, rest 0.3-0.8

**5 Variants**:

| Variant | Hebbian | Topology | Energy Eviction | Description |
|---------|---------|----------|-----------------|-------------|
| Full | ✓ | ✓ | ✓ | Complete TetraMem |
| NoHebbian | ✗ | ✓ | ✓ | No graph edges, KNN only |
| NoTopology | ✓ | ✗ | ✓ | Random placement |
| NoEnergy | ✓ | ✓ | ✗ | LRU eviction |
| BareMinimum | ✗ | ✗ | ✗ | Pure KNN + LRU |

**3 Metrics**:
1. **Multi-hop recall**: % of same-cluster memories found within 3-hop graph walk
2. **Forgetting rate**: % of important memories (importance≥0.7) lost after eviction
3. **Avg path length**: hops from query to nearest same-cluster target

## Results

| Variant | HopRecall | ForgetRate | AvgPathLen | Time |
|---------|-----------|------------|------------|------|
| **Full** | **100.00%** | **0.00%** | **2.8** | 44.5ms |
| NoHebbian | 0.00% | 0.00% | ∞ | 9.4ms |
| NoTopology | 100.00% | 0.00% | 2.8 | 44.9ms |
| NoEnergy | 99.66% | 100.00% | 3.0 | 41.0ms |
| BareMinimum | 0.00% | 100.00% | ∞ | 9.1ms |

## Delta vs Full

| Variant | HopRecall Δ | ForgetRate Δ | PathLength Δ |
|---------|-------------|--------------|--------------|
| NoHebbian | **−100%** | +0% | N/A (no path) |
| NoTopology | 0% | +0% | +0.0 hops |
| NoEnergy | −0.34% | **+100%** | +0.2 hops |
| BareMinimum | **−100%** | **+100%** | N/A (no path) |

## Interpretation

### Hebbian edges are essential (critical finding)

Removing Hebbian graph drops multi-hop recall from 100% to 0%. Without graph edges, there are no traversal paths — only flat KNN retrieval. **This is the single most important component.**

### Energy eviction protects important memories (critical finding)

LRU eviction (NoEnergy) loses **100% of important memories** because it evicts the oldest entries regardless of importance. Energy-aware eviction preserves all of them. Under 60% eviction pressure, this is the difference between keeping and destroying the memory system's most valuable content.

### Topology placement shows no measurable effect (honest negative)

NoTopology (random placement) performs identically to Full on all metrics. At this scale (1000 memories, 20 clusters), semantic placement of dark dimensions provides no measurable advantage over random coordinates. This may change at larger scale or with different cluster structures — but at this scale, **topology is not contributing**.

### BareMinimum vs Full: dramatic gap

Without Hebbian or energy eviction, the system cannot do multi-hop retrieval and loses all important memories under pressure. The 2x speed advantage (9ms vs 44ms) comes from doing dramatically less work — and getting dramatically worse results.

## Caveats

- **Scale**: 1000 memories, 60% eviction. Real systems face millions. Results may differ.
- **Hebbian edges are synthetic**: Built from sequential neighbors + similarity threshold, not learned from real usage.
- **Topology null result may be scale-dependent**: At 100K+ memories, semantic placement may reduce collision or improve locality.
- **Eviction is contrived**: Clean separation of important (0.8) vs unimportant (0.3) makes energy eviction look perfect. Real importance distributions are messier.
- **Single dataset**: Only tested on clustered synthetic data. Different data distributions may yield different ablation results.

## How to Run

```bash
cargo test --test ablation_study --release
```

## Next Steps

1. Test at larger scale (10K, 50K memories) to see if topology placement matters
2. Use real data (text embeddings, time series) instead of synthetic clusters
3. Add continuous learning scenario: stream new memories, measure recall of old clusters over time
4. Test with fuzzier importance distributions (uniform noise on importance scores)
5. Compare against a simple graph baseline (random graph with same edge count, instead of Hebbian)
