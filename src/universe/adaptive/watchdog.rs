// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::autoscale::AutoScaler;
use crate::universe::backup::{BackupScheduler, BackupTrigger};
use crate::universe::crystal::CrystalEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use crate::universe::observer::{HealthLevel, HealthReport, UniverseObserver};
use crate::universe::regulation::RegulationEngine;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchdogLevel {
    Normal,
    Warning,
    Critical,
    Emergency,
}

impl WatchdogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            WatchdogLevel::Normal => "NORMAL",
            WatchdogLevel::Warning => "WARNING",
            WatchdogLevel::Critical => "CRITICAL",
            WatchdogLevel::Emergency => "EMERGENCY",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WatermarkThresholds {
    pub utilization_warning: f64,
    pub utilization_critical: f64,
    pub utilization_emergency: f64,
    pub memory_corruption_max: usize,
    pub max_consecutive_failures: u32,
    pub conservation_violation_limit: u32,
    pub energy_expansion_cap_ratio: f64,
    pub auto_backup_interval_ops: u64,
    pub node_count_warning: usize,
    pub node_count_critical: usize,
}

impl Default for WatermarkThresholds {
    fn default() -> Self {
        Self {
            utilization_warning: 0.75,
            utilization_critical: 0.90,
            utilization_emergency: 0.98,
            memory_corruption_max: 0,
            max_consecutive_failures: 10,
            conservation_violation_limit: 3,
            energy_expansion_cap_ratio: 10.0,
            auto_backup_interval_ops: 500,
            node_count_warning: 50_000,
            node_count_critical: 200_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WatchdogAction {
    pub action: String,
    pub detail: String,
    pub level: WatchdogLevel,
}

impl std::fmt::Display for WatchdogAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.level.as_str(),
            self.action,
            self.detail
        )
    }
}

#[derive(Debug, Clone)]
pub struct WatchdogReport {
    pub level: WatchdogLevel,
    pub health: HealthLevel,
    pub utilization: f64,
    pub node_count: usize,
    pub memory_count: usize,
    pub conservation_ok: bool,
    pub consecutive_conservation_failures: u32,
    pub total_checkups: u64,
    pub actions: Vec<WatchdogAction>,
    pub backup_count: usize,
    pub initial_energy: f64,
    pub current_energy: f64,
    pub elapsed_ms: f64,
}

impl std::fmt::Display for WatchdogReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Watchdog[{} health:{} util:{:.1}% nodes:{} mems:{} cons:{}:{} backup:{} E:{:.0}→{:.0}]",
            self.level.as_str(),
            self.health.as_str(),
            self.utilization * 100.0,
            self.node_count,
            self.memory_count,
            if self.conservation_ok { "OK" } else { "FAIL" },
            self.consecutive_conservation_failures,
            self.backup_count,
            self.initial_energy,
            self.current_energy,
        )
    }
}

pub struct Watchdog {
    thresholds: WatermarkThresholds,
    regulation: RegulationEngine,
    scaler: AutoScaler,
    consecutive_failures: u32,
    conservation_failures: u32,
    total_checkups: u64,
    initial_energy: f64,
    last_backup_time: Option<Instant>,
    created_at: Instant,
}

impl Watchdog {
    pub fn new(initial_energy: f64, thresholds: WatermarkThresholds) -> Self {
        Self {
            thresholds,
            regulation: RegulationEngine::new(),
            scaler: AutoScaler::new(),
            consecutive_failures: 0,
            conservation_failures: 0,
            total_checkups: 0,
            initial_energy,
            last_backup_time: None,
            created_at: Instant::now(),
        }
    }

    pub fn with_defaults(initial_energy: f64) -> Self {
        Self::new(initial_energy, WatermarkThresholds::default())
    }

    pub fn total_checkups(&self) -> u64 {
        self.total_checkups
    }

    pub fn uptime_ms(&self) -> f64 {
        self.created_at.elapsed().as_secs_f64() * 1000.0
    }

