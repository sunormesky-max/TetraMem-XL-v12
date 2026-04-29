use crate::universe::coord::Coord7D;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::lattice::Lattice;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use crate::universe::observer::{HealthReport, UniverseObserver};

#[derive(Debug, Clone)]
pub struct ScaleReport {
    pub energy_expanded_by: f64,
    pub nodes_added: usize,
    pub nodes_removed: usize,
    pub rebalanced: usize,
    pub reason: ScaleReason,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleReason {
    HighUtilization,
    LowUtilization,
    MemoryPressure,
    Manual,
    None,
}

impl std::fmt::Display for ScaleReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Scale[+{:.0}E +{}N -{}N rebal:{} reason:{:?}]",
            self.energy_expanded_by,
            self.nodes_added,
            self.nodes_removed,
            self.rebalanced,
            self.reason,
        )
    }
}

pub struct AutoScaleConfig {
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub scale_up_energy_factor: f64,
    pub scale_down_energy_factor: f64,
    pub growth_shell: i32,
    pub node_energy_amount: f64,
    pub node_physical_ratio: f64,
    pub compact_dark_ratio: f64,
    pub memory_pressure_threshold: f64,
}

impl Default for AutoScaleConfig {
    fn default() -> Self {
        Self {
            scale_up_threshold: 0.80,
            scale_down_threshold: 0.15,
            scale_up_energy_factor: 0.5,
            scale_down_energy_factor: 0.2,
            growth_shell: 2,
            node_energy_amount: 50.0,
            node_physical_ratio: 0.5,
            compact_dark_ratio: 0.05,
            memory_pressure_threshold: 0.90,
        }
    }
}

pub struct AutoScaler {
    config: AutoScaleConfig,
}

impl AutoScaler {
    pub fn new() -> Self {
        Self {
            config: AutoScaleConfig::default(),
        }
    }

    pub fn with_config(config: AutoScaleConfig) -> Self {
        Self { config }
    }

    pub fn evaluate(&self, report: &HealthReport) -> ScaleReason {
        if report.energy_utilization > self.config.memory_pressure_threshold {
            return ScaleReason::MemoryPressure;
        }
        if report.energy_utilization > self.config.scale_up_threshold {
            return ScaleReason::HighUtilization;
        }
        if report.energy_utilization < self.config.scale_down_threshold && report.node_count > 10 {
            return ScaleReason::LowUtilization;
        }
        ScaleReason::None
    }

    pub fn scale_up(
        &self,
        universe: &mut DarkUniverse,
        reason: ScaleReason,
    ) -> ScaleReport {
        let additional = universe.total_energy() * self.config.scale_up_energy_factor;
        let _ = universe.expand_energy_pool(additional);

        let center = Self::find_bounding_center(universe);
        let mut nodes_added = 0usize;
        let r = self.config.growth_shell;

        for dx in -r..=r {
            for dy in -r..=r {
                for dz in -r..=r {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    let ex = center[0] + dx;
                    let ey = center[1] + dy;
                    let ez = center[2] + dz;
                    let ec = Coord7D::new_even([ex, ey, ez, 0, 0, 0, 0]);
                    if !universe.contains(&ec) {
                        if universe
                            .materialize_biased(ec, self.config.node_energy_amount, self.config.node_physical_ratio)
                            .is_ok()
                        {
                            nodes_added += 1;
                        }
                    }
                    let oc = Coord7D::new_odd([ex, ey, ez, 0, 0, 0, 0]);
                    if !universe.contains(&oc) {
                        if universe
                            .materialize_biased(oc, self.config.node_energy_amount * 0.8, 0.3)
                            .is_ok()
                        {
                            nodes_added += 1;
                        }
                    }
                }
            }
        }

        ScaleReport {
            energy_expanded_by: additional,
            nodes_added,
            nodes_removed: 0,
            rebalanced: 0,
            reason,
        }
    }

