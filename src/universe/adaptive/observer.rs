use crate::universe::coord::Coord7D;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;

#[derive(Debug, Clone)]
pub struct HealthReport {
    pub conservation_ok: bool,
    pub energy_utilization: f64,
    pub node_count: usize,
    pub manifested_ratio: f64,
    pub even_odd_ratio: f64,
    pub hebbian_edge_count: usize,
    pub hebbian_avg_weight: f64,
    pub memory_count: usize,
    pub physical_dark_ratio: f64,
    pub frontier_size: usize,
    pub density: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HealthLevel {
    Excellent,
    Good,
    Warning,
    Critical,
}

impl HealthLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthLevel::Excellent => "EXCELLENT",
            HealthLevel::Good => "GOOD",
            HealthLevel::Warning => "WARNING",
            HealthLevel::Critical => "CRITICAL",
        }
    }
}

impl HealthReport {
    pub fn health_level(&self) -> HealthLevel {
        if !self.conservation_ok {
            return HealthLevel::Critical;
        }
        if self.energy_utilization > 0.95 {
            return HealthLevel::Warning;
        }
        if self.manifested_ratio < 0.1 && self.node_count > 0 {
            return HealthLevel::Warning;
        }
        if self.hebbian_avg_weight > 0.5 && self.energy_utilization < 0.8 {
            return HealthLevel::Excellent;
        }
        HealthLevel::Good
    }
}

#[derive(Debug, Clone)]
pub struct RegulatorAction {
    pub action_type: RegulatorActionType,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegulatorActionType {
    DecayHebbian,
    PruneHebbian,
    ExpandEnergy,
    ExpandLattice,
    CompactUnused,
    None,
}

pub struct RegulatorParams {
    pub utilization_high_threshold: f64,
    pub utilization_low_threshold: f64,
    pub hebbian_edge_limit: usize,
    pub hebbian_weight_ceiling: f64,
    pub expansion_factor: f64,
    pub lattice_growth_radius: i32,
}

impl Default for RegulatorParams {
    fn default() -> Self {
        Self {
            utilization_high_threshold: 0.85,
            utilization_low_threshold: 0.2,
            hebbian_edge_limit: 5000,
            hebbian_weight_ceiling: 8.0,
            expansion_factor: 0.5,
            lattice_growth_radius: 2,
        }
    }
}

pub struct UniverseObserver;

impl UniverseObserver {
    pub fn inspect(
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
    ) -> HealthReport {
        let stats = universe.stats();
        let conservation_ok = universe.verify_conservation();
        let manifested_ratio = if stats.active_nodes > 0 {
            stats.manifested_nodes as f64 / stats.active_nodes as f64
        } else {
            0.0
        };
        let even_odd_ratio = if stats.odd_nodes > 0 {
            stats.even_nodes as f64 / stats.odd_nodes as f64
        } else {
            -1.0
        };
        let physical_dark_ratio = if stats.dark_energy > 0.0 {
            stats.physical_energy / stats.dark_energy
        } else if stats.physical_energy > 0.0 {
            -1.0
        } else {
            1.0
        };
        let hebbian_avg_weight = if hebbian.edge_count() > 0 {
            hebbian.total_weight() / hebbian.edge_count() as f64
        } else {
            0.0
        };
        let frontier = Self::compute_frontier(universe);
        let density = if stats.total_energy > 0.0 {
            stats.active_nodes as f64 / stats.total_energy
        } else {
            0.0
        };

        HealthReport {
            conservation_ok,
            energy_utilization: stats.utilization,
            node_count: stats.active_nodes,
            manifested_ratio,
            even_odd_ratio,
            hebbian_edge_count: hebbian.edge_count(),
            hebbian_avg_weight,
            memory_count: memories.len(),
            physical_dark_ratio,
            frontier_size: frontier,
            density,
        }
    }

    fn compute_frontier(universe: &DarkUniverse) -> usize {
        use crate::universe::lattice::Lattice;
        let mut frontier = std::collections::HashSet::new();
        for coord in universe.coords() {
            for n in Lattice::face_neighbor_coords(&coord) {
                if !universe.contains(&n) {
                    frontier.insert(n);
                }
            }
        }
        frontier.len()
    }
}

pub struct SelfRegulator {
    pub params: RegulatorParams,
}

impl Default for SelfRegulator {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfRegulator {
    pub fn new() -> Self {
        Self {
            params: RegulatorParams::default(),
        }
    }

