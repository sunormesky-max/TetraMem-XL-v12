use crate::universe::crystal::CrystalEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use crate::universe::persist::{PersistEngine, PersistError, UniverseSnapshot};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackupTrigger {
    Manual,
    Timer,
    PreOperation,
    ConservationCheckpoint,
}

#[derive(Debug, Clone)]
pub struct BackupMetadata {
    pub id: u64,
    pub timestamp_ms: u64,
    pub trigger: BackupTrigger,
    pub node_count: usize,
    pub memory_count: usize,
    pub hebbian_edges: usize,
    pub crystal_channels: usize,
    pub total_energy: f64,
    pub conservation_ok: bool,
    pub bytes: usize,
    pub generation: u32,
}

impl std::fmt::Display for BackupMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let trigger = match self.trigger {
            BackupTrigger::Manual => "MANUAL",
            BackupTrigger::Timer => "TIMER",
            BackupTrigger::PreOperation => "PRE-OP",
            BackupTrigger::ConservationCheckpoint => "CONSERV",
        };
        write!(
            f,
            "Backup#{} gen{} [{}] nodes:{} mems:{} edges:{} crystals:{} E:{:.0} cons:{} {}bytes",
            self.id, self.generation, trigger,
            self.node_count, self.memory_count,
            self.hebbian_edges, self.crystal_channels,
            self.total_energy,
            if self.conservation_ok { "OK" } else { "FAIL" },
            self.bytes
        )
    }
}

struct BackupEntry {
    metadata: BackupMetadata,
    snapshot: UniverseSnapshot,
}

pub struct BackupConfig {
    pub max_generations: u32,
    pub max_total_backups: usize,
    pub conservation_checkpoint_interval: u64,
    pub rotate_on_high_memory_mb: f64,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            max_generations: 5,
            max_total_backups: 20,
            conservation_checkpoint_interval: 100,
            rotate_on_high_memory_mb: 100.0,
        }
    }
}

pub struct BackupReport {
    pub metadata: BackupMetadata,
    pub elapsed_ms: f64,
    pub rotated: usize,
}

impl std::fmt::Display for BackupReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:.1}ms rotated:{}",
            self.metadata, self.elapsed_ms, self.rotated
        )
    }
}

pub struct BackupScheduler {
    config: BackupConfig,
    backups: Vec<BackupEntry>,
    next_id: u64,
    operation_count: u64,
    last_backup_op_count: u64,
    current_generation: u32,
}

