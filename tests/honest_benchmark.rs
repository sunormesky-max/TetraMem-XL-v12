// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — Honest Benchmark
//
// This is NOT a marketing benchmark. It measures TetraMem's internal
// performance on synthetic data and reports results honestly.
//
// Tests:
//   1. KNN recall@k: TetraMem brute-force vs ground-truth nearest neighbors
//   2. Multi-hop associative recall: Hebbian graph walk vs pure KNN expansion
//   3. Throughput: Raw encode + index rate
//   4. Energy conservation under load

use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;

use tetramem_v12::universe::coord::Coord7D;
use tetramem_v12::universe::hebbian::HebbianMemory;
use tetramem_v12::universe::memory::MemoryCodec;
use tetramem_v12::universe::memory::{SemanticConfig, SemanticEmbedding, SemanticEngine};
use tetramem_v12::universe::node::DarkUniverse;

fn generate_random_data(n: usize, dim: usize, seed: u64) -> Vec<Vec<f64>> {
    let mut data = Vec::with_capacity(n);
    let mut s = seed;
    for _ in 0..n {
        let mut vec = Vec::with_capacity(dim);
        for _ in 0..dim {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let val = ((s >> 33) as f64) / (1u64 << 31) as f64 - 1.0;
            vec.push(val);
        }
        data.push(vec);
    }
    data
}

fn generate_clustered_data(
    n_clusters: usize,
    points_per_cluster: usize,
    dim: usize,
    spread: f64,
    seed: u64,
) -> Vec<Vec<f64>> {
    let mut data = Vec::with_capacity(n_clusters * points_per_cluster);
    let centroids = generate_random_data(n_clusters, dim, seed);
    let noise = generate_random_data(n_clusters * points_per_cluster, dim, seed + 1);
    for (ci, centroid) in centroids.iter().enumerate() {
        for pi in 0..points_per_cluster {
            let idx = ci * points_per_cluster + pi;
            let point: Vec<f64> = centroid
                .iter()
                .zip(noise[idx].iter())
                .map(|(c, n)| c + n * spread)
                .collect();
            data.push(point);
        }
    }
    data
}

fn brute_force_ground_truth(
    query_emb: &SemanticEmbedding,
    all_embeddings: &[SemanticEmbedding],
    k: usize,
) -> Vec<(usize, f64)> {
    let mut scored: Vec<(usize, f64)> = all_embeddings
        .iter()
        .enumerate()
        .map(|(i, e)| (i, query_emb.cosine_similarity(e)))
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);
    scored
}

fn recall_at_k(result_indices: &[usize], ground_truth: &[(usize, f64)]) -> f64 {
    let gt_set: HashSet<usize> = ground_truth.iter().map(|(idx, _)| *idx).collect();
    if gt_set.is_empty() {
        return 1.0;
    }
    let hits = result_indices
        .iter()
        .filter(|idx| gt_set.contains(idx))
        .count();
    hits as f64 / gt_set.len() as f64
}

fn make_anchor(i: usize) -> Coord7D {
    Coord7D::new_even([
        (i as i32) % 1000,
        ((i as i32) / 1000) % 1000,
        ((i as i32) / 1_000_000) % 100,
        (i as i32) % 50,
        0,
        0,
        0,
    ])
}

struct IndexedMemory {
    #[allow(dead_code)]
    data_idx: usize,
    #[allow(dead_code)]
    anchor: Coord7D,
}