    pub fn checkup(
        &mut self,
        universe: &mut DarkUniverse,
        hebbian: &mut HebbianMemory,
        crystal: &mut CrystalEngine,
        memories: &[MemoryAtom],
    ) -> WatchdogReport {
        let t = Instant::now();
        self.total_checkups += 1;

        let health_report = UniverseObserver::inspect(universe, hebbian, memories);
        let stats = universe.stats();
        let conservation_ok = universe.verify_conservation();
        let mut actions = Vec::new();

        if !conservation_ok {
            self.conservation_failures += 1;
            actions.push(WatchdogAction {
                action: "conservation_alert".to_string(),
                detail: format!(
                    "conservation failure #{} (diff may be in tolerance range)",
                    self.conservation_failures
                ),
                level: WatchdogLevel::Critical,
            });
        } else {
            self.conservation_failures = 0;
        }

        let level = self.compute_level(&health_report, &stats, conservation_ok);

        if level >= WatchdogLevel::Critical {
            self.handle_critical(universe, hebbian, crystal, memories, &mut actions);
        }

        if level >= WatchdogLevel::Warning {
            self.handle_warning(universe, hebbian, crystal, memories, &stats, &mut actions);
        }

        if level >= WatchdogLevel::Emergency {
            let injection = stats.total_energy * 0.5;
            let cap = self.initial_energy * self.thresholds.energy_expansion_cap_ratio;
            if stats.total_energy + injection <= cap {
                actions.push(WatchdogAction {
                    action: "emergency_energy_injection".to_string(),
                    detail: format!(
                        "utilization {:.1}% — injecting {:.0} energy (cap {:.0})",
                        stats.utilization * 100.0,
                        injection,
                        cap
                    ),
                    level: WatchdogLevel::Emergency,
                });
                if !universe.expand_energy_pool(injection) {
                    tracing::error!(
                        "emergency_energy_injection: failed to expand pool by {:.0}",
                        injection
                    );
                }
            } else {
                actions.push(WatchdogAction {
                    action: "emergency_injection_skipped".to_string(),
                    detail: format!(
                        "injection {:.0} would exceed cap {:.0} (current {:.0})",
                        injection, cap, stats.total_energy
                    ),
                    level: WatchdogLevel::Emergency,
                });
            }
        }

        let elapsed_ms = t.elapsed().as_secs_f64() * 1000.0;

        WatchdogReport {
            level,
            health: health_report.health_level(),
            utilization: stats.utilization,
            node_count: stats.active_nodes,
            memory_count: memories.len(),
            conservation_ok,
            consecutive_conservation_failures: self.conservation_failures,
            total_checkups: self.total_checkups,
            actions,
            backup_count: 0,
            initial_energy: self.initial_energy,
            current_energy: stats.total_energy,
            elapsed_ms,
        }
    }

    pub fn checkup_with_backup(
        &mut self,
        universe: &mut DarkUniverse,
        hebbian: &mut HebbianMemory,
        crystal: &mut CrystalEngine,
        memories: &[MemoryAtom],
        backup: &mut BackupScheduler,
    ) -> WatchdogReport {
        let mut report = self.checkup(universe, hebbian, crystal, memories);

        let should_backup = self.should_auto_backup(backup);
        if should_backup {
            match backup.create_backup(BackupTrigger::Timer, universe, hebbian, memories, crystal) {
                Ok(br) => {
                    self.last_backup_time = Some(Instant::now());
                    report.backup_count = backup.backup_count();
                    report.actions.push(WatchdogAction {
                        action: "auto_backup".to_string(),
                        detail: format!("{}", br.metadata),
                        level: WatchdogLevel::Normal,
                    });
                }
                Err(_) => {
                    report.actions.push(WatchdogAction {
                        action: "backup_failed".to_string(),
                        detail: "automatic backup failed".to_string(),
                        level: WatchdogLevel::Warning,
                    });
                }
            }
        }

        if !report.conservation_ok
            && backup
                .create_backup(
                    BackupTrigger::ConservationCheckpoint,
                    universe,
                    hebbian,
                    memories,
                    crystal,
                )
                .is_ok()
        {
            report.actions.push(WatchdogAction {
                action: "conservation_checkpoint".to_string(),
                detail: "pre-recovery backup created".to_string(),
                level: WatchdogLevel::Critical,
            });
        }

        report
    }

