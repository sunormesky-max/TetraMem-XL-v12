// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::autoscale::AutoScaler;
use crate::universe::crystal::CrystalEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use crate::universe::observer::{HealthReport, UniverseObserver};

#[derive(Debug, Clone)]
pub struct DimensionPressure {
    pub dims: [f64; 7],
    pub max_pressure_dim: usize,
    pub max_pressure_value: f64,
    pub min_pressure_dim: usize,
    pub min_pressure_value: f64,
    pub imbalance: f64,
}

#[derive(Debug, Clone)]
pub struct RegulationReport {
    pub dimension_pressure: DimensionPressure,
    pub actions: Vec<RegAction>,
    pub stress_level: f64,
    pub entropy: f64,
}

#[derive(Debug, Clone)]
pub struct RegAction {
    pub action: String,
    pub detail: String,
}

impl std::fmt::Display for RegulationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Reg[stress:{:.2} entropy:{:.2} pressure_dim:{} imbalance:{:.2} actions:{}]",
            self.stress_level,
            self.entropy,
            self.dimension_pressure.max_pressure_dim,
            self.dimension_pressure.imbalance,
            self.actions.len()
        )
    }
}

pub struct RegulationEngine {
    pub pressure_threshold: f64,
    pub entropy_target: f64,
    pub stress_threshold: f64,
    pub crystal_decay_threshold: usize,
    pub hebbian_target_avg: f64,
}

impl Default for RegulationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RegulationEngine {
    pub fn new() -> Self {
        Self {
            pressure_threshold: 3.0,
            entropy_target: 0.5,
            stress_threshold: 0.8,
            crystal_decay_threshold: 5000,
            hebbian_target_avg: 2.0,
        }
    }

    pub fn with_target_avg(mut self, target: f64) -> Self {
        self.hebbian_target_avg = target;
        self
    }

    pub fn regulate(
        &self,
        universe: &mut DarkUniverse,
        hebbian: &mut HebbianMemory,
        crystal: &mut CrystalEngine,
        memories: &[MemoryAtom],
    ) -> RegulationReport {
        let report = UniverseObserver::inspect(universe, hebbian, memories);
        let pressure = self.compute_dimension_pressure(universe);
        let entropy = self.compute_entropy(universe);
        let stress = self.compute_stress(&report, &pressure);
        let mut actions = Vec::new();

        if pressure.imbalance > self.pressure_threshold {
            let dim = pressure.max_pressure_dim;
            let drained = self.redistribute_from_dimension(universe, dim, 0.1);
            actions.push(RegAction {
                action: "dimension_rebalance".to_string(),
                detail: format!(
                    "drained {:.1} from dim {} (pressure {:.2})",
                    drained, dim, pressure.max_pressure_value
                ),
            });
        }

        if entropy < self.entropy_target * 0.5 {
            hebbian.decay_all();
            actions.push(RegAction {
                action: "entropy_boost".to_string(),
                detail: format!(
                    "entropy {:.3} below target {:.3}, decay applied",
                    entropy, self.entropy_target
                ),
            });
        }

        if report.hebbian_edge_count > self.crystal_decay_threshold {
            let active: std::collections::HashSet<crate::universe::coord::Coord7D> =
                universe.coords().into_iter().collect();
            let removed = crystal.decay_unused(&active);
            if removed > 0 {
                actions.push(RegAction {
                    action: "crystal_cleanup".to_string(),
                    detail: format!("removed {} orphaned crystals", removed),
                });
            }
        }

        if stress > self.stress_threshold {
            let scaler = AutoScaler::new();
            let scale_report = scaler.auto_scale(universe, hebbian, memories);
            if scale_report.nodes_added > 0 || scale_report.energy_expanded_by > 0.0 {
                actions.push(RegAction {
                    action: "stress_expansion".to_string(),
                    detail: format!(
                        "+{} nodes +{:.0} energy",
                        scale_report.nodes_added, scale_report.energy_expanded_by
                    ),
                });
            }
        }

        hebbian.normalize_weights(self.hebbian_target_avg);
        let avg_w = if hebbian.edge_count() > 0 {
            hebbian.total_weight() / hebbian.edge_count() as f64
        } else {
            0.0
        };
        if avg_w > self.hebbian_target_avg * 1.1 {
            actions.push(RegAction {
                action: "hebbian_normalization".to_string(),
                detail: format!("avg_weight normalized to {:.2}", avg_w),
            });
        }

        RegulationReport {
            dimension_pressure: pressure,
            actions,
            stress_level: stress,
            entropy,
        }
    }

    fn compute_dimension_pressure(&self, universe: &DarkUniverse) -> DimensionPressure {
        let mut dim_totals = [0.0f64; 7];
        let mut count = 0usize;

        for coord in universe.coords_iter() {
            if let Some(node) = universe.get_node(&coord) {
                for (d, &v) in node.energy().dims().iter().enumerate() {
                    dim_totals[d] += v;
                }
                count += 1;
            }
        }

        if count == 0 {
            return DimensionPressure {
                dims: [0.0; 7],
                max_pressure_dim: 0,
                max_pressure_value: 0.0,
                min_pressure_dim: 0,
                min_pressure_value: 0.0,
                imbalance: 0.0,
            };
        }

        let mean = dim_totals.iter().sum::<f64>() / 7.0;
        let mut max_dim = 0usize;
        let mut max_val = 0.0f64;
        let mut min_dim = 0usize;
        let mut min_val = f64::MAX;

        for (d, &total) in dim_totals.iter().enumerate() {
            if total > max_val {
                max_val = total;
                max_dim = d;
            }
            if total < min_val {
                min_val = total;
                min_dim = d;
            }
        }

        let imbalance = if mean > 0.0 {
            (max_val - min_val) / mean
        } else {
            0.0
        };

        DimensionPressure {
            dims: dim_totals,
            max_pressure_dim: max_dim,
            max_pressure_value: max_val,
            min_pressure_dim: min_dim,
            min_pressure_value: min_val,
            imbalance,
        }
    }