fn main() {
    println!("TetraMem-XL v12.0 — HONEST BENCHMARK");
    println!("══════════════════════════════════════════════════════════");
    println!("This measures internal performance on synthetic data.");
    println!("No external baselines. No vs-v8 comparisons. No hype.");
    println!();

    let data_dim = 14usize;
    let k_values = [1, 5, 10, 20];

    println!("━━━ 1. KNN RECALL@K (TetraMem brute-force vs ground truth) ━━━");
    println!(
        "Data dim: {} | Embedding dim: 64 | Metric: cosine similarity",
        data_dim
    );
    println!();
    println!(
        "{:<8} {:<4} {:>10} {:>12} {:>10}",
        "N", "k", "recall@k", "avg_query", "total"
    );
    println!("{}", "-".repeat(50));

    for &n in &[100usize, 500, 1_000, 5_000] {
        let data = generate_random_data(n, data_dim, 42);
        let mut universe = DarkUniverse::new(10_000_000_000.0);
        let mut engine = SemanticEngine::new(SemanticConfig::default());

        let mut indexed_mems: Vec<IndexedMemory> = Vec::new();
        let mut anchor_to_idx: HashMap<Coord7D, usize> = HashMap::new();

        for (i, d) in data.iter().enumerate() {
            let anchor = make_anchor(i);
            if let Ok(atom) = MemoryCodec::encode(&mut universe, &anchor, d) {
                engine.index_memory_data_only(&atom, d);
                anchor_to_idx.insert(*atom.anchor(), indexed_mems.len());
                indexed_mems.push(IndexedMemory {
                    data_idx: i,
                    anchor: *atom.anchor(),
                });
            }
        }
        let indexed = indexed_mems.len();
        if indexed < 20 {
            println!("{:<8} — too few encoded", n);
            continue;
        }

        let embeddings: Vec<SemanticEmbedding> = data
            .iter()
            .map(|d| SemanticEmbedding::from_data(d))
            .collect();
        let n_queries = 50usize.min(indexed);
        let query_indices: Vec<usize> = (0..n_queries).map(|i| (i * indexed) / n_queries).collect();

        for &k in &k_values {
            if k > indexed {
                continue;
            }
            let mut total_recall = 0.0f64;
            let start = Instant::now();

            for &qi in &query_indices {
                let query_emb = &embeddings[indexed_mems[qi].data_idx];
                let results = engine.search_similar(&data[indexed_mems[qi].data_idx], k);
                let result_indices: Vec<usize> = results
                    .iter()
                    .filter_map(|r| {
                        let key = &r.atom_key;
                        let anchor_basis = key.vertices_basis[0];
                        let anchor_coord = Coord7D::new_even(anchor_basis);
                        anchor_to_idx.get(&anchor_coord).copied()
                    })
                    .collect();
                let ground_truth = brute_force_ground_truth(query_emb, &embeddings, k);
                total_recall += recall_at_k(&result_indices, &ground_truth);
            }

            let elapsed = start.elapsed();
            let avg_recall = total_recall / n_queries as f64;
            let avg_us = elapsed.as_micros() as f64 / n_queries as f64;

            println!(
                "{:<8} {:<4} {:>10.4} {:>10.0}µs {:>8.1}ms",
                indexed,
                k,
                avg_recall,
                avg_us,
                elapsed.as_secs_f64() * 1000.0,
            );
        }
        println!();
    }

    println!("━━━ 2. MULTI-HOP ASSOCIATIVE RECALL ━━━");
    println!("Measures: does Hebbian graph walk discover memories pure KNN misses?");
    println!("Data: clustered (intra-cluster similarity, inter-cluster gap)");
    println!();

    let cluster_configs: &[(usize, usize, f64)] = &[(10, 50, 0.3), (20, 50, 0.3), (50, 20, 0.3)];

    for &(n_clusters, per_cluster, spread) in cluster_configs {
        let data = generate_clustered_data(n_clusters, per_cluster, data_dim, spread, 99);

        let mut universe = DarkUniverse::new(10_000_000_000.0);
        let mut engine = SemanticEngine::new(SemanticConfig::default());
        let mut hebbian = HebbianMemory::new();

        let mut anchor_coords: Vec<Coord7D> = Vec::new();
        let mut stored_data: Vec<Vec<f64>> = Vec::new();

        for (i, d) in data.iter().enumerate() {
            let anchor = make_anchor(i);
            if let Ok(atom) = MemoryCodec::encode(&mut universe, &anchor, d) {
                engine.index_memory_data_only(&atom, d);

                for prev in anchor_coords.iter().rev().take(3) {
                    let w = 1.0 + (atom.data_dim() as f64) * 0.1;
                    hebbian.boost_edge(atom.anchor(), prev, w);
                }

                anchor_coords.push(*atom.anchor());
                stored_data.push(d.clone());
            }
        }

        let indexed = anchor_coords.len();
        if indexed < 20 {
            println!(
                "  {} clusters × {} — too few encoded",
                n_clusters, per_cluster
            );
            continue;
        }

        let k = 10usize.min(indexed / 2);
        let n_queries = 20usize.min(indexed);
        let mut knn_total = 0usize;
        let mut hebbian_extra = 0usize;
        let mut hop_counts = [0usize; 4];

        let start = Instant::now();

        for qi in (0..n_queries).map(|i| (i * indexed) / n_queries) {
            let knn_results = engine.search_similar(&stored_data[qi], k);

            let origin = anchor_coords[qi];
            let mut all_reachable = HashSet::new();
            all_reachable.insert(origin);

            for (n_hops, hop_count) in hop_counts.iter_mut().enumerate().skip(1).take(3) {
                let mut frontier: Vec<Coord7D> = vec![origin];
                let mut visited: HashSet<Coord7D> = HashSet::new();
                visited.insert(origin);

                for _ in 0..n_hops {
                    let mut next = Vec::new();
                    for node in &frontier {
                        for (nbr, _) in hebbian.get_neighbors(node) {
                            if visited.insert(nbr) {
                                next.push(nbr);
                            }
                        }
                    }
                    frontier = next;
                }

                for coord in &visited {
                    if !knn_results.iter().any(|r| {
                        let ab = r.atom_key.vertices_basis[0];
                        let ac = Coord7D::new_even(ab);
                        ac == *coord
                    }) {
                        all_reachable.insert(*coord);
                        *hop_count += 1;
                    }
                }
            }

            knn_total += knn_results.len();
            hebbian_extra += all_reachable.len() - 1;
        }

        let elapsed = start.elapsed();
        println!(
            "  {} clusters × {} (spread={}) — {} memories, {} Hebbian edges",
            n_clusters,
            per_cluster,
            spread,
            indexed,
            hebbian.edge_count(),
        );
        println!("    k={} queries={}", k, n_queries);
        println!(
            "    Pure KNN avg:           {:.1} results/query",
            knn_total as f64 / n_queries as f64
        );
        println!(
            "    Hebbian extra (missed by KNN): {:.1} results/query",
            hebbian_extra as f64 / n_queries as f64,
        );
        for (h, &count) in hop_counts.iter().enumerate().skip(1).take(3) {
            println!("      hop-{} discoveries: {}", h, count);
        }
        println!("    Time: {:.1}ms", elapsed.as_secs_f64() * 1000.0);
        println!();
    }

    println!("━━━ 3. THROUGHPUT — Encode + Embed + Index ━━━");
    println!();
    println!(
        "{:<8} {:>8} {:>10} {:>12}",
        "N", "encoded", "time_ms", "rate/s"
    );
    println!("{}", "-".repeat(42));

    for &n in &[100usize, 500, 1_000, 5_000] {
        let data = generate_random_data(n, data_dim, 77);
        let mut universe = DarkUniverse::new(100_000_000_000.0);
        let mut engine = SemanticEngine::new(SemanticConfig::default());

        let start = Instant::now();
        let mut encoded = 0usize;
        for (i, d) in data.iter().enumerate() {
            let anchor = make_anchor(i);
            if let Ok(atom) = MemoryCodec::encode(&mut universe, &anchor, d) {
                engine.index_memory_data_only(&atom, d);
                encoded += 1;
            }
        }
        let elapsed = start.elapsed();
        let rate = encoded as f64 / elapsed.as_secs_f64();

        println!(
            "{:<8} {:>8} {:>10.1} {:>12.0}",
            n,
            encoded,
            elapsed.as_secs_f64() * 1000.0,
            rate
        );
    }
    println!();

    println!("━━━ 4. ENERGY CONSERVATION UNDER LOAD ━━━");
    println!();
    println!(
        "{:<8} {:>12} {:>12} {:>12} {:>5}",
        "N", "initial", "after", "drift", "ok"
    );
    println!("{}", "-".repeat(52));

    for &n in &[100usize, 1_000, 5_000] {
        let data = generate_random_data(n, data_dim, 123);
        let mut universe = DarkUniverse::new(10_000_000_000.0);

        let initial = universe.total_energy();
        for (i, d) in data.iter().enumerate() {
            let anchor = make_anchor(i);
            let _ = MemoryCodec::encode(&mut universe, &anchor, d);
        }
        let after = universe.total_energy();
        let drift = (after - initial).abs();

        println!(
            "{:<8} {:>12.2} {:>12.2} {:>12.2e} {:>5}",
            n,
            initial,
            after,
            drift,
            if drift < 1e-6 { "YES" } else { "NO" },
        );
    }
    println!();

    println!("══════════════════════════════════════════════════════════");
    println!("END OF HONEST BENCHMARK");
    println!();
    println!("DISCLAIMERS:");
    println!("  - KNN is brute-force O(n). No HNSW/ANN index structure.");
    println!("  - No comparison against external systems (Qdrant, HNSWlib, etc).");
    println!("  - Synthetic random/clustered data only. Not real-world corpora.");
    println!("  - Hebbian edges are synthetic (sequential neighbors), not learned.");
    println!("  - To compare against real baselines: add hnswlib or qdrant-client");
    println!("    as dev-dependency and run side-by-side on same data.");
}