    fn should_auto_backup(&self, backup: &BackupScheduler) -> bool {
        let ops_since = backup.operation_count();
        let interval = self.thresholds.auto_backup_interval_ops;
        if interval == 0 {
            return false;
        }

        if let Some(last) = self.last_backup_time {
            let since_last = backup.operation_count();
            since_last >= interval || last.elapsed().as_secs() >= 60
        } else {
            ops_since >= interval
        }
    }

    fn compute_level(
        &self,
        health: &HealthReport,
        stats: &crate::universe::node::UniverseStats,
        conservation_ok: bool,
    ) -> WatchdogLevel {
        if !conservation_ok
            || self.conservation_failures >= self.thresholds.conservation_violation_limit
        {
            return WatchdogLevel::Emergency;
        }

        if stats.utilization >= self.thresholds.utilization_emergency {
            return WatchdogLevel::Emergency;
        }

        if stats.utilization >= self.thresholds.utilization_critical
            || stats.active_nodes >= self.thresholds.node_count_critical
        {
            return WatchdogLevel::Critical;
        }

        if stats.utilization >= self.thresholds.utilization_warning
            || stats.active_nodes >= self.thresholds.node_count_warning
            || health.health_level() == HealthLevel::Warning
        {
            return WatchdogLevel::Warning;
        }

        WatchdogLevel::Normal
    }

    fn handle_critical(
        &self,
        universe: &mut DarkUniverse,
        hebbian: &mut HebbianMemory,
        crystal: &mut CrystalEngine,
        memories: &[MemoryAtom],
        actions: &mut Vec<WatchdogAction>,
    ) {
        let reg_report = self
            .regulation
            .regulate(universe, hebbian, crystal, memories);
        for action in &reg_report.actions {
            actions.push(WatchdogAction {
                action: format!("regulation_{}", action.action),
                detail: action.detail.clone(),
                level: WatchdogLevel::Critical,
            });
        }
    }

    fn handle_warning(
        &self,
        universe: &mut DarkUniverse,
        hebbian: &mut HebbianMemory,
        _crystal: &mut CrystalEngine,
        memories: &[MemoryAtom],
        stats: &crate::universe::node::UniverseStats,
        actions: &mut Vec<WatchdogAction>,
    ) {
        let scale_report = self.scaler.auto_scale(universe, hebbian, memories);
        if scale_report.energy_expanded_by > 0.0 || scale_report.nodes_added > 0 {
            actions.push(WatchdogAction {
                action: "auto_scale".to_string(),
                detail: format!(
                    "+{:.0}E +{}nodes (util was {:.1}%)",
                    scale_report.energy_expanded_by,
                    scale_report.nodes_added,
                    stats.utilization * 100.0
                ),
                level: WatchdogLevel::Warning,
            });
        }
    }

    pub fn protect_encode(
        &mut self,
        universe: &mut DarkUniverse,
        hebbian: &mut HebbianMemory,
        crystal: &mut CrystalEngine,
        memories: &[MemoryAtom],
        backup: &mut BackupScheduler,
    ) -> WatchdogLevel {
        let report = self.checkup_with_backup(universe, hebbian, crystal, memories, backup);
        report.level
    }

    pub fn validate_recovery(
        &mut self,
        universe: &DarkUniverse,
        memories: &[MemoryAtom],
    ) -> Result<(), String> {
        if !universe.verify_conservation() {
            self.consecutive_failures += 1;
            if self.consecutive_failures >= self.thresholds.max_consecutive_failures {
                return Err(format!(
                    "conservation validation failed {} consecutive times",
                    self.consecutive_failures
                ));
            }
        } else {
            self.consecutive_failures = 0;
        }

        let mut corrupt = 0;
        for mem in memories {
            if !mem.exists_in(universe) {
                corrupt += 1;
            }
        }
        if corrupt > self.thresholds.memory_corruption_max {
            return Err(format!("{} corrupted memories detected", corrupt));
        }

        Ok(())
    }
}

impl WatchdogLevel {
    fn order(&self) -> u8 {
        match self {
            WatchdogLevel::Normal => 0,
            WatchdogLevel::Warning => 1,
            WatchdogLevel::Critical => 2,
            WatchdogLevel::Emergency => 3,
        }
    }
}

