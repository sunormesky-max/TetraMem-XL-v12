// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use serde::Serialize;

use crate::universe::memory::semantic::SemanticEmbedding;
use crate::universe::memory::MemoryAtom;

const CONTRADICTION_DISTANCE_THRESHOLD: f64 = 0.3;
const MERGE_SIMILARITY_THRESHOLD: f64 = 0.85;

#[derive(Debug, Clone, Serialize)]
pub struct ContradictionPair {
    pub anchor_a: String,
    pub anchor_b: String,
    pub distance: f64,
    pub description_a: Option<String>,
    pub description_b: Option<String>,
    pub tags_a: Vec<String>,
    pub tags_b: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MergeCandidate {
    pub anchor_a: String,
    pub anchor_b: String,
    pub similarity: f64,
    pub description_a: Option<String>,
    pub description_b: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContradictionReport {
    pub contradictions: Vec<ContradictionPair>,
    pub merge_candidates: Vec<MergeCandidate>,
}

pub struct ContradictionDetector {
    pub contradiction_threshold: f64,
    pub merge_threshold: f64,
}

impl Default for ContradictionDetector {
    fn default() -> Self {
        Self {
            contradiction_threshold: CONTRADICTION_DISTANCE_THRESHOLD,
            merge_threshold: MERGE_SIMILARITY_THRESHOLD,
        }
    }
}

impl ContradictionDetector {
    pub fn detect(&self, memories: &[MemoryAtom]) -> ContradictionReport {
        let mut contradictions = Vec::new();
        let mut merge_candidates = Vec::new();

        let embeddings: Vec<(usize, SemanticEmbedding)> = memories
            .iter()
            .enumerate()
            .filter_map(|(i, m)| {
                if m.data_dim() > 0 {
                    let emb = SemanticEmbedding::from_annotation(m);
                    Some((i, emb))
                } else {
                    None
                }
            })
            .collect();

        for i in 0..embeddings.len() {
            for j in (i + 1)..embeddings.len() {
                let (idx_a, ref emb_a) = embeddings[i];
                let (idx_b, ref emb_b) = embeddings[j];
                let ma = &memories[idx_a];
                let mb = &memories[idx_b];

                let similarity = emb_a.cosine_similarity(emb_b);
                let distance = 1.0 - similarity;

                let desc_a = ma.description().map(String::from);
                let desc_b = mb.description().map(String::from);
                let anchor_a = format!("{}", ma.anchor());
                let anchor_b = format!("{}", mb.anchor());

                if distance < self.contradiction_threshold {
                    let has_conflict = descriptions_conflict(ma.description(), mb.description())
                        || tags_conflict(ma.tags(), mb.tags());

                    if has_conflict {
                        contradictions.push(ContradictionPair {
                            anchor_a: anchor_a.clone(),
                            anchor_b: anchor_b.clone(),
                            distance,
                            description_a: desc_a.clone(),
                            description_b: desc_b.clone(),
                            tags_a: ma.tags().to_vec(),
                            tags_b: mb.tags().to_vec(),
                        });
                    }
                }

                if similarity > self.merge_threshold {
                    merge_candidates.push(MergeCandidate {
                        anchor_a,
                        anchor_b,
                        similarity,
                        description_a: desc_a,
                        description_b: desc_b,
                    });
                }
            }
        }

        contradictions.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        merge_candidates.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        ContradictionReport {
            contradictions,
            merge_candidates,
        }
    }
}

pub fn descriptions_conflict(a: Option<&str>, b: Option<&str>) -> bool {
    let (Some(da), Some(db)) = (a, b) else {
        return false;
    };
    let da_lower = da.to_lowercase();
    let db_lower = db.to_lowercase();

    if da_lower.contains("not ") || da_lower.contains(" not ") {
        let cleaned = da_lower.replace(" not ", " ").replace("not ", "");
        if db_lower.contains(cleaned.trim()) {
            return true;
        }
    }
    if db_lower.contains("not ") || db_lower.contains(" not ") {
        let cleaned = db_lower.replace(" not ", " ").replace("not ", "");
        if da_lower.contains(cleaned.trim()) {
            return true;
        }
    }

    let opposite_pairs = [
        ("impossible", "possible"),
        ("false", "true"),
        ("wrong", "right"),
        ("bad", "good"),
        ("never", "always"),
        ("cannot", "can"),
        ("no ", "yes "),
    ];
    for (neg, pos) in &opposite_pairs {
        if da_lower.contains(neg) && db_lower.contains(pos)
            || db_lower.contains(neg) && da_lower.contains(pos)
        {
            return true;
        }
    }
    false
}

pub fn tags_conflict(a: &[String], b: &[String]) -> bool {
    let opposite_pairs = [
        ("positive", "negative"),
        ("true", "false"),
        ("good", "bad"),
        ("correct", "incorrect"),
        ("valid", "invalid"),
        ("confirmed", "denied"),
    ];
    for (pos, neg) in &opposite_pairs {
        let a_has_pos = a.iter().any(|t| t.eq_ignore_ascii_case(pos));
        let b_has_neg = b.iter().any(|t| t.eq_ignore_ascii_case(neg));
        let a_has_neg = a.iter().any(|t| t.eq_ignore_ascii_case(neg));
        let b_has_pos = b.iter().any(|t| t.eq_ignore_ascii_case(pos));
        if (a_has_pos && b_has_neg) || (a_has_neg && b_has_pos) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptions_conflict_not() {
        assert!(descriptions_conflict(
            Some("this is not true"),
            Some("this is true")
        ));
    }

    #[test]
    fn descriptions_no_conflict() {
        assert!(!descriptions_conflict(
            Some("the sky is blue"),
            Some("water is wet")
        ));
    }

    #[test]
    fn tags_conflict_positive_negative() {
        assert!(tags_conflict(
            &[String::from("positive")],
            &[String::from("negative")]
        ));
    }

    #[test]
    fn tags_no_conflict() {
        assert!(!tags_conflict(
            &[String::from("science")],
            &[String::from("physics")]
        ));
    }
}
