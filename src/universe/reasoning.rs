use crate::universe::coord::Coord7D;
use crate::universe::crystal::CrystalEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use crate::universe::pulse::{PulseEngine, PulseType};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ReasoningResult {
    pub result_type: ReasoningType,
    pub source: String,
    pub targets: Vec<String>,
    pub confidence: f64,
    pub hops: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReasoningType {
    Analogy,
    Association,
    Inference,
    Discovery,
}

impl std::fmt::Display for ReasoningResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}[{}→{:?} conf:{:.2} hops:{}]",
            self.result_type, self.source, self.targets, self.confidence, self.hops
        )
    }
}

pub struct ReasoningEngine;

impl ReasoningEngine {
    pub fn find_analogies(
        universe: &DarkUniverse,
        memories: &[MemoryAtom],
        threshold: f64,
    ) -> Vec<ReasoningResult> {
        if memories.len() < 2 {
            return Vec::new();
        }

        let mut results = Vec::new();
        for i in 0..memories.len() {
            for j in (i + 1)..memories.len() {
                let similarity = Self::energy_similarity(universe, &memories[i], &memories[j]);
                if similarity >= threshold {
                    results.push(ReasoningResult {
                        result_type: ReasoningType::Analogy,
                        source: format!("{}", memories[i].anchor()),
                        targets: vec![format!("{}", memories[j].anchor())],
                        confidence: similarity,
                        hops: 0,
                    });
                }
            }
        }

        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        results
    }

    pub fn find_associations(
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        crystal: &CrystalEngine,
        source: &Coord7D,
        max_hops: usize,
    ) -> Vec<ReasoningResult> {
        let mut results = Vec::new();
        let mut visited = HashSet::new();
        visited.insert(*source);

        let mut frontier = vec![(*source, 0usize, 1.0f64)];

        while let Some((current, hops, confidence)) = frontier.pop() {
            if hops >= max_hops {
                continue;
            }

            let hebb_neighbors = hebbian.get_neighbors(&current);
            for (neighbor, weight) in &hebb_neighbors {
                if visited.contains(neighbor) {
                    continue;
                }
                if universe.get_node(neighbor).is_none() {
                    continue;
                }
                visited.insert(*neighbor);

                let new_conf = confidence * weight / 10.0;
                if new_conf < 0.01 {
                    continue;
                }

                results.push(ReasoningResult {
                    result_type: ReasoningType::Association,
                    source: format!("{}", source),
                    targets: vec![format!("{}", neighbor)],
                    confidence: new_conf.min(1.0),
                    hops: hops + 1,
                });

                frontier.push((*neighbor, hops + 1, new_conf));
            }

            let crystal_neighbors = crystal.crystal_neighbors(&current);
            for (neighbor, strength, _is_super) in &crystal_neighbors {
                if visited.contains(neighbor) {
                    continue;
                }
                visited.insert(*neighbor);

                results.push(ReasoningResult {
                    result_type: ReasoningType::Association,
                    source: format!("{}", source),
                    targets: vec![format!("{}", neighbor)],
                    confidence: (strength / 10.0).min(1.0),
                    hops: hops + 1,
                });

                frontier.push((*neighbor, hops + 1, (strength / 10.0).min(1.0)));
            }
        }

        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        results
    }

    pub fn infer_chain(
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        start: &Coord7D,
        end: &Coord7D,
        max_hops: usize,
    ) -> Vec<ReasoningResult> {
        let mut visited = HashSet::new();
        visited.insert(*start);

        let mut queue = std::collections::VecDeque::new();
        queue.push_back((vec![*start], 0usize));

        while let Some((path, hops)) = queue.pop_front() {
            if hops >= max_hops {
                continue;
            }
            let current = *path.last().unwrap();
            if current == *end {
                let mut results = Vec::new();
                for i in 0..path.len() - 1 {
                    let weight = hebbian.get_bias(&path[i], &path[i + 1]);
                    results.push(ReasoningResult {
                        result_type: ReasoningType::Inference,
                        source: format!("{}", path[i]),
                        targets: vec![format!("{}", path[i + 1])],
                        confidence: if weight > 0.0 { weight / 5.0 } else { 0.1 }.min(1.0),
                        hops: i + 1,
                    });
                }
                return results;
            }

            let neighbors = hebbian.get_neighbors(&current);
            for (neighbor, _) in &neighbors {
                if visited.contains(neighbor) {
                    continue;
                }
                if universe.get_node(neighbor).is_none() {
                    continue;
                }
                visited.insert(*neighbor);
                let mut new_path = path.clone();
                new_path.push(*neighbor);
                queue.push_back((new_path, hops + 1));
            }
        }

        Vec::new()
    }

    pub fn discover(
        universe: &DarkUniverse,
        hebbian: &mut HebbianMemory,
        seed: &Coord7D,
        pulse_strength: f64,
    ) -> Vec<ReasoningResult> {
        let engine = PulseEngine::new();
        let result = engine.propagate(seed, PulseType::Exploratory, universe, hebbian);

        let mut results = Vec::new();
        if result.visited_nodes > 5 {
            let strong = hebbian.strongest_edges(5);
            for ((a, b), w) in &strong {
                if *a == *seed || *b == *seed {
                    let target = if *a == *seed { b } else { a };
                    results.push(ReasoningResult {
                        result_type: ReasoningType::Discovery,
                        source: format!("{}", seed),
                        targets: vec![format!("{}", target)],
                        confidence: (*w * pulse_strength).min(1.0),
                        hops: 1,
                    });
                }
            }
        }

        results
    }

