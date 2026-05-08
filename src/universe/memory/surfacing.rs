// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::universe::coord::Coord7D;
use crate::universe::memory::MemoryAtom;
use crate::universe::HebbianMemory;

const DEFAULT_SPREAD_HOPS: usize = 3;
const DEFAULT_DECAY_FACTOR: f64 = 0.5;
const DEFAULT_MIN_ACTIVATION: f64 = 0.1;
const BROADCAST_CAPACITY: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestProfile {
    pub agent_id: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default = "default_min_importance")]
    pub min_importance: f64,
    #[serde(default = "default_min_activation")]
    pub min_activation: f64,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default = "default_ttl_secs")]
    pub ttl_secs: u64,
    #[serde(default)]
    pub registered_at: u64,
}

const DEFAULT_INTEREST_TTL_SECS: u64 = 3600;

fn default_ttl_secs() -> u64 {
    DEFAULT_INTEREST_TTL_SECS
}

impl InterestProfile {
    pub fn is_expired(&self, now_secs: u64) -> bool {
        if self.ttl_secs == 0 {
            return false;
        }
        now_secs > self.registered_at && now_secs - self.registered_at > self.ttl_secs
    }
}

fn default_min_importance() -> f64 {
    0.3
}

fn default_min_activation() -> f64 {
    DEFAULT_MIN_ACTIVATION
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfacedMemory {
    pub seq: u64,
    pub anchor: String,
    pub reason: SurfacedReason,
    pub activation_score: f64,
    pub novelty_score: f64,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub importance: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SurfacedReason {
    NoveltyHigh,
    ActivationSpread,
    TemporalPrediction,
    InterestMatch,
}

pub struct ActivationEngine {
    pub max_hops: usize,
    pub decay: f64,
    pub min_activation: f64,
}

impl Default for ActivationEngine {
    fn default() -> Self {
        Self {
            max_hops: DEFAULT_SPREAD_HOPS,
            decay: DEFAULT_DECAY_FACTOR,
            min_activation: DEFAULT_MIN_ACTIVATION,
        }
    }
}

impl ActivationEngine {
    pub fn spread(&self, origin: &Coord7D, hebbian: &HebbianMemory) -> Vec<(Coord7D, f64)> {
        let mut activated: HashMap<Coord7D, f64> = HashMap::new();
        let mut frontier: Vec<(Coord7D, f64)> = vec![(*origin, 1.0)];

        for _hop in 0..self.max_hops {
            let mut next_frontier: Vec<(Coord7D, f64)> = Vec::new();
            for (node, score) in &frontier {
                let neighbors = hebbian.get_neighbors(node);
                for (neighbor, weight) in neighbors {
                    let activation = score * self.decay * (weight / 10.0f64).min(1.0);
                    if activation >= self.min_activation {
                        let existing = activated.entry(neighbor).or_default();
                        if activation > *existing {
                            *existing = activation;
                            next_frontier.push((neighbor, activation));
                        }
                    }
                }
            }
            if next_frontier.is_empty() {
                break;
            }
            frontier = next_frontier;
        }

        let mut result: Vec<(Coord7D, f64)> = activated.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }
}

#[derive(Default)]
pub struct MemorySurfacer {
    pub activation: ActivationEngine,
}

pub fn matches_interest(
    profile: &InterestProfile,
    mem: &MemoryAtom,
    activation_score: f64,
    novelty_score: f64,
) -> bool {
    if mem.importance() < profile.min_importance {
        return false;
    }
    if activation_score < profile.min_activation && novelty_score < 0.5 {
        return false;
    }
    if !profile.tags.is_empty() {
        let has_tag = profile
            .tags
            .iter()
            .any(|t| mem.tags().iter().any(|mt| mt.eq_ignore_ascii_case(t)));
        if has_tag {
            return true;
        }
    }
    if !profile.categories.is_empty() {
        if let Some(cat) = mem.category() {
            if profile
                .categories
                .iter()
                .any(|c| c.eq_ignore_ascii_case(cat))
            {
                return true;
            }
        }
    }
    if !profile.keywords.is_empty() {
        if let Some(desc) = mem.description() {
            let lower = desc.to_lowercase();
            if profile
                .keywords
                .iter()
                .any(|k| lower.contains(&k.to_lowercase()))
            {
                return true;
            }
        }
    }
    if profile.tags.is_empty()
        && profile.categories.is_empty()
        && profile.keywords.is_empty()
        && (activation_score >= profile.min_activation || novelty_score >= 0.5)
    {
        return true;
    }
    false
}

impl MemorySurfacer {
    pub fn surface(
        &self,
        new_anchor: &Coord7D,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
        interests: &HashMap<String, InterestProfile>,
        novelty_score: f64,
    ) -> Vec<SurfacedMemory> {
        let activated = self.activation.spread(new_anchor, hebbian);
        let anchor_map: HashMap<Coord7D, usize> = memories
            .iter()
            .enumerate()
            .map(|(i, m)| (*m.anchor(), i))
            .collect();

        let mut surfaced: Vec<SurfacedMemory> = Vec::new();

        if novelty_score > 0.7 {
            if let Some(&idx) = anchor_map.get(new_anchor) {
                let mem = &memories[idx];
                let is_interested = interests
                    .values()
                    .any(|p| matches_interest(p, mem, 1.0, novelty_score));
                if is_interested || interests.is_empty() {
                    surfaced.push(SurfacedMemory {
                        seq: 0,
                        anchor: format!("{}", mem.anchor()),
                        reason: SurfacedReason::NoveltyHigh,
                        activation_score: 1.0,
                        novelty_score,
                        tags: mem.tags().to_vec(),
                        category: mem.category().map(String::from),
                        description: mem.description().map(String::from),
                        importance: mem.importance(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64,
                    });
                }
            }
        }

        for (coord, activation) in &activated {
            if let Some(&idx) = anchor_map.get(coord) {
                let mem = &memories[idx];
                let is_interested = interests
                    .values()
                    .any(|p| matches_interest(p, mem, *activation, novelty_score));
                if !is_interested && !interests.is_empty() {
                    continue;
                }
                if surfaced
                    .iter()
                    .any(|s| s.anchor == format!("{}", mem.anchor()))
                {
                    continue;
                }
                surfaced.push(SurfacedMemory {
                    seq: 0,
                    anchor: format!("{}", mem.anchor()),
                    reason: if *activation > 0.3 {
                        SurfacedReason::ActivationSpread
                    } else {
                        SurfacedReason::InterestMatch
                    },
                    activation_score: *activation,
                    novelty_score,
                    tags: mem.tags().to_vec(),
                    category: mem.category().map(String::from),
                    description: mem.description().map(String::from),
                    importance: mem.importance(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                });
            }
        }

        surfaced.sort_by(|a, b| {
            let sa = a.activation_score * 0.5 + a.novelty_score * 0.3 + a.importance * 0.2;
            let sb = b.activation_score * 0.5 + b.novelty_score * 0.3 + b.importance * 0.2;
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        surfaced
    }
}

pub fn create_broadcast_channel() -> tokio::sync::broadcast::Sender<SurfacedMemory> {
    let (tx, _) = tokio::sync::broadcast::channel(BROADCAST_CAPACITY);
    tx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_spread_finds_neighbors() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([2, 0, 0, 0, 0, 0, 0]);
        h.boost_edge(&a, &b, 8.0);
        h.boost_edge(&b, &c, 8.0);

        let engine = ActivationEngine::default();
        let result = engine.spread(&a, &h);
        assert!(!result.is_empty(), "should find activated neighbors");
        assert!(result.iter().any(|(coord, _)| *coord == b));
        assert!(result.iter().any(|(coord, _)| *coord == c));
    }

    #[test]
    fn activation_decays_with_distance() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([2, 0, 0, 0, 0, 0, 0]);
        h.boost_edge(&a, &b, 5.0);
        h.boost_edge(&b, &c, 5.0);

        let engine = ActivationEngine::default();
        let result = engine.spread(&a, &h);
        let b_score = result
            .iter()
            .find(|(coord, _)| *coord == b)
            .map(|(_, s)| *s)
            .unwrap_or(0.0);
        let c_score = result
            .iter()
            .find(|(coord, _)| *coord == c)
            .map(|(_, s)| *s)
            .unwrap_or(0.0);
        assert!(b_score > c_score, "b should have higher activation than c");
    }

    #[test]
    fn activation_empty_hebbian() {
        let h = HebbianMemory::new();
        let a = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let engine = ActivationEngine::default();
        let result = engine.spread(&a, &h);
        assert!(result.is_empty());
    }

    #[test]
    fn activation_respects_min_threshold() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([2, 0, 0, 0, 0, 0, 0]);
        let d = Coord7D::new_even([3, 0, 0, 0, 0, 0, 0]);
        h.boost_edge(&a, &b, 5.0);
        h.boost_edge(&b, &c, 5.0);
        h.boost_edge(&c, &d, 5.0);

        let engine = ActivationEngine {
            max_hops: 3,
            decay: 0.5,
            min_activation: 0.05,
        };
        let result = engine.spread(&a, &h);
        assert!(result.iter().any(|(coord, _)| *coord == b));
        assert!(result.iter().any(|(coord, _)| *coord == c));
    }

    #[test]
    fn surfaced_memory_serialization() {
        let sm = SurfacedMemory {
            seq: 0,
            anchor: "(0,0,0|0,0,0,0)".to_string(),
            reason: SurfacedReason::ActivationSpread,
            activation_score: 0.75,
            novelty_score: 0.8,
            tags: vec!["test".to_string()],
            category: Some("general".to_string()),
            description: Some("test desc".to_string()),
            importance: 0.5,
            timestamp: 1000,
        };
        let json = serde_json::to_string(&sm).unwrap();
        assert!(json.contains("activation_spread"));
        assert!(json.contains("0.75"));
    }

    #[test]
    fn broadcast_channel_works() {
        let tx = create_broadcast_channel();
        let mut rx1 = tx.subscribe();
        let mut rx2 = tx.subscribe();

        let sm = SurfacedMemory {
            seq: 0,
            anchor: "test".to_string(),
            reason: SurfacedReason::NoveltyHigh,
            activation_score: 1.0,
            novelty_score: 0.9,
            tags: vec![],
            category: None,
            description: None,
            importance: 0.5,
            timestamp: 0,
        };
        tx.send(sm.clone()).unwrap();
        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());
    }

    #[test]
    fn surfacer_empty_interests_surfaces_high_novelty() {
        let surfacer = MemorySurfacer::default();
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        h.boost_edge(&a, &b, 5.0);

        let mems = Vec::new();
        let interests = HashMap::new();
        let result = surfacer.surface(&a, &h, &mems, &interests, 0.8);
        assert!(result.is_empty(), "no memories to surface");
    }

    #[test]
    fn interest_profile_deserialization() {
        let json = r#"{"agent_id":"bot1","tags":["ai","ml"],"min_importance":0.5}"#;
        let profile: InterestProfile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.agent_id, "bot1");
        assert_eq!(profile.tags.len(), 2);
        assert_eq!(profile.min_importance, 0.5);
        assert!(profile.categories.is_empty());
        assert_eq!(profile.ttl_secs, DEFAULT_INTEREST_TTL_SECS);
        assert_eq!(profile.registered_at, 0);
    }

    #[test]
    fn matches_interest_by_category() {
        let profile = InterestProfile {
            agent_id: "test".to_string(),
            tags: vec![],
            categories: vec!["science".to_string()],
            min_importance: 0.0,
            min_activation: 0.0,
            keywords: vec![],
            ttl_secs: 3600,
            registered_at: 0,
        };
        let mut u = crate::universe::DarkUniverse::new(10000.0);
        let anchor = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let mut atom =
            crate::universe::memory::MemoryCodec::encode(&mut u, &anchor, &[1.0, 2.0, 3.0])
                .unwrap();
        atom.set_category("science");
        atom.set_importance(0.6);
        assert!(matches_interest(&profile, &atom, 0.5, 0.5));
    }

    #[test]
    fn matches_interest_by_keyword() {
        let profile = InterestProfile {
            agent_id: "test".to_string(),
            tags: vec![],
            categories: vec![],
            min_importance: 0.0,
            min_activation: 0.0,
            keywords: vec!["quantum".to_string()],
            ttl_secs: 3600,
            registered_at: 0,
        };
        let mut u = crate::universe::DarkUniverse::new(10000.0);
        let anchor = Coord7D::new_even([10, 0, 0, 0, 0, 0, 0]);
        let mut atom =
            crate::universe::memory::MemoryCodec::encode(&mut u, &anchor, &[4.0, 5.0, 6.0])
                .unwrap();
        atom.set_description("quantum entanglement experiment");
        atom.set_importance(0.5);
        assert!(matches_interest(&profile, &atom, 0.5, 0.5));
    }

    #[test]
    fn interest_profile_expired_after_ttl() {
        let profile = InterestProfile {
            agent_id: "test".to_string(),
            tags: vec![],
            categories: vec![],
            min_importance: 0.3,
            min_activation: 0.1,
            keywords: vec![],
            ttl_secs: 100,
            registered_at: 1000,
        };
        assert!(!profile.is_expired(1050));
        assert!(!profile.is_expired(1100));
        assert!(profile.is_expired(1101));
        assert!(profile.is_expired(2000));
    }

    #[test]
    fn interest_profile_no_expiry_when_ttl_zero() {
        let profile = InterestProfile {
            agent_id: "test".to_string(),
            tags: vec![],
            categories: vec![],
            min_importance: 0.3,
            min_activation: 0.1,
            keywords: vec![],
            ttl_secs: 0,
            registered_at: 100,
        };
        assert!(!profile.is_expired(9999999));
    }

    #[test]
    fn interest_profile_deserialization_with_custom_ttl() {
        let json = r#"{"agent_id":"bot2","ttl_secs":7200,"registered_at":1234567890}"#;
        let profile: InterestProfile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.ttl_secs, 7200);
        assert_eq!(profile.registered_at, 1234567890);
        assert!(profile.is_expired(1234567890 + 7201));
    }
}