    pub fn scale_down(
        &self,
        universe: &mut DarkUniverse,
        memories: &[MemoryAtom],
    ) -> ScaleReport {
        let mut nodes_removed = 0usize;
        let rebalanced = 0usize;

        let memory_anchors: std::collections::HashSet<Coord7D> = memories
            .iter()
            .flat_map(|m| m.vertices().to_vec())
            .collect();

        let coords: Vec<Coord7D> = universe.coords();
        for coord in &coords {
            if memory_anchors.contains(coord) {
                continue;
            }
            if let Some(node) = universe.get_node(coord) {
                if node.manifestation_ratio() < self.config.compact_dark_ratio {
                    universe.dematerialize(coord);
                    nodes_removed += 1;
                }
            }
        }

        let stats = universe.stats();
        let denominator = (1.0 - self.config.scale_down_energy_factor).max(0.01);
        let target_energy = stats.allocated_energy / denominator;
        if target_energy < universe.total_energy() {
            let excess = universe.total_energy() - target_energy;
            let shrink_amount = excess * 0.5;
            let available = universe.available_energy();
            if shrink_amount <= available {
                let _ = universe.shrink_energy_pool(shrink_amount);
            }
        }

        ScaleReport {
            energy_expanded_by: 0.0,
            nodes_added: 0,
            nodes_removed,
            rebalanced,
            reason: ScaleReason::LowUtilization,
        }
    }

    pub fn auto_scale(
        &self,
        universe: &mut DarkUniverse,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
    ) -> ScaleReport {
        let report = UniverseObserver::inspect(universe, hebbian, memories);
        let reason = self.evaluate(&report);

        match reason {
            ScaleReason::HighUtilization | ScaleReason::MemoryPressure => {
                self.scale_up(universe, reason)
            }
            ScaleReason::LowUtilization => self.scale_down(universe, memories),
            ScaleReason::Manual => self.scale_up(universe, ScaleReason::Manual),
            ScaleReason::None => ScaleReport {
                energy_expanded_by: 0.0,
                nodes_added: 0,
                nodes_removed: 0,
                rebalanced: 0,
                reason: ScaleReason::None,
            },
        }
    }

    pub fn scale_to_fit_memory(
        &self,
        universe: &mut DarkUniverse,
        data: &[f64],
    ) -> Result<ScaleReport, ()> {
        let physical_base = 50.0;
        let data_offset = 50.0;
        let estimated_energy = data.iter().map(|v| (v + data_offset).max(0.0)).sum::<f64>()
            + (physical_base + data_offset) * 3.0 * 4.0
            + data_offset * 4.0 * 4.0;

        let mut report = ScaleReport {
            energy_expanded_by: 0.0,
            nodes_added: 0,
            nodes_removed: 0,
            rebalanced: 0,
            reason: ScaleReason::Manual,
        };

        if universe.available_energy() < estimated_energy {
            let needed = estimated_energy - universe.available_energy();
            let expansion = needed * 2.0;
            let _ = universe.expand_energy_pool(expansion);
            report.energy_expanded_by = expansion;
        }

        let center = Self::find_bounding_center(universe);
        for dx in -2..=2i32 {
            for dy in -2..=2i32 {
                for dz in -2..=2i32 {
                    let ec = Coord7D::new_even([
                        center[0] + dx,
                        center[1] + dy,
                        center[2] + dz,
                        0, 0, 0, 0,
                    ]);
                    if !universe.contains(&ec) {
                        if universe
                            .materialize_biased(ec, self.config.node_energy_amount, 0.6)
                            .is_ok()
                        {
                            report.nodes_added += 1;
                        }
                    }
                    let oc = Coord7D::new_odd([
                        center[0] + dx,
                        center[1] + dy,
                        center[2] + dz,
                        0, 0, 0, 0,
                    ]);
                    if !universe.contains(&oc) {
                        if universe
                            .materialize_biased(oc, self.config.node_energy_amount * 0.8, 0.3)
                            .is_ok()
                        {
                            report.nodes_added += 1;
                        }
                    }
                }
            }
        }

        Ok(report)
    }

