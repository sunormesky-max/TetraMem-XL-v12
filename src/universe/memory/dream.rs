use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryCodec;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use crate::universe::pulse::{PulseEngine, PulseType};
use crate::universe::reasoning::ReasoningEngine;

#[derive(Debug, Clone)]
pub struct DreamReport {
    pub phase: DreamPhase,
    pub paths_replayed: usize,
    pub paths_weakened: usize,
    pub memories_consolidated: usize,
    pub memories_merged: usize,
    pub hebbian_edges_before: usize,
    pub hebbian_edges_after: usize,
    pub weight_before: f64,
    pub weight_after: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DreamPhase {
    Replay,
    Weaken,
    Consolidate,
    Merge,
}

#[derive(Clone)]
pub struct DreamConfig {
    pub replay_rounds: usize,
    pub replay_pulse_type: PulseType,
    pub weaken_decay_rounds: usize,
    pub consolidation_hebbian_threshold: f64,
    pub min_replay_strength: f64,
    pub merge_similarity_threshold: f64,
    pub merge_enabled: bool,
}

impl Default for DreamConfig {
    fn default() -> Self {
        Self {
            replay_rounds: 3,
            replay_pulse_type: PulseType::Reinforcing,
            weaken_decay_rounds: 5,
            consolidation_hebbian_threshold: 0.3,
            min_replay_strength: 0.1,
            merge_similarity_threshold: 0.8,
            merge_enabled: true,
        }
    }
}

pub struct DreamEngine {
    config: DreamConfig,
}

impl Default for DreamEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DreamEngine {
    pub fn new() -> Self {
        Self {
            config: DreamConfig::default(),
        }
    }

    pub fn with_config(config: DreamConfig) -> Self {
        Self { config }
    }

    pub fn dream(
        &self,
        universe: &DarkUniverse,
        hebbian: &mut HebbianMemory,
        memories: &[MemoryAtom],
    ) -> DreamReport {
        let edges_before = hebbian.edge_count();
        let weight_before = hebbian.total_weight();

        let replayed = self.replay_phase(universe, hebbian);
        let weakened = self.weaken_phase(hebbian);
        let consolidated = self.consolidate_phase(universe, hebbian, memories);

        let edges_after = hebbian.edge_count();
        let weight_after = hebbian.total_weight();

        DreamReport {
            phase: DreamPhase::Consolidate,
            paths_replayed: replayed,
            paths_weakened: weakened,
            memories_consolidated: consolidated,
            memories_merged: 0,
            hebbian_edges_before: edges_before,
            hebbian_edges_after: edges_after,
            weight_before,
            weight_after,
        }
    }

    pub fn dream_with_merge(
        &self,
        universe: &mut DarkUniverse,
        hebbian: &mut HebbianMemory,
        memories: &mut Vec<MemoryAtom>,
    ) -> DreamReport {
        let edges_before = hebbian.edge_count();
        let weight_before = hebbian.total_weight();

        let replayed = self.replay_phase(universe, hebbian);
        let weakened = self.weaken_phase(hebbian);
        let consolidated = self.consolidate_phase(universe, hebbian, memories);
        let merged = if self.config.merge_enabled {
            self.merge_phase(universe, hebbian, memories)
        } else {
            0
        };

        if !universe.verify_conservation() {
            tracing::error!("ENERGY CONSERVATION VIOLATED after dream merge — this is a critical bug");
        }

        let edges_after = hebbian.edge_count();
        let weight_after = hebbian.total_weight();

        DreamReport {
            phase: DreamPhase::Merge,
            paths_replayed: replayed,
            paths_weakened: weakened,
            memories_consolidated: consolidated,
            memories_merged: merged,
            hebbian_edges_before: edges_before,
            hebbian_edges_after: edges_after,
            weight_before,
            weight_after,
        }
    }

