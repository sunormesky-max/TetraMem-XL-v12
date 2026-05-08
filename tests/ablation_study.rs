// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — Ablation Study
//
// Tests whether the combination of Hebbian + Topology + EnergyEviction
// provides measurable advantages over variants missing each component.
//
// 5 variants:
//   Full         — Hebbian + BCC topology + Energy eviction
//   NoHebbian    — KNN only, no graph edges
//   NoTopology   — Random placement instead of semantic clustering
//   NoEnergy     — LRU eviction instead of energy-aware eviction
//   BareMinimum  — Pure KNN + LRU, no graph, no topology
//
// 3 metrics:
//   1. Multi-hop recall@k — how many same-cluster memories found within 3 hops
//   2. Forgetting rate    — after eviction, % of important memories lost
//   3. Path length        — avg hops from query to target memory

use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;

use tetramem_v12::universe::coord::Coord7D;
use tetramem_v12::universe::hebbian::HebbianMemory;
use tetramem_v12::universe::memory::MemoryCodec;
use tetramem_v12::universe::memory::{SemanticConfig, SemanticEmbedding, SemanticEngine};
use tetramem_v12::universe::node::DarkUniverse;

const DATA_DIM: usize = 14;
const N_CLUSTERS: usize = 20;
const PER_CLUSTER: usize = 50;
const SPREAD: f64 = 0.3;
const TOTAL: usize = N_CLUSTERS * PER_CLUSTER;
const MAX_MEMORIES: usize = 400;
const EVICT_COUNT: usize = TOTAL - MAX_MEMORIES;

fn lcg(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
}

fn generate_clustered_data() -> (Vec<Vec<f64>>, Vec<usize>) {
    let mut data = Vec::with_capacity(TOTAL);
    let mut labels = Vec::with_capacity(TOTAL);
    let mut s = 42u64;
    let mut centroids = Vec::new();
    for _ in 0..N_CLUSTERS {
        let mut c = Vec::with_capacity(DATA_DIM);
        for _ in 0..DATA_DIM {
            s = lcg(s);
            c.push(((s >> 33) as f64) / (1u64 << 31) as f64 - 1.0);
        }
        centroids.push(c);
    }
    for ci in 0..N_CLUSTERS {
        for _ in 0..PER_CLUSTER {
            let mut point = Vec::with_capacity(DATA_DIM);
            for di in 0..DATA_DIM {
                s = lcg(s);
                let noise = ((s >> 33) as f64) / (1u64 << 31) as f64 - 1.0;
                point.push(centroids[ci][di] + noise * SPREAD);
            }
            data.push(point);
            labels.push(ci);
        }
    }
    (data, labels)
}

struct MemoryEntry {
    data_idx: usize,
    cluster: usize,
    anchor: Coord7D,
    importance: f64,
    data: Vec<f64>,
}

struct AblationResult {
    name: String,
    multi_hop_recall: f64,
    forgetting_rate: f64,
    avg_path_length: f64,
    total_time_ms: f64,
}

fn make_topology_anchor(i: usize, cluster: usize, data: &[f64]) -> Coord7D {
    let dark0 = (data[0] * 100.0) as i32;
    let dark1 = (data[1] * 100.0) as i32;
    let dark2 = (data[2] * 100.0) as i32;
    let dark3 = (data[3] * 100.0) as i32;
    let phys_x = (i as i32) % 50 + (cluster as i32) * 60;
    let phys_y = ((i as i32) / 50) % 50;
    let phys_z = 0;
    Coord7D::new_even([phys_x, phys_y, phys_z, dark0, dark1, dark2, dark3])
}

fn make_random_anchor(i: usize, seed: u64) -> Coord7D {
    let mut s = seed.wrapping_add(i as u64 * 997);
    s = lcg(s);
    let x = ((s >> 16) as i32) % 1000;
    s = lcg(s);
    let y = ((s >> 16) as i32) % 1000;
    s = lcg(s);
    let z = ((s >> 16) as i32) % 100;
    s = lcg(s);
    let d0 = ((s >> 16) as i32) % 200;
    s = lcg(s);
    let d1 = ((s >> 16) as i32) % 200;
    s = lcg(s);
    let d2 = ((s >> 16) as i32) % 200;
    s = lcg(s);
    let d3 = ((s >> 16) as i32) % 200;
    Coord7D::new_even([x, y, z, d0, d1, d2, d3])
}