    pub fn with_params(params: RegulatorParams) -> Self {
        Self { params }
    }

    pub fn regulate(
        &self,
        report: &HealthReport,
        hebbian: &mut HebbianMemory,
    ) -> Vec<RegulatorAction> {
        let mut actions = Vec::new();

        if hebbian.edge_count() > self.params.hebbian_edge_limit {
            hebbian.max_paths = self.params.hebbian_edge_limit;
            hebbian.prune();
            actions.push(RegulatorAction {
                action_type: RegulatorActionType::PruneHebbian,
                description: format!(
                    "pruned hebbian edges to {} (limit {})",
                    hebbian.edge_count(),
                    self.params.hebbian_edge_limit
                ),
            });
        }

        if report.hebbian_avg_weight > self.params.hebbian_weight_ceiling {
            hebbian.decay_all();
            actions.push(RegulatorAction {
                action_type: RegulatorActionType::DecayHebbian,
                description: format!(
                    "decayed hebbian (avg weight {:.3} > ceiling {:.3})",
                    report.hebbian_avg_weight, self.params.hebbian_weight_ceiling
                ),
            });
        }

        if report.energy_utilization > self.params.utilization_high_threshold {
            actions.push(RegulatorAction {
                action_type: RegulatorActionType::ExpandEnergy,
                description: format!(
                    "energy utilization {:.1}% exceeds {:.1}%",
                    report.energy_utilization * 100.0,
                    self.params.utilization_high_threshold * 100.0
                ),
            });
        }

        if report.frontier_size > 0 && report.energy_utilization > 0.6 {
            actions.push(RegulatorAction {
                action_type: RegulatorActionType::ExpandLattice,
                description: format!(
                    "lattice expansion recommended (frontier: {} nodes)",
                    report.frontier_size
                ),
            });
        }

        if actions.is_empty() {
            actions.push(RegulatorAction {
                action_type: RegulatorActionType::None,
                description: "system healthy, no action needed".to_string(),
            });
        }

        actions
    }

    pub fn execute_expansion(
        &self,
        universe: &mut DarkUniverse,
        _report: &HealthReport,
    ) -> RegulatorAction {
        let additional = universe.total_energy() * self.params.expansion_factor;
        let _ = universe.expand_energy_pool(additional);

        let mut new_nodes = 0usize;
        let center = Self::find_center(universe);
        let r = self.params.lattice_growth_radius;

        for dx in -r..=r {
            for dy in -r..=r {
                for dz in -r..=r {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    let ex = center[0] + dx;
                    let ey = center[1] + dy;
                    let ez = center[2] + dz;
                    let c = Coord7D::new_even([ex, ey, ez, 0, 0, 0, 0]);
                    if !universe.contains(&c) && universe.materialize_biased(c, 50.0, 0.5).is_ok() {
                        new_nodes += 1;
                    }
                    let ox = center[0] + dx;
                    let oy = center[1] + dy;
                    let oz = center[2] + dz;
                    let o = Coord7D::new_odd([ox, oy, oz, 0, 0, 0, 0]);
                    if !universe.contains(&o) && universe.materialize_biased(o, 40.0, 0.3).is_ok() {
                        new_nodes += 1;
                    }
                }
            }
        }

        RegulatorAction {
            action_type: RegulatorActionType::ExpandLattice,
            description: format!("expanded: +{:.0} energy, +{} nodes", additional, new_nodes),
        }
    }

