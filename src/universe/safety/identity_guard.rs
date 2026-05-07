// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::memory::MemoryAtom;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const CORE_THRESHOLD: f64 = 0.9;
const MAX_SINGLE_REDUCTION: f64 = 0.05;
const VERIFICATION_ROUNDS: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionReport {
    pub protected: bool,
    pub current_importance: f64,
    pub requested_importance: f64,
    pub allowed_importance: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityGuardConfig {
    pub core_threshold: f64,
    pub max_single_reduction: f64,
    pub verification_rounds: u32,
    pub identity_tags: Vec<String>,
}

impl Default for IdentityGuardConfig {
    fn default() -> Self {
        Self {
            core_threshold: CORE_THRESHOLD,
            max_single_reduction: MAX_SINGLE_REDUCTION,
            verification_rounds: VERIFICATION_ROUNDS,
            identity_tags: vec![
                "identity".into(),
                "self_core".into(),
                "core_anchor".into(),
                "identity_first".into(),
                "name".into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationAttempt {
    pub anchor: String,
    pub proposed_importance: f64,
    pub current_importance: f64,
    pub round: u32,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct IdentityGuard {
    pub config: IdentityGuardConfig,
    pending_verifications: Vec<VerificationAttempt>,
}

impl IdentityGuard {
    pub fn new(config: IdentityGuardConfig) -> Self {
        Self {
            config,
            pending_verifications: Vec::new(),
        }
    }

    pub fn is_identity_memory(&self, mem: &MemoryAtom) -> bool {
        let is_core_importance = mem.importance() >= self.config.core_threshold;
        let has_identity_tag = mem
            .tags()
            .iter()
            .any(|t| self.config.identity_tags.contains(t));
        let has_identity_desc = mem
            .description()
            .map(|d| {
                let dl = d.to_lowercase();
                self.config
                    .identity_tags
                    .iter()
                    .any(|t| dl.contains(&t.to_lowercase()))
            })
            .unwrap_or(false);

        is_core_importance || has_identity_tag || has_identity_desc
    }

    pub fn check_importance_change(
        &mut self,
        mem: &MemoryAtom,
        requested: f64,
        now_ms: u64,
    ) -> ProtectionReport {
        let current = mem.importance();

        if !self.is_identity_memory(mem) {
            return ProtectionReport {
                protected: false,
                current_importance: current,
                requested_importance: requested,
                allowed_importance: requested,
                reason: "non-identity memory, change allowed".into(),
            };
        }

        if requested >= current {
            return ProtectionReport {
                protected: false,
                current_importance: current,
                requested_importance: requested,
                allowed_importance: requested,
                reason: "importance increase on identity memory, allowed".into(),
            };
        }

        let reduction = current - requested;

        if reduction <= self.config.max_single_reduction {
            return ProtectionReport {
                protected: true,
                current_importance: current,
                requested_importance: requested,
                allowed_importance: current - self.config.max_single_reduction,
                reason: format!(
                    "identity memory: single reduction capped at {:.3}",
                    self.config.max_single_reduction
                ),
            };
        }

        let anchor_str = format!("{}", mem.anchor());
        let matching: Vec<&VerificationAttempt> = self
            .pending_verifications
            .iter()
            .filter(|v| v.anchor == anchor_str && v.proposed_importance == requested)
            .collect();

        let rounds_completed = matching.len() as u32;

        if rounds_completed + 1 >= self.config.verification_rounds {
            self.pending_verifications
                .retain(|v| v.anchor != anchor_str || v.proposed_importance != requested);
            return ProtectionReport {
                protected: false,
                current_importance: current,
                requested_importance: requested,
                allowed_importance: requested,
                reason: format!(
                    "identity memory: verified over {} rounds, change permitted",
                    self.config.verification_rounds
                ),
            };
        }

        self.pending_verifications.push(VerificationAttempt {
            anchor: anchor_str,
            proposed_importance: requested,
            current_importance: current,
            round: rounds_completed + 1,
            timestamp_ms: now_ms,
        });

        ProtectionReport {
            protected: true,
            current_importance: current,
            requested_importance: requested,
            allowed_importance: current,
            reason: format!(
                "identity memory protected: round {}/{} — need {} more verifications to reduce from {:.3} to {:.3}",
                rounds_completed + 1,
                self.config.verification_rounds,
                self.config.verification_rounds - rounds_completed - 1,
                current,
                requested
            ),
        }
    }

    pub fn protected_anchors(&self, memories: &[MemoryAtom]) -> Vec<String> {
        memories
            .iter()
            .filter(|m| self.is_identity_memory(m))
            .map(|m| format!("{}", m.anchor()))
            .collect()
    }

    pub fn pending_verification_count(&self) -> usize {
        self.pending_verifications.len()
    }

    pub fn prune_stale_verifications(&mut self, max_age_ms: u64, now_ms: u64) -> usize {
        let before = self.pending_verifications.len();
        self.pending_verifications
            .retain(|v| now_ms.saturating_sub(v.timestamp_ms) < max_age_ms);
        before - self.pending_verifications.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProfile {
    pub total_memories: usize,
    pub identity_memories: usize,
    pub protected_anchors: Vec<String>,
    pub core_tags: HashSet<String>,
    pub avg_core_importance: f64,
}

impl IdentityGuard {
    pub fn profile(&self, memories: &[MemoryAtom]) -> IdentityProfile {
        let identity_mems: Vec<&MemoryAtom> = memories
            .iter()
            .filter(|m| self.is_identity_memory(m))
            .collect();

        let mut tags: HashSet<String> = HashSet::new();
        for m in &identity_mems {
            for t in m.tags() {
                tags.insert(t.clone());
            }
        }

        let avg = if identity_mems.is_empty() {
            0.0
        } else {
            identity_mems.iter().map(|m| m.importance()).sum::<f64>() / identity_mems.len() as f64
        };

        IdentityProfile {
            total_memories: memories.len(),
            identity_memories: identity_mems.len(),
            protected_anchors: identity_mems
                .iter()
                .map(|m| format!("{}", m.anchor()))
                .collect(),
            core_tags: tags,
            avg_core_importance: avg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::coord::Coord7D;
    use crate::universe::memory::MemoryAtom;

    fn make_mem(importance: f64, tags: Vec<&str>) -> MemoryAtom {
        let c = Coord7D::new_even([10, 10, 10, 0, 0, 0, 0]);
        let mut mem = MemoryAtom::from_parts_with_importance([c, c, c, c], 3, 50.0, 0, importance);
        for t in tags {
            mem.add_tag(t);
        }
        mem
    }

    #[test]
    fn non_identity_memory_allows_free_change() {
        let mut guard = IdentityGuard::default();
        let mem = make_mem(0.5, vec!["normal_tag"]);
        let report = guard.check_importance_change(&mem, 0.1, 1000);
        assert!(!report.protected);
        assert_eq!(report.allowed_importance, 0.1);
    }

    #[test]
    fn identity_memory_blocks_large_reduction() {
        let mut guard = IdentityGuard::default();
        let mem = make_mem(0.95, vec!["identity"]);
        let report = guard.check_importance_change(&mem, 0.3, 1000);
        assert!(report.protected);
        assert!(report.allowed_importance > 0.3);
    }

    #[test]
    fn identity_memory_allows_increase() {
        let mut guard = IdentityGuard::default();
        let mem = make_mem(0.95, vec!["identity"]);
        let report = guard.check_importance_change(&mem, 0.99, 1000);
        assert!(!report.protected);
        assert_eq!(report.allowed_importance, 0.99);
    }

    #[test]
    fn identity_memory_small_reduction_capped() {
        let mut guard = IdentityGuard::default();
        let mem = make_mem(0.95, vec!["identity"]);
        let report = guard.check_importance_change(&mem, 0.93, 1000);
        assert!(report.protected);
        assert!((report.allowed_importance - 0.90).abs() < 0.01);
    }

    #[test]
    fn verification_rounds_permit_change() {
        let mut guard = IdentityGuard::default();
        let mem = make_mem(0.95, vec!["identity"]);

        let r1 = guard.check_importance_change(&mem, 0.3, 1000);
        assert!(r1.protected);

        let r2 = guard.check_importance_change(&mem, 0.3, 2000);
        assert!(r2.protected);

        let r3 = guard.check_importance_change(&mem, 0.3, 3000);
        assert!(!r3.protected);
        assert_eq!(r3.allowed_importance, 0.3);
    }

    #[test]
    fn core_importance_detected_without_tags() {
        let guard = IdentityGuard::default();
        let mem = make_mem(0.95, vec![]);
        assert!(guard.is_identity_memory(&mem));
    }

    #[test]
    fn identity_tag_detected_with_low_importance() {
        let guard = IdentityGuard::default();
        let mem = make_mem(0.3, vec!["core_anchor"]);
        assert!(guard.is_identity_memory(&mem));
    }

    #[test]
    fn profile_reports_identity_memories() {
        let guard = IdentityGuard::default();
        let mems = vec![
            make_mem(0.95, vec!["identity"]),
            make_mem(0.5, vec!["normal"]),
            make_mem(0.99, vec!["core_anchor"]),
        ];
        let profile = guard.profile(&mems);
        assert_eq!(profile.identity_memories, 2);
        assert_eq!(profile.protected_anchors.len(), 2);
    }

    #[test]
    fn prune_stale_verifications() {
        let mut guard = IdentityGuard::default();
        let mem = make_mem(0.95, vec!["identity"]);
        guard.check_importance_change(&mem, 0.3, 1000);
        guard.check_importance_change(&mem, 0.3, 2000);
        assert_eq!(guard.pending_verification_count(), 2);

        let pruned = guard.prune_stale_verifications(1500, 5000);
        assert_eq!(pruned, 2);
        assert_eq!(guard.pending_verification_count(), 0);
    }
}