fn build_hebbian_edges(entries: &[MemoryEntry], hebbian: &mut HebbianMemory) {
    for (i, entry) in entries.iter().enumerate() {
        for j in i.saturating_sub(3)..i {
            if j < entries.len() {
                let w = 1.0 + entry.data.len() as f64 * 0.05;
                hebbian.boost_edge(&entry.anchor, &entries[j].anchor, w);
            }
        }
        for entry2 in entries.iter().rev().take(5) {
            let sim = compute_data_similarity(&entry.data, &entry2.data);
            if sim > 0.7 {
                hebbian.boost_edge(&entry.anchor, &entry2.anchor, sim * 2.0);
            }
        }
    }
}

fn compute_data_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let nb: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if na < 1e-10 || nb < 1e-10 { 0.0 } else { (dot / (na * nb)).clamp(0.0, 1.0) }
}

fn energy_evict(entries: &mut Vec<MemoryEntry>, count: usize) -> usize {
    let mut scored: Vec<(usize, f64)> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| (i, e.importance))
        .collect();
    scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut to_remove: HashSet<usize> = HashSet::new();
    let mut important_lost = 0usize;
    for &(idx, imp) in scored.iter().take(count) {
        to_remove.insert(idx);
        if imp >= 0.7 {
            important_lost += 1;
        }
    }
    let mut new_entries = Vec::new();
    for (i, e) in entries.drain(..).enumerate() {
        if !to_remove.contains(&i) {
            new_entries.push(e);
        }
    }
    *entries = new_entries;
    important_lost
}

fn lru_evict(entries: &mut Vec<MemoryEntry>, count: usize) -> usize {
    let _total_important = entries.iter().filter(|e| e.importance >= 0.7).count();
    let mut to_remove: HashSet<usize> = HashSet::new();
    for i in 0..count.min(entries.len()) {
        to_remove.insert(i);
    }
    let important_lost = to_remove
        .iter()
        .filter(|&&idx| entries[idx].importance >= 0.7)
        .count();
    let mut new_entries = Vec::new();
    for (i, e) in entries.drain(..).enumerate() {
        if !to_remove.contains(&i) {
            new_entries.push(e);
        }
    }
    *entries = new_entries;
    important_lost
}