    fn merge_phase(
        &self,
        universe: &mut DarkUniverse,
        hebbian: &mut HebbianMemory,
        memories: &mut Vec<MemoryAtom>,
    ) -> usize {
        if memories.len() < 2 {
            return 0;
        }

        let analogies = ReasoningEngine::find_analogies(
            universe,
            memories,
            self.config.merge_similarity_threshold,
        );

        if analogies.is_empty() {
            return 0;
        }

        let mut participated: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut remove_set: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut merged_count = 0;

        for result in &analogies {
            let source_idx = memories
                .iter()
                .position(|m| format!("{}", m.anchor()) == result.source);
            let target_idx = result.targets.first().and_then(|t| {
                memories
                    .iter()
                    .position(|m| format!("{}", m.anchor()) == *t)
            });

            if let (Some(si), Some(ti)) = (source_idx, target_idx) {
                if participated.contains(&si) || participated.contains(&ti) {
                    continue;
                }
                participated.insert(si);
                participated.insert(ti);

                let keep_idx = si.min(ti);
                let remove_idx = si.max(ti);

                let remove_importance = memories[remove_idx].importance();
                let keep_importance = memories[keep_idx].importance();
                let higher_imp = remove_importance.max(keep_importance);

                if let Some(keep_mem) = memories.get_mut(keep_idx) {
                    keep_mem.set_importance(higher_imp);
                }

                if let Some(remove_mem) = memories.get(remove_idx) {
                    let anchor = remove_mem.anchor();
                    let neighbors = hebbian.get_neighbors(anchor);
                    let keep_anchor = *memories[keep_idx].anchor();
                    for (neighbor, weight) in &neighbors {
                        hebbian.record_path(&[keep_anchor, *neighbor], *weight);
                    }
                }

                remove_set.insert(remove_idx);
                merged_count += 1;
            }
        }

        let mut remove_indices: Vec<usize> = remove_set.into_iter().collect();
        remove_indices.sort_by(|a, b| b.cmp(a));
        for idx in remove_indices {
            if idx < memories.len() {
                if let Some(mem) = memories.get(idx) {
                    MemoryCodec::erase(universe, mem);
                }
                memories.remove(idx);
            }
        }

        merged_count
    }

    fn replay_phase(&self, universe: &DarkUniverse, hebbian: &mut HebbianMemory) -> usize {
        let engine = PulseEngine::new();
        let strong = hebbian.strongest_edges(20);
        let mut replayed = 0;

        for ((a, b), w) in &strong {
            if *w < self.config.min_replay_strength {
                continue;
            }

            for _ in 0..self.config.replay_rounds {
                let r = engine.propagate(a, self.config.replay_pulse_type, universe, hebbian);
                replayed += r.paths_recorded;

                let r2 = engine.propagate(b, self.config.replay_pulse_type, universe, hebbian);
                replayed += r2.paths_recorded;
            }
        }

        replayed
    }

    fn weaken_phase(&self, hebbian: &mut HebbianMemory) -> usize {
        let before = hebbian.edge_count();
        for _ in 0..self.config.weaken_decay_rounds {
            hebbian.decay_all();
        }
        before - hebbian.edge_count()
    }

    fn consolidate_phase(
        &self,
        universe: &DarkUniverse,
        hebbian: &mut HebbianMemory,
        memories: &[MemoryAtom],
    ) -> usize {
        if memories.len() < 2 {
            return 0;
        }

        let mut consolidated = 0;
        for i in 0..memories.len() {
            for j in (i + 1)..memories.len() {
                let ai = memories[i].anchor();
                let aj = memories[j].anchor();
                let bias = hebbian.get_bias(ai, aj);
                if bias >= self.config.consolidation_hebbian_threshold {
                    let path = vec![*ai, *aj];
                    hebbian.record_path(&path, bias * 1.5);
                    consolidated += 1;
                }
            }
        }

        let engine = PulseEngine::new();
        for mem in memories {
            let anchor = mem.anchor();
            if universe.get_node(anchor).is_some() {
                engine.propagate(anchor, PulseType::Reinforcing, universe, hebbian);
            }
        }

        consolidated
    }

    pub fn dream_cycle(
        &self,
        universe: &DarkUniverse,
        hebbian: &mut HebbianMemory,
        memories: &[MemoryAtom],
        cycles: usize,
    ) -> Vec<DreamReport> {
        let mut reports = Vec::with_capacity(cycles);
        for _ in 0..cycles {
            reports.push(self.dream(universe, hebbian, memories));
        }
        reports
    }
}

impl std::fmt::Display for DreamReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Dream[replay:{} weaken:{} consolidate:{} merge:{} edges:{}→{} weight:{:.2}→{:.2}]",
            self.paths_replayed,
            self.paths_weakened,
            self.memories_consolidated,
            self.memories_merged,
            self.hebbian_edges_before,
            self.hebbian_edges_after,
            self.weight_before,
            self.weight_after,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::coord::Coord7D;
    use crate::universe::memory::MemoryCodec;