impl PartialOrd for WatchdogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WatchdogLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order().cmp(&other.order())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::coord::Coord7D;

    #[test]
    fn watchdog_normal_state() {
        let mut wd = Watchdog::with_defaults(1000.0);
        let mut u = DarkUniverse::new(1000.0);
        let mut h = HebbianMemory::new();
        let mut c = CrystalEngine::new();
        let mems: Vec<MemoryAtom> = vec![];

        let report = wd.checkup(&mut u, &mut h, &mut c, &mems);
        assert_eq!(report.level, WatchdogLevel::Normal);
        assert!(report.conservation_ok);
    }

    #[test]
    fn watchdog_high_utilization_warning() {
        let thresholds = WatermarkThresholds {
            utilization_warning: 0.5,
            ..Default::default()
        };
        let mut wd = Watchdog::new(1000.0, thresholds);

        let mut u = DarkUniverse::new(100.0);
        for i in 0..80 {
            let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            u.materialize_uniform(c, 1.0).ok();
        }
        let mut h = HebbianMemory::new();
        let mut c = CrystalEngine::new();
        let mems: Vec<MemoryAtom> = vec![];

        let report = wd.checkup(&mut u, &mut h, &mut c, &mems);
        assert!(report.level >= WatchdogLevel::Warning);
    }

    #[test]
    fn watchdog_with_backup() {
        let mut wd = Watchdog::with_defaults(5000.0);
        let mut backup = BackupScheduler::with_defaults();
        let mut u = DarkUniverse::new(5000.0);
        let mut h = HebbianMemory::new();
        let mut c = CrystalEngine::new();

        let data = vec![1.0, 2.0];
        let anchor = Coord7D::new_even([5, 0, 0, 0, 0, 0, 0]);
        let mem = crate::universe::memory::MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
        let mems = vec![mem];

        for _ in 0..500 {
            backup.record_operation();
        }

        let report = wd.checkup_with_backup(&mut u, &mut h, &mut c, &mems, &mut backup);
        assert!(report.backup_count > 0 || backup.backup_count() > 0);
    }

    #[test]
    fn watchdog_validate_recovery_ok() {
        let mut wd = Watchdog::with_defaults(1000.0);
        let u = DarkUniverse::new(1000.0);
        let mems: Vec<MemoryAtom> = vec![];
        assert!(wd.validate_recovery(&u, &mems).is_ok());
    }

    #[test]
    fn watchdog_emergency_energy_injection() {
        let thresholds = WatermarkThresholds {
            utilization_emergency: 0.80,
            ..Default::default()
        };
        let mut wd = Watchdog::new(100.0, thresholds);

        let mut u = DarkUniverse::new(100.0);
        for i in 0..90 {
            let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            u.materialize_uniform(c, 1.0).ok();
        }
        let before_energy = u.total_energy();
        let mut h = HebbianMemory::new();
        let mut c = CrystalEngine::new();
        let mems: Vec<MemoryAtom> = vec![];

        let report = wd.checkup(&mut u, &mut h, &mut c, &mems);
        assert!(report.level >= WatchdogLevel::Emergency);
        assert!(u.total_energy() > before_energy);
    }

    #[test]
    fn watchdog_long_run_checkups() {
        let mut wd = Watchdog::with_defaults(100_000.0);
        let mut u = DarkUniverse::new(100_000.0);
        let mut h = HebbianMemory::new();
        let mut c = CrystalEngine::new();
        let mut mems: Vec<MemoryAtom> = vec![];
        let scaler = AutoScaler::new();

        for i in 0..200 {
            let data = vec![i as f64 * 0.1];
            let anchor = Coord7D::new_even([i * 10, 0, 0, 0, 0, 0, 0]);
            match crate::universe::memory::MemoryCodec::encode(&mut u, &anchor, &data) {
                Ok(mem) => mems.push(mem),
                Err(_) => {
                    scaler.scale_near_anchor(&mut u, &anchor, &data).ok();
                }
            }
        }

        for _ in 0..10 {
            let _report = wd.checkup(&mut u, &mut h, &mut c, &mems);
            assert!(u.verify_conservation());
        }
        assert_eq!(wd.total_checkups(), 10);
    }
}