fn multi_hop_recall(
    entries: &[MemoryEntry],
    hebbian: &HebbianMemory,
    engine: &SemanticEngine,
    n_queries: usize,
) -> f64 {
    let cluster_map = build_cluster_map(entries);
    let k = 10usize.min(entries.len() / 2);
    let mut total_recall = 0.0f64;
    let mut queries_run = 0usize;

    for qi in (0..n_queries).map(|i| (i * entries.len()) / n_queries) {
        if qi >= entries.len() {
            break;
        }
        let query_cluster = entries[qi].cluster;
        let same_cluster_count = entries.iter().filter(|e| e.cluster == query_cluster).count();
        if same_cluster_count <= 1 {
            continue;
        }

        let knn_results = engine.search_similar(&entries[qi].data, k);
        let knn_set: HashSet<usize> = knn_results
            .iter()
            .filter_map(|r| {
                let ab = r.atom_key.vertices_basis[0];
                let ac = Coord7D::new_even(ab);
                cluster_map.get(&ac).copied()
            })
            .collect();

        let origin = entries[qi].anchor;
        let mut visited = HashSet::new();
        visited.insert(origin);
        let mut frontier = vec![origin];

        for _ in 0..3 {
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

        let mut found_in_cluster = 0usize;
        let total_in_cluster = entries
            .iter()
            .filter(|e| e.cluster == query_cluster)
            .count()
            .saturating_sub(1);

        for coord in &visited {
            if let Some(&idx) = cluster_map.get(coord) {
                if entries[idx].cluster == query_cluster && idx != qi {
                    found_in_cluster += 1;
                }
        }
        }
        for &idx in &knn_set {
            if idx < entries.len() && entries[idx].cluster == query_cluster && idx != qi {
                found_in_cluster += 1;
            }
        }

        if total_in_cluster > 0 {
            total_recall += (found_in_cluster as f64 / total_in_cluster as f64).min(1.0);
        }
        queries_run += 1;
    }

    if queries_run > 0 {
        total_recall / queries_run as f64
    } else {
        0.0
    }
}

fn avg_path_length(
    entries: &[MemoryEntry],
    hebbian: &HebbianMemory,
    n_queries: usize,
) -> f64 {
    let mut total_hops = 0usize;
    let mut found = 0usize;

    for qi in (0..n_queries).map(|i| (i * entries.len()) / n_queries) {
        if qi >= entries.len() {
            break;
        }
        let query_cluster = entries[qi].cluster;
        let target = entries.iter().enumerate().find(|(idx, e)| {
            *idx != qi && e.cluster == query_cluster
        });
        let (_, target_entry) = match target {
            Some(t) => t,
            None => continue,
        };
        let target_coord = target_entry.anchor;
        let origin = entries[qi].anchor;

        let mut visited = HashSet::new();
        visited.insert(origin);
        let mut frontier = vec![origin];

        for hop in 1..=6 {
            let mut next = Vec::new();
            for node in &frontier {
                for (nbr, _) in hebbian.get_neighbors(node) {
                    if !visited.contains(&nbr) {
                        if nbr == target_coord {
                            total_hops += hop;
                            found += 1;
                            break;
                        }
                        visited.insert(nbr);
                        next.push(nbr);
                    }
                }
                if found > 0 && total_hops > 0 {
                    let last_hop = total_hops;
                    if last_hop == hop + 1 {
                        break;
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            frontier = next;
        }
    }

    if found > 0 {
        total_hops as f64 / found as f64
    } else {
        f64::INFINITY
    }
}

fn build_cluster_map(entries: &[MemoryEntry]) -> HashMap<Coord7D, usize> {
    let mut map = HashMap::new();
    for (i, e) in entries.iter().enumerate() {
        map.insert(e.anchor, i);
    }
    map
}

fn run_variant(
    name: &str,
    use_hebbian: bool,
    use_topology: bool,
    use_energy_evict: bool,
    data: &[Vec<f64>],
    labels: &[usize],
) -> AblationResult {
    let start = Instant::now();

    let mut universe = DarkUniverse::new(10_000_000_000.0);
    let mut engine = SemanticEngine::new(SemanticConfig::default());
    let mut hebbian = HebbianMemory::new();
    let mut entries: Vec<MemoryEntry> = Vec::new();

    let mut rng_seed = 12345u64;

    for (i, d) in data.iter().enumerate() {
        let anchor = if use_topology {
            make_topology_anchor(i, labels[i], d)
        } else {
            let a = make_random_anchor(i, rng_seed);
            rng_seed = lcg(rng_seed);
            a
        };

        if let Ok(atom) = MemoryCodec::encode(&mut universe, &anchor, d) {
            engine.index_memory_data_only(&atom, d);

            let importance = if labels[i] < 5 { 0.8 } else { 0.3 + (i as f64 % 0.5) };

            entries.push(MemoryEntry {
                data_idx: i,
                cluster: labels[i],
                anchor: *atom.anchor(),
                importance,
                data: d.clone(),
            });
        }
    }

    if use_hebbian {
        build_hebbian_edges(&entries, &mut hebbian);
    }

    let important_before = entries.iter().filter(|e| e.importance >= 0.7).count();
    let _important_total = entries.iter().filter(|e| e.cluster < 5).count();

    if EVICT_COUNT > 0 && entries.len() > MAX_MEMORIES {
        let _lost = if use_energy_evict {
            energy_evict(&mut entries, EVICT_COUNT)
        } else {
            lru_evict(&mut entries, EVICT_COUNT)
        };

        engine = SemanticEngine::new(SemanticConfig::default());
        for e in &entries {
            let _emb = SemanticEmbedding::from_data(&e.data);
            let tmp_anchor = e.anchor;
            if let Ok(atom) = MemoryCodec::encode(&mut universe, &tmp_anchor, &e.data) {
                engine.index_memory_data_only(&atom, &e.data);
            }
        }

        if use_hebbian {
            hebbian = HebbianMemory::new();
            build_hebbian_edges(&entries, &mut hebbian);
        }
    }

    let important_after = entries.iter().filter(|e| e.importance >= 0.7).count();
    let forgetting = if important_before > 0 {
        1.0 - (important_after as f64 / important_before as f64)
    } else {
        0.0
    };

    let recall = multi_hop_recall(&entries, &hebbian, &engine, 30);
    let path_len = avg_path_length(&entries, &hebbian, 20);
    let elapsed = start.elapsed();

    AblationResult {
        name: name.to_string(),
        multi_hop_recall: recall,
        forgetting_rate: forgetting,
        avg_path_length: path_len,
        total_time_ms: elapsed.as_secs_f64() * 1000.0,
    }
}

fn main() {
    println!("TetraMem-XL v12.0 — ABLATION STUDY");
    println!("══════════════════════════════════════════════════════════");
    println!("Hypothesis: Hebbian + Topology + EnergyEviction > any subset");
    println!();
    println!("Dataset: {} clusters × {} points = {} total", N_CLUSTERS, PER_CLUSTER, TOTAL);
    println!("Eviction: keep {} of {} (evict {})", MAX_MEMORIES, TOTAL, EVICT_COUNT);
    println!("Clusters 0-4 marked important (importance=0.8)");
    println!();

    let (data, labels) = generate_clustered_data();

    let variants: Vec<(&str, bool, bool, bool)> = vec![
        ("Full",            true,  true,  true),
        ("NoHebbian",       false, true,  true),
        ("NoTopology",      true,  false, true),
        ("NoEnergy",        true,  true,  false),
        ("BareMinimum",     false, false, false),
    ];

    println!("━━━ Results ━━━");
    println!();
    println!("{:<15} {:>12} {:>12} {:>12} {:>10}",
        "Variant", "HopRecall", "ForgetRate", "AvgPathLen", "Time");
    println!("{}", "-".repeat(65));

    let mut results = Vec::new();
    for (name, h, t, e) in &variants {
        let r = run_variant(name, *h, *t, *e, &data, &labels);
        println!(
            "{:<15} {:>11.2}% {:>11.2}% {:>12.1} {:>8.1}ms",
            r.name,
            r.multi_hop_recall * 100.0,
            r.forgetting_rate * 100.0,
            if r.avg_path_length.is_infinite() { f64::INFINITY } else { r.avg_path_length },
            r.total_time_ms,
        );
        results.push(r);
    }

    println!();
    println!("━━━ Analysis ━━━");
    println!();

    let full = &results[0];
    for r in &results[1..] {
        let recall_delta = (r.multi_hop_recall - full.multi_hop_recall) * 100.0;
        let forget_delta = (r.forgetting_rate - full.forgetting_rate) * 100.0;
        let path_delta = if r.avg_path_length.is_infinite() || full.avg_path_length.is_infinite() {
            f64::NAN
        } else {
            r.avg_path_length - full.avg_path_length
        };

        println!("{} vs Full:", r.name);
        println!("  HopRecall:     {}{:.2}%", if recall_delta >= 0.0 { "+" } else { "" }, recall_delta);
        println!("  ForgetRate:    {}{:.2}%", if forget_delta >= 0.0 { "+" } else { "" }, forget_delta);
        if path_delta.is_nan() {
            println!("  PathLength:    N/A");
        } else {
            println!("  PathLength:    {}{:.1} hops", if path_delta >= 0.0 { "+" } else { "" }, path_delta);
        }
        println!();
    }

    println!("══════════════════════════════════════════════════════════");
    println!("END OF ABLATION STUDY");
    println!();
    println!("Interpretation guide:");
    println!("  - If Full > all variants: combination provides measurable advantage");
    println!("  - If NoHebbian ≈ Full: graph edges add no value in this scenario");
    println!("  - If NoTopology ≈ Full: semantic placement adds no value");
    println!("  - If NoEnergy ≈ Full: energy eviction ≈ LRU for this workload");
    println!("  - If BareMinimum ≈ Full: the whole architecture is unnecessary overhead");
    println!("  - Negative forget_delta = Full loses FEWER important memories");
}