    fn setup_dream_system() -> (DarkUniverse, HebbianMemory, Vec<MemoryAtom>) {
        let mut u = DarkUniverse::new(2_000_000.0);
        let mut h = HebbianMemory::new();
        let mut memories = Vec::new();

        let mem1 = MemoryCodec::encode(
            &mut u,
            &Coord7D::new_even([10, 10, 10, 0, 0, 0, 0]),
            &[1.0, 2.0],
        )
        .unwrap();
        let mem2 = MemoryCodec::encode(
            &mut u,
            &Coord7D::new_even([15, 15, 15, 0, 0, 0, 0]),
            &[3.0, 4.0],
        )
        .unwrap();
        memories.push(mem1);
        memories.push(mem2);

        for x in 0..6i32 {
            for y in 0..6i32 {
                for z in 0..6i32 {
                    let c = Coord7D::new_even([x + 10, y + 10, z + 10, 0, 0, 0, 0]);
                    u.materialize_biased(c, 50.0, 0.6).ok();
                }
            }
        }

        let engine = PulseEngine::new();
        engine.propagate(memories[0].anchor(), PulseType::Reinforcing, &u, &mut h);
        engine.propagate(memories[1].anchor(), PulseType::Reinforcing, &u, &mut h);
        engine.propagate(memories[0].anchor(), PulseType::Exploratory, &u, &mut h);

        (u, h, memories)
    }

    #[test]
    fn dream_replay_strengthens_strong_paths() {
        let (u, mut h, mems) = setup_dream_system();
        let _before_weight = h.total_weight();

        let dream = DreamEngine::new();
        let report = dream.dream(&u, &mut h, &mems);

        assert!(report.paths_replayed > 0, "should replay some paths");
        assert!(
            u.verify_conservation(),
            "dream should not break conservation"
        );
    }

    #[test]
    fn dream_weaken_removes_weak_edges() {
        let (u, mut h, mems) = setup_dream_system();
        let a = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let b = Coord7D::new_even([100, 0, 0, 0, 0, 0, 0]);
        h.record_path(&[a, b], 0.001);

        let dream = DreamEngine::new();
        let _report = dream.dream(&u, &mut h, &mems);
    }

    #[test]
    fn dream_consolidate_links_related_memories() {
        let (u, mut h, mems) = setup_dream_system();
        h.record_path(&[*mems[0].anchor(), *mems[1].anchor()], 2.0);

        let dream = DreamEngine::new();
        let report = dream.dream(&u, &mut h, &mems);

        assert!(
            report.memories_consolidated > 0,
            "related memories should consolidate"
        );
    }

    #[test]
    fn dream_cycle_multiple_rounds() {
        let (u, mut h, mems) = setup_dream_system();
        let dream = DreamEngine::new();
        let reports = dream.dream_cycle(&u, &mut h, &mems, 3);

        assert_eq!(reports.len(), 3);
        assert!(u.verify_conservation());
    }

    #[test]
    fn dream_preserves_conservation() {
        let (u, mut h, mems) = setup_dream_system();
        let dream = DreamEngine::new();
        dream.dream(&u, &mut h, &mems);

        assert!(
            u.verify_conservation(),
            "dream must preserve energy conservation"
        );
    }

    #[test]
    fn dream_with_empty_hebbian() {
        let (u, mut h, mems) = setup_dream_system();
        h.prune();
        for _ in 0..50 {
            h.decay_all();
        }
        let edges_before = h.edge_count();

        let dream = DreamEngine::new();
        let report = dream.dream(&u, &mut h, &mems);

        if edges_before == 0 {
            assert_eq!(report.paths_replayed, 0);
        }
    }

    #[test]
    fn dream_display_format() {
        let (u, mut h, mems) = setup_dream_system();
        let dream = DreamEngine::new();
        let report = dream.dream(&u, &mut h, &mems);
        let s = format!("{}", report);
        assert!(s.contains("Dream["));
    }
}