    pub fn scale_near_anchor(
        &self,
        universe: &mut DarkUniverse,
        _anchor: &Coord7D,
        data: &[f64],
    ) -> Result<ScaleReport, ()> {
        let physical_base = 50.0;
        let data_offset = 50.0;
        let encode_energy = data.iter().map(|v| (v + data_offset).max(0.0)).sum::<f64>()
            + (physical_base + data_offset) * 3.0 * 4.0
            + data_offset * 4.0 * 4.0;

        let mut report = ScaleReport {
            energy_expanded_by: 0.0,
            nodes_added: 0,
            nodes_removed: 0,
            rebalanced: 0,
            reason: ScaleReason::Manual,
        };

        if universe.available_energy() < encode_energy {
            let needed = encode_energy - universe.available_energy();
            let expansion = needed * 3.0;
            let _ = universe.expand_energy_pool(expansion);
            report.energy_expanded_by = expansion;
        }

        Ok(report)
    }

    fn find_bounding_center(universe: &DarkUniverse) -> [i32; 3] {
        let coords = universe.coords();
        if coords.is_empty() {
            return [0, 0, 0];
        }

        let mut min = [i32::MAX; 3];
        let mut max = [i32::MIN; 3];
        for c in &coords {
            let p = c.physical();
            for d in 0..3 {
                min[d] = min[d].min(p[d]);
                max[d] = max[d].max(p[d]);
            }
        }
        [(min[0] + max[0]) / 2, (min[1] + max[1]) / 2, (min[2] + max[2]) / 2]
    }