    fn compute_entropy(&self, universe: &DarkUniverse) -> f64 {
        let stats = universe.stats();
        if stats.active_nodes == 0 {
            return 0.0;
        }
        let p_manifested = stats.manifested_nodes as f64 / stats.active_nodes as f64;
        let p_dark = 1.0 - p_manifested;
        let mut entropy = 0.0;
        if p_manifested > 0.0 {
            entropy -= p_manifested * p_manifested.ln();
        }
        if p_dark > 0.0 {
            entropy -= p_dark * p_dark.ln();
        }
        entropy
    }

    fn compute_stress(&self, report: &HealthReport, pressure: &DimensionPressure) -> f64 {
        let util_stress = if report.energy_utilization > 0.8 {
            (report.energy_utilization - 0.8) * 5.0
        } else {
            0.0
        };
        let pressure_stress = pressure.imbalance / self.pressure_threshold;
        let edge_stress = if report.hebbian_edge_count > 3000 {
            0.3
        } else {
            0.0
        };

        (util_stress + pressure_stress + edge_stress).min(1.0)
    }

    fn redistribute_from_dimension(
        &self,
        universe: &mut DarkUniverse,
        dim: usize,
        fraction: f64,
    ) -> f64 {
        let coords = universe.coords();
        let mut total_drained = 0.0f64;

        for coord in &coords {
            if let Some(node) = universe.get_node_mut(coord) {
                match node.energy_mut().redistribute_dim(dim, fraction) {
                    Ok(drained) => total_drained += drained,
                    Err(e) => {
                        tracing::debug!(
                            "redistribute_dim({}) failed for {:?}: {:?}",
                            dim,
                            coord,
                            e
                        );
                        continue;
                    }
                }
            }
        }

        total_drained
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::coord::Coord7D;
    use crate::universe::memory::MemoryCodec;

    fn setup() -> (DarkUniverse, HebbianMemory, CrystalEngine, Vec<MemoryAtom>) {
        let mut u = DarkUniverse::new(1_000_000.0);
        let h = HebbianMemory::new();

        let m = MemoryCodec::encode(
            &mut u,
            &Coord7D::new_even([10, 10, 10, 0, 0, 0, 0]),
            &[1.0, 2.0],
        )
        .unwrap();

        for x in 0..4i32 {
            for y in 0..4i32 {
                for z in 0..4i32 {
                    let c = Coord7D::new_even([x + 10, y + 10, z + 10, 0, 0, 0, 0]);
                    u.materialize_biased(c, 50.0, 0.6).ok();
                    let c2 = Coord7D::new_odd([x + 10, y + 10, z + 10, 0, 0, 0, 0]);
                    u.materialize_biased(c2, 40.0, 0.2).ok();
                }
            }
        }

        let mut crystal = CrystalEngine::new();
        crystal.crystallize(&h, &u);

        (u, h, crystal, vec![m])
    }

    #[test]
    fn regulate_balanced_system() {
        let (mut u, mut h, mut crystal, mems) = setup();
        let engine = RegulationEngine::new();
        let report = engine.regulate(&mut u, &mut h, &mut crystal, &mems);

        assert!(report.stress_level <= 1.0);
        assert!(report.entropy >= 0.0);
    }

    #[test]
    fn dimension_pressure_computed() {
        let (u, h, _crystal, mems) = setup();
        let engine = RegulationEngine::new();
        let _report = UniverseObserver::inspect(&u, &h, &mems);
        let pressure = engine.compute_dimension_pressure(&u);

        assert!(pressure.max_pressure_value >= pressure.min_pressure_value);
        assert!(pressure.imbalance >= 0.0);
    }

    #[test]
    fn entropy_nonzero_with_nodes() {
        let (u, _, _, _) = setup();
        let engine = RegulationEngine::new();
        let entropy = engine.compute_entropy(&u);
        assert!(
            entropy > 0.0,
            "system with both manifested and dark nodes should have entropy"
        );
    }

    #[test]
    fn regulation_report_display() {
        let (mut u, mut h, mut crystal, mems) = setup();
        let engine = RegulationEngine::new();
        let report = engine.regulate(&mut u, &mut h, &mut crystal, &mems);
        let s = format!("{}", report);
        assert!(s.contains("Reg["));
    }

    #[test]
    fn stress_high_on_overutilization() {
        let mut u = DarkUniverse::new(100.0);
        u.materialize_biased(Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]), 95.0, 0.9)
            .unwrap();
        let h = HebbianMemory::new();
        let mems = Vec::new();

        let report = UniverseObserver::inspect(&u, &h, &mems);
        let engine = RegulationEngine::new();
        let pressure = engine.compute_dimension_pressure(&u);
        let stress = engine.compute_stress(&report, &pressure);

        assert!(stress > 0.5, "over-utilized system should be stressed");
    }
}