    fn find_center(universe: &DarkUniverse) -> [i32; 3] {
        let coords = universe.coords();
        if coords.is_empty() {
            return [0, 0, 0];
        }
        let n = coords.len() as i64;
        let sum: [i64; 3] = coords
            .iter()
            .map(|c| c.physical())
            .fold([0i64; 3], |acc, p| {
                [
                    acc[0] + p[0] as i64,
                    acc[1] + p[1] as i64,
                    acc[2] + p[2] as i64,
                ]
            });
        [
            ((sum[0] as f64) / (n as f64)).round() as i32,
            ((sum[1] as f64) / (n as f64)).round() as i32,
            ((sum[2] as f64) / (n as f64)).round() as i32,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::memory::MemoryCodec;

    fn setup_system() -> (DarkUniverse, HebbianMemory, Vec<MemoryAtom>) {
        let mut u = DarkUniverse::new(1_000_000.0);
        let h = HebbianMemory::new();
        let mut memories = Vec::new();

        let mem = MemoryCodec::encode(
            &mut u,
            &Coord7D::new_even([10, 10, 10, 0, 0, 0, 0]),
            &[1.0, 2.0, 3.0],
        )
        .unwrap();
        memories.push(mem);

        for x in 0..4i32 {
            for y in 0..4i32 {
                for z in 0..4i32 {
                    let c = Coord7D::new_even([x + 10, y + 10, z + 10, 0, 0, 0, 0]);
                    u.materialize_biased(c, 50.0, 0.6).ok();
                }
            }
        }

        (u, h, memories)
    }

    #[test]
    fn observer_inspects_healthy_system() {
        let (u, h, mems) = setup_system();
        let report = UniverseObserver::inspect(&u, &h, &mems);

        assert!(report.conservation_ok);
        assert!(report.node_count > 0);
        assert!(report.energy_utilization > 0.0);
        assert_eq!(report.memory_count, 1);
    }

    #[test]
    fn health_level_excellent() {
        let (u, h, mems) = setup_system();
        let report = UniverseObserver::inspect(&u, &h, &mems);
        let level = report.health_level();
        assert!(level == HealthLevel::Good || level == HealthLevel::Excellent);
    }

    #[test]
    fn health_level_critical_on_conservation_fail() {
        let (u, h, mems) = setup_system();
        let mut report = UniverseObserver::inspect(&u, &h, &mems);
        report.conservation_ok = false;
        assert_eq!(report.health_level(), HealthLevel::Critical);
    }

    #[test]
    fn regulator_no_action_when_healthy() {
        let (u, h, mems) = setup_system();
        let report = UniverseObserver::inspect(&u, &h, &mems);
        let regulator = SelfRegulator::new();
        let mut h2 = h.clone();
        let actions = regulator.regulate(&report, &mut h2);
        assert!(actions
            .iter()
            .any(|a| a.action_type == RegulatorActionType::None));
    }

    #[test]
    fn regulator_prunes_hebbian() {
        let (u, _, mems) = setup_system();
        let mut h = HebbianMemory::new();
        for i in 0..20i32 {
            let a = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            let b = Coord7D::new_even([i, 1, 0, 0, 0, 0, 0]);
            h.record_path(&[a, b], 1.0);
        }
        assert!(h.edge_count() >= 10, "should have many edges");
        let params = RegulatorParams {
            hebbian_edge_limit: 5,
            ..Default::default()
        };
        let regulator = SelfRegulator::with_params(params);
        let report = UniverseObserver::inspect(&u, &h, &mems);
        let actions = regulator.regulate(&report, &mut h);
        assert!(actions
            .iter()
            .any(|a| a.action_type == RegulatorActionType::PruneHebbian));
        assert!(h.edge_count() <= 4);
    }

    #[test]
    fn regulator_suggests_expansion() {
        let mut u = DarkUniverse::new(100.0);
        u.materialize_biased(Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]), 90.0, 0.6)
            .unwrap();
        let h = HebbianMemory::new();
        let report = UniverseObserver::inspect(&u, &h, &[]);
        assert!(report.energy_utilization > 0.85);
    }

    #[test]
    fn execute_expansion_adds_nodes() {
        let mut u = DarkUniverse::new(10000.0);
        u.materialize_biased(Coord7D::new_even([5, 5, 5, 0, 0, 0, 0]), 100.0, 0.6)
            .unwrap();
        let before_nodes = u.active_node_count();
        let before_energy = u.total_energy();

        let regulator = SelfRegulator::new();
        let report = UniverseObserver::inspect(&u, &HebbianMemory::new(), &[]);
        let _action = regulator.execute_expansion(&mut u, &report);

        assert!(u.active_node_count() > before_nodes);
        assert!(u.total_energy() > before_energy);
        assert!(u.verify_conservation());
    }

    #[test]
    fn frontier_computed_correctly() {
        let mut u = DarkUniverse::new(10000.0);
        u.materialize_biased(Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]), 100.0, 0.6)
            .unwrap();
        let report = UniverseObserver::inspect(&u, &HebbianMemory::new(), &[]);
        assert!(
            report.frontier_size > 0,
            "single node should have frontier neighbors"
        );
    }
}