    pub fn frontier_expansion(
        &self,
        universe: &mut DarkUniverse,
        max_new: usize,
    ) -> ScaleReport {
        let coords = universe.coords();
        let mut frontier = Vec::new();

        for coord in &coords {
            for n in Lattice::face_neighbor_coords(coord) {
                if !universe.contains(&n) {
                    frontier.push(n);
                }
            }
        }

        frontier.sort();
        frontier.dedup();

        let mut added = 0usize;
        for c in frontier.iter().take(max_new) {
            if universe
                .materialize_biased(*c, self.config.node_energy_amount, 0.4)
                .is_ok()
            {
                added += 1;
            }
        }

        ScaleReport {
            energy_expanded_by: 0.0,
            nodes_added: added,
            nodes_removed: 0,
            rebalanced: 0,
            reason: ScaleReason::Manual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::memory::MemoryCodec;

    fn make_small_universe() -> DarkUniverse {
        let mut u = DarkUniverse::new(10000.0);
        for x in 0..3i32 {
            for y in 0..3i32 {
                for z in 0..3i32 {
                    let c = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
                    u.materialize_biased(c, 50.0, 0.6).ok();
                }
            }
        }
        u
    }

    #[test]
    fn evaluate_high_utilization() {
        let mut u = DarkUniverse::new(100.0);
        u.materialize_biased(Coord7D::new_even([0; 7]), 85.0, 0.6).unwrap();
        let h = HebbianMemory::new();
        let report = UniverseObserver::inspect(&u, &h, &[]);

        let scaler = AutoScaler::new();
        let reason = scaler.evaluate(&report);
        assert_eq!(reason, ScaleReason::HighUtilization);
    }

    #[test]
    fn evaluate_low_utilization() {
        let u = DarkUniverse::new(10000.0);
        let h = HebbianMemory::new();
        let report = UniverseObserver::inspect(&u, &h, &[]);
        let scaler = AutoScaler::new();
        assert_eq!(scaler.evaluate(&report), ScaleReason::None);
    }

    #[test]
    fn scale_up_adds_energy_and_nodes() {
        let mut u = make_small_universe();
        let before_energy = u.total_energy();
        let _before_nodes = u.active_node_count();

        let scaler = AutoScaler::new();
        let report = scaler.scale_up(&mut u, ScaleReason::HighUtilization);

        assert!(report.energy_expanded_by > 0.0);
        assert!(u.total_energy() > before_energy);
        assert!(u.verify_conservation());
    }

    #[test]
    fn scale_down_removes_dark_nodes() {
        let mut u = DarkUniverse::new(100000.0);
        let c1 = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let c2 = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c3 = Coord7D::new_even([2, 0, 0, 0, 0, 0, 0]);
        u.materialize_biased(c1, 100.0, 0.8).unwrap();
        u.materialize_biased(c2, 100.0, 0.01).unwrap();
        u.materialize_biased(c3, 100.0, 0.01).unwrap();

        let mems = Vec::new();
        let scaler = AutoScaler::new();
        let _report = scaler.scale_down(&mut u, &mems);

        assert!(u.verify_conservation());
    }

    #[test]
    fn auto_scale_triggers_on_high_util() {
        let mut u = DarkUniverse::new(200.0);
        u.materialize_biased(Coord7D::new_even([0; 7]), 180.0, 0.6).unwrap();
        let h = HebbianMemory::new();

        let scaler = AutoScaler::new();
        let report = scaler.auto_scale(&mut u, &h, &[]);

        assert_ne!(report.reason, ScaleReason::None);
        assert!(u.verify_conservation());
    }

    #[test]
    fn auto_scale_noop_when_balanced() {
        let mut u = make_small_universe();
        let h = HebbianMemory::new();

        let mut config = AutoScaleConfig::default();
        config.scale_down_threshold = 0.05;
        let scaler = AutoScaler::with_config(config);
        let report = scaler.auto_scale(&mut u, &h, &[]);

        assert_eq!(report.reason, ScaleReason::None);
    }

    #[test]
    fn scale_to_fit_memory_expands() {
        let mut u = DarkUniverse::new(100.0);
        let scaler = AutoScaler::new();
        let data = vec![100.0; 20];
        let report = scaler.scale_to_fit_memory(&mut u, &data).unwrap();

        assert!(report.energy_expanded_by > 0.0);
        assert!(u.total_energy() > 100.0);
    }

    #[test]
    fn frontier_expansion_grows_lattice() {
        let mut u = DarkUniverse::new(100000.0);
        u.materialize_biased(Coord7D::new_even([5, 5, 5, 0, 0, 0, 0]), 100.0, 0.6).unwrap();
        let before = u.active_node_count();

        let scaler = AutoScaler::new();
        let report = scaler.frontier_expansion(&mut u, 20);

        assert!(report.nodes_added > 0);
        assert!(u.active_node_count() > before);
        assert!(u.verify_conservation());
    }

    #[test]
    fn scale_preserves_memories() {
        let mut u = DarkUniverse::new(1_000_000.0);
        let anchor = Coord7D::new_even([10, 10, 10, 0, 0, 0, 0]);
        let mem = MemoryCodec::encode(&mut u, &anchor, &[1.0, 2.0, 3.0]).unwrap();

        let h = HebbianMemory::new();
        let scaler = AutoScaler::new();
        scaler.auto_scale(&mut u, &h, &[mem.clone()]);

        let decoded = MemoryCodec::decode(&u, &mem).unwrap();
        assert!((decoded[0] - 1.0).abs() < 1e-10);
        assert!((decoded[1] - 2.0).abs() < 1e-10);
        assert!((decoded[2] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn scale_report_display() {
        let report = ScaleReport {
            energy_expanded_by: 500.0,
            nodes_added: 10,
            nodes_removed: 2,
            rebalanced: 0,
            reason: ScaleReason::HighUtilization,
        };
        let s = format!("{}", report);
        assert!(s.contains("Scale["));
    }
}