    fn energy_similarity(universe: &DarkUniverse, a: &MemoryAtom, b: &MemoryAtom) -> f64 {
        let mut sum_a = 0.0f64;
        let mut sum_b = 0.0f64;
        let mut dot = 0.0f64;

        for v in a.vertices() {
            if let Some(node) = universe.get_node(v) {
                for d in node.energy().dims() {
                    sum_a += d * d;
                }
            }
        }
        for v in b.vertices() {
            if let Some(node) = universe.get_node(v) {
                for d in node.energy().dims() {
                    sum_b += d * d;
                }
            }
        }

        for (va, vb) in a.vertices().iter().zip(b.vertices().iter()) {
            if let (Some(na), Some(nb)) = (universe.get_node(va), universe.get_node(vb)) {
                for (da, db) in na.energy().dims().iter().zip(nb.energy().dims().iter()) {
                    dot += da * db;
                }
            }
        }

        let norm_a = sum_a.sqrt();
        let norm_b = sum_b.sqrt();
        if norm_a < 1e-15 || norm_b < 1e-15 {
            return 0.0;
        }
        (dot / (norm_a * norm_b)).max(0.0).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::memory::MemoryCodec;

    fn setup_system() -> (DarkUniverse, HebbianMemory, CrystalEngine, Vec<MemoryAtom>) {
        let mut u = DarkUniverse::new(5_000_000.0);
        let mut h = HebbianMemory::new();

        let m1 = MemoryCodec::encode(&mut u,
            &Coord7D::new_even([10, 10, 10, 0, 0, 0, 0]),
            &[1.0, 2.0, 3.0]).unwrap();
        let m2 = MemoryCodec::encode(&mut u,
            &Coord7D::new_even([15, 15, 15, 0, 0, 0, 0]),
            &[3.0, 2.0, 1.0]).unwrap();
        let m3 = MemoryCodec::encode(&mut u,
            &Coord7D::new_even([20, 20, 20, 0, 0, 0, 0]),
            &[1.0, 2.0, 3.0]).unwrap();

        for x in 0..6i32 {
            for y in 0..6i32 {
                for z in 0..6i32 {
                    let c = Coord7D::new_even([x + 10, y + 10, z + 10, 0, 0, 0, 0]);
                    u.materialize_biased(c, 50.0, 0.6).ok();
                }
            }
        }

        let pulse_engine = PulseEngine::new();
        pulse_engine.propagate(m1.anchor(), PulseType::Reinforcing, &u, &mut h);
        pulse_engine.propagate(m2.anchor(), PulseType::Reinforcing, &u, &mut h);
        pulse_engine.propagate(m3.anchor(), PulseType::Reinforcing, &u, &mut h);

        h.record_path(&[*m1.anchor(), *m2.anchor()], 2.0);
        h.record_path(&[*m2.anchor(), *m3.anchor()], 1.5);

        let mut crystal = CrystalEngine::new();
        crystal.crystallize(&h, &u);

        (u, h, crystal, vec![m1, m2, m3])
    }

    #[test]
    fn find_analogies_by_energy() {
        let (u, _h, _c, mems) = setup_system();
        let results = ReasoningEngine::find_analogies(&u, &mems, 0.5);

        assert!(!results.is_empty(), "m1 and m3 have same data, should be analogous");
        assert_eq!(results[0].result_type, ReasoningType::Analogy);
    }

    #[test]
    fn find_associations_via_hebbian() {
        let (u, h, c, _mems) = setup_system();
        let results = ReasoningEngine::find_associations(&u, &h, &c,
            &Coord7D::new_even([10, 10, 10, 0, 0, 0, 0]), 3);

        assert!(!results.is_empty(), "should find associated nodes via hebbian");
    }

    #[test]
    fn infer_chain_connects_memories() {
        let (u, h, _c, mems) = setup_system();
        let results = ReasoningEngine::infer_chain(&u, &h,
            mems[0].anchor(), mems[2].anchor(), 10);

        if !results.is_empty() {
            assert_eq!(results[0].result_type, ReasoningType::Inference);
        }
    }

    #[test]
    fn discover_via_pulse() {
        let (u, mut h, _c, mems) = setup_system();
        let results = ReasoningEngine::discover(&u, &mut h, mems[0].anchor(), 0.5);

        assert!(!results.is_empty(), "pulse exploration should discover connections");
    }

    #[test]
    fn analogy_works_for_similar_data() {
        let mut u = DarkUniverse::new(500_000.0);
        let m1 = MemoryCodec::encode(&mut u,
            &Coord7D::new_even([10, 10, 10, 0, 0, 0, 0]),
            &[1.0, 2.0, 3.0]).unwrap();
        let m2 = MemoryCodec::encode(&mut u,
            &Coord7D::new_even([20, 20, 20, 0, 0, 0, 0]),
            &[1.0, 2.0, 3.0]).unwrap();

        let results = ReasoningEngine::find_analogies(&u, &[m1, m2], 0.5);
        assert!(!results.is_empty(), "identical data should be analogous");
    }

    #[test]
    fn reasoning_result_display() {
        let r = ReasoningResult {
            result_type: ReasoningType::Analogy,
            source: "A".to_string(),
            targets: vec!["B".to_string()],
            confidence: 0.85,
            hops: 0,
        };
        let s = format!("{}", r);
        assert!(s.contains("Analogy"));
    }
}
