use serde::{Deserialize, Serialize};
use std::time::Instant;

const DEFAULT_BUDGET_RATIO: f64 = 0.05;
const MIN_PERCEPTION_ENERGY: f64 = 10.0;
const QUALITY_SCALE: f64 = 100.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionBudget {
    total_budget: f64,
    allocated: f64,
    spent: f64,
    returned: f64,
    max_single_perception: f64,
    topology_weight: [f64; 7],
}

#[derive(Debug, Clone)]
pub struct PerceptionAlloc {
    id: u64,
    amount: f64,
    topology_boost: f64,
    created_at: Instant,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerceptionReport {
    pub total_budget: f64,
    pub allocated: f64,
    pub spent: f64,
    pub returned: f64,
    pub utilization: f64,
    pub active_perceptions: usize,
}

#[derive(Debug)]
pub enum PerceptionError {
    InsufficientBudget { requested: f64, available: f64 },
    OverAllocate { requested: f64, max: f64 },
    InvalidReturn { returned: f64, spent: f64 },
    UnknownAllocation { id: u64 },
}

impl std::fmt::Display for PerceptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PerceptionError::InsufficientBudget { requested, available } => {
                write!(f, "insufficient perception budget: requested={}, available={}", requested, available)
            }
            PerceptionError::OverAllocate { requested, max } => {
                write!(f, "single perception exceeds max: requested={}, max={}", requested, max)
            }
            PerceptionError::InvalidReturn { returned, spent } => {
                write!(f, "return exceeds spent: returned={}, spent={}", returned, spent)
            }
            PerceptionError::UnknownAllocation { id } => {
                write!(f, "unknown perception allocation: {}", id)
            }
        }
    }
}

impl std::error::Error for PerceptionError {}

impl PerceptionBudget {
    pub fn new(total_universe_energy: f64) -> Self {
        let total_budget = (total_universe_energy * DEFAULT_BUDGET_RATIO).max(MIN_PERCEPTION_ENERGY);
        Self {
            total_budget,
            allocated: 0.0,
            spent: 0.0,
            returned: 0.0,
            max_single_perception: total_budget * 0.25,
            topology_weight: [1.0, 1.2, 1.5, 1.8, 2.0, 2.5, 3.0],
        }
    }

    pub fn with_budget(total_budget: f64) -> Self {
        let max = (total_budget * 0.25).max(1.0);
        Self {
            total_budget: total_budget.max(MIN_PERCEPTION_ENERGY),
            allocated: 0.0,
            spent: 0.0,
            returned: 0.0,
            max_single_perception: max,
            topology_weight: [1.0, 1.2, 1.5, 1.8, 2.0, 2.5, 3.0],
        }
    }

    pub fn available(&self) -> f64 {
        self.total_budget - self.allocated
    }

    pub fn allocate(
        &mut self,
        base_cost: f64,
        topology_level: usize,
    ) -> Result<PerceptionAlloc, PerceptionError> {
        let level = topology_level.min(6);
        let boost = self.topology_weight[level];
        let amount = base_cost * boost;

        if amount > self.max_single_perception {
            return Err(PerceptionError::OverAllocate {
                requested: amount,
                max: self.max_single_perception,
            });
        }

        if amount > self.available() {
            return Err(PerceptionError::InsufficientBudget {
                requested: amount,
                available: self.available(),
            });
        }

        self.allocated += amount;

        Ok(PerceptionAlloc {
            id: Self::next_id(),
            amount,
            topology_boost: boost,
            created_at: Instant::now(),
        })
    }

    pub fn settle(&mut self, alloc: PerceptionAlloc, actual_cost: f64) -> Result<f64, PerceptionError> {
        let used = actual_cost.min(alloc.amount);
        let refund = alloc.amount - used;

        self.allocated -= alloc.amount;
        self.spent += used;
        self.returned += refund;

        Ok(refund)
    }

    pub fn quality_output(&self, energy_spent: f64, topology_level: usize) -> f64 {
        let level = topology_level.min(6) as f64;
        let boost = self.topology_weight[topology_level.min(6)];
        (energy_spent * boost * level / QUALITY_SCALE).min(1.0)
    }