impl BackupScheduler {
    pub fn new(config: BackupConfig) -> Self {
        Self {
            config,
            backups: Vec::new(),
            next_id: 1,
            operation_count: 0,
            last_backup_op_count: 0,
            current_generation: 0,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(BackupConfig::default())
    }

    pub fn operation_count(&self) -> u64 {
        self.operation_count
    }

    pub fn backup_count(&self) -> usize {
        self.backups.len()
    }

    pub fn current_generation(&self) -> u32 {
        self.current_generation
    }

    pub fn latest_metadata(&self) -> Option<&BackupMetadata> {
        self.backups.last().map(|e| &e.metadata)
    }

    pub fn record_operation(&mut self) {
        self.operation_count += 1;
    }

    pub fn should_checkpoint(&self) -> bool {
        let interval = self.config.conservation_checkpoint_interval;
        if interval == 0 {
            return false;
        }
        self.operation_count - self.last_backup_op_count >= interval
    }

    pub fn should_timer_backup(&self, last_backup: Option<Instant>, interval_ms: u64) -> bool {
        if let Some(last) = last_backup {
            last.elapsed().as_millis() as u64 >= interval_ms
        } else {
            true
        }
    }

    pub fn create_backup(
        &mut self,
        trigger: BackupTrigger,
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
        crystal: &CrystalEngine,
    ) -> Result<BackupReport, PersistError> {
        let t = Instant::now();

        if trigger == BackupTrigger::Manual || trigger == BackupTrigger::ConservationCheckpoint {
            self.current_generation += 1;
        }

        let (snapshot, _) = PersistEngine::serialize(universe, hebbian, memories, crystal)?;
        let json_bytes = serde_json::to_string(&snapshot)
            .map(|s| s.len())
            .unwrap_or(0);

        let stats = universe.stats();
        let metadata = BackupMetadata {
            id: self.next_id,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            trigger,
            node_count: stats.active_nodes,
            memory_count: memories.len(),
            hebbian_edges: hebbian.edge_count(),
            crystal_channels: crystal.channel_count(),
            total_energy: stats.total_energy,
            conservation_ok: universe.verify_conservation(),
            bytes: json_bytes,
            generation: self.current_generation,
        };

        self.next_id += 1;
        self.last_backup_op_count = self.operation_count;

        let entry = BackupEntry {
            metadata: metadata.clone(),
            snapshot,
        };
        self.backups.push(entry);

        let rotated = self.rotate();

        let elapsed_ms = t.elapsed().as_secs_f64() * 1000.0;
        Ok(BackupReport {
            metadata,
            elapsed_ms,
            rotated,
        })
    }

    fn rotate(&mut self) -> usize {
        let before = self.backups.len();

        while self.backups.len() > self.config.max_total_backups {
            self.backups.remove(0);
        }

        let mut by_gen: std::collections::HashMap<u32, Vec<usize>> = std::collections::HashMap::new();
        for (i, e) in self.backups.iter().enumerate() {
            by_gen.entry(e.metadata.generation).or_default().push(i);
        }

        let oldest_allowed = if self.current_generation > self.config.max_generations {
            self.current_generation - self.config.max_generations
        } else {
            0
        };
        self.backups.retain(|e| e.metadata.generation >= oldest_allowed);

        for gen in by_gen.keys() {
            let gen_ids: Vec<u64> = self.backups.iter()
                .filter(|e| e.metadata.generation == *gen)
                .skip(self.config.max_generations as usize)
                .map(|e| e.metadata.id)
                .collect();
            for id in gen_ids {
                self.backups.retain(|e| e.metadata.id != id);
            }
        }

        before - self.backups.len()
    }

    pub fn restore_latest(
        &self,
    ) -> Option<(DarkUniverse, HebbianMemory, Vec<MemoryAtom>, CrystalEngine)> {
        self.backups.last().and_then(|entry| {
            PersistEngine::deserialize(&entry.snapshot).ok()
        })
    }

    pub fn restore_generation(
        &self,
        generation: u32,
    ) -> Option<(DarkUniverse, HebbianMemory, Vec<MemoryAtom>, CrystalEngine)> {
        let entry = self.backups.iter()
            .filter(|e| e.metadata.generation == generation)
            .last()?;
        PersistEngine::deserialize(&entry.snapshot).ok()
    }

    pub fn restore_by_id(
        &self,
        id: u64,
    ) -> Option<(DarkUniverse, HebbianMemory, Vec<MemoryAtom>, CrystalEngine)> {
        let entry = self.backups.iter().find(|e| e.metadata.id == id)?;
        PersistEngine::deserialize(&entry.snapshot).ok()
    }

    pub fn list_backups(&self) -> Vec<BackupMetadata> {
        self.backups.iter().map(|e| e.metadata.clone()).collect()
    }

    pub fn prune_before_generation(&mut self, gen: u32) -> usize {
        let before = self.backups.len();
        self.backups.retain(|e| e.metadata.generation >= gen);
        before - self.backups.len()
    }

    pub fn total_backup_bytes(&self) -> usize {
        self.backups.iter().map(|e| e.metadata.bytes).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::coord::Coord7D;

    #[test]
    fn backup_scheduler_basic() {
        let mut sched = BackupScheduler::with_defaults();
        let u = DarkUniverse::new(1000.0);
        let h = HebbianMemory::new();
        let c = CrystalEngine::new();
        let mems: Vec<MemoryAtom> = vec![];

        let report = sched.create_backup(BackupTrigger::Manual, &u, &h, &mems, &c).unwrap();
        assert_eq!(sched.backup_count(), 1);
        assert_eq!(report.metadata.generation, 1);
        assert!(report.metadata.conservation_ok);
    }

    #[test]
    fn backup_restore_roundtrip() {
        let mut sched = BackupScheduler::with_defaults();
        let mut u = DarkUniverse::new(5000.0);
        let mut h = HebbianMemory::new();
        let mut mems = Vec::new();

        let data = vec![1.0, 2.0, 3.0];
        let anchor = Coord7D::new_even([10, 0, 0, 0, 0, 0, 0]);
        let mem = crate::universe::memory::MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
        mems.push(mem);
        h.record_path(&[anchor, Coord7D::new_even([11, 0, 0, 0, 0, 0, 0])], 1.5);

        sched.create_backup(BackupTrigger::Manual, &u, &h, &mems, &CrystalEngine::new()).unwrap();

        let (u2, h2, m2, _c2) = sched.restore_latest().unwrap();
        assert_eq!(u2.active_node_count(), u.active_node_count());
        assert_eq!(h2.edge_count(), h.edge_count());
        assert_eq!(m2.len(), 1);

        let decoded = crate::universe::memory::MemoryCodec::decode(&u2, &m2[0]).unwrap();
        for (a, b) in data.iter().zip(decoded.iter()) {
            assert!((a - b).abs() < 1e-10);
        }
    }

    #[test]
    fn backup_rotation() {
        let mut config = BackupConfig::default();
        config.max_total_backups = 3;
        config.max_generations = 2;
        let mut sched = BackupScheduler::new(config);
        let u = DarkUniverse::new(1000.0);
        let h = HebbianMemory::new();
        let c = CrystalEngine::new();
        let mems: Vec<MemoryAtom> = vec![];

        for _ in 0..5 {
            sched.create_backup(BackupTrigger::Manual, &u, &h, &mems, &c).unwrap();
        }
        assert!(sched.backup_count() <= 3);
    }

    #[test]
    fn checkpoint_interval() {
        let mut sched = BackupScheduler::with_defaults();
        assert!(!sched.should_checkpoint());

        for _ in 0..100 {
            sched.record_operation();
        }
        assert!(sched.should_checkpoint());
    }

    #[test]
    fn prune_old_generations() {
        let mut sched = BackupScheduler::with_defaults();
        let u = DarkUniverse::new(1000.0);
        let h = HebbianMemory::new();
        let c = CrystalEngine::new();
        let mems: Vec<MemoryAtom> = vec![];

        sched.create_backup(BackupTrigger::Manual, &u, &h, &mems, &c).unwrap();
        sched.create_backup(BackupTrigger::Manual, &u, &h, &mems, &c).unwrap();
        assert_eq!(sched.backup_count(), 2);

        let removed = sched.prune_before_generation(2);
        assert_eq!(removed, 1);
        assert_eq!(sched.backup_count(), 1);
    }
}