    pub fn report(&self) -> PerceptionReport {
        PerceptionReport {
            total_budget: self.total_budget,
            allocated: self.allocated,
            spent: self.spent,
            returned: self.returned,
            utilization: if self.total_budget > 0.0 {
                self.spent / self.total_budget
            } else {
                0.0
            },
            active_perceptions: 0,
        }
    }

    pub fn replenish(&mut self, total_universe_energy: f64) {
        let new_budget = (total_universe_energy * DEFAULT_BUDGET_RATIO).max(MIN_PERCEPTION_ENERGY);
        self.total_budget = new_budget;
        self.max_single_perception = new_budget * 0.25;
    }

    fn next_id() -> u64 {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_creation() {
        let pb = PerceptionBudget::new(1_000_000.0);
        assert!(pb.total_budget > 0.0);
        assert!(pb.available() == pb.total_budget);
        assert!(pb.max_single_perception < pb.total_budget);
    }

    #[test]
    fn allocate_and_settle() {
        let mut pb = PerceptionBudget::with_budget(1000.0);
        let alloc = pb.allocate(10.0, 0).unwrap();
        assert!((pb.available() - (1000.0 - alloc.amount)).abs() < 1e-10);

        let refund = pb.settle(alloc, 8.0).unwrap();
        assert!((refund - 2.0).abs() < 1e-10);
        assert!((pb.spent - 8.0).abs() < 1e-10);
        assert!((pb.returned - 2.0).abs() < 1e-10);
        assert!((pb.available() - 1000.0).abs() < 1e-10);
    }

    #[test]
    fn topology_boost_increases_cost() {
        let mut pb = PerceptionBudget::with_budget(1000.0);
        let a0 = pb.allocate(10.0, 0).unwrap();
        pb.settle(a0, 10.0).unwrap();

        let a6 = pb.allocate(10.0, 6).unwrap();
        let a6_amount = a6.amount;
        assert!(a6_amount > 10.0);
        assert!(a6.topology_boost > 1.0);
        pb.settle(a6, a6_amount).unwrap();
    }

    #[test]
    fn over_allocate_rejected() {
        let mut pb = PerceptionBudget::with_budget(100.0);
        let result = pb.allocate(pb.max_single_perception + 1.0, 0);
        assert!(matches!(result, Err(PerceptionError::OverAllocate { .. })));
    }

    #[test]
    fn insufficient_budget_rejected() {
        let mut pb = PerceptionBudget::with_budget(12.0);
        let _ = pb.allocate(3.0, 0).unwrap();
        let _ = pb.allocate(3.0, 0).unwrap();
        let _ = pb.allocate(3.0, 0).unwrap();
        let _ = pb.allocate(3.0, 0).unwrap();
        let result = pb.allocate(1.0, 0);
        assert!(matches!(result, Err(PerceptionError::InsufficientBudget { .. })));
    }

    #[test]
    fn quality_output_increases_with_topology() {
        let pb = PerceptionBudget::with_budget(1000.0);
        let q0 = pb.quality_output(10.0, 0);
        let q6 = pb.quality_output(10.0, 6);
        assert!(q6 > q0);
        assert!(q0 >= 0.0 && q0 <= 1.0);
        assert!(q6 >= 0.0 && q6 <= 1.0);
    }

    #[test]
    fn budget_conservation() {
        let mut pb = PerceptionBudget::with_budget(1000.0);
        let alloc = pb.allocate(50.0, 3).unwrap();
        let refund = pb.settle(alloc, 30.0).unwrap();

        let report = pb.report();
        let accounted = report.spent + report.returned + report.allocated;
        assert!((accounted - report.spent - report.returned).abs() < 1e-10);
        assert!((report.spent - 30.0).abs() < 1e-10);
        assert!((report.returned - refund).abs() < 1e-10);
    }

    #[test]
    fn replenish_resets_budget() {
        let mut pb = PerceptionBudget::with_budget(100.0);
        let _ = pb.allocate(10.0, 0).unwrap();
        assert!(pb.available() < 100.0);

        pb.replenish(1_000_000.0);
        assert!(pb.total_budget > 100.0);
        assert!(pb.available() > 100.0);
    }
}
