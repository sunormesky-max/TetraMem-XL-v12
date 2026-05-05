// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::coord::Coord7D;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;

const HAAR_LEVELS: usize = 4;
const NOVELTY_HIGH_THRESHOLD: f64 = 0.7;
const NOVELTY_LOW_THRESHOLD: f64 = 0.35;

#[derive(Debug, Clone)]
pub struct NoveltyReport {
    pub score: f64,
    pub level: NoveltyLevel,
    pub nearest_distance: f64,
    pub nearest_anchor: Option<Coord7D>,
    pub suggested_importance: f64,
    pub wavelet_energy: f64,
    pub detail_energy: f64,
    pub anomaly_score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoveltyLevel {
    HighlyNovel,
    ModeratelyNovel,
    Familiar,
    Redundant,
}

impl std::fmt::Display for NoveltyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoveltyLevel::HighlyNovel => write!(f, "highly_novel"),
            NoveltyLevel::ModeratelyNovel => write!(f, "moderately_novel"),
            NoveltyLevel::Familiar => write!(f, "familiar"),
            NoveltyLevel::Redundant => write!(f, "redundant"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NoveltyConfig {
    pub high_threshold: f64,
    pub low_threshold: f64,
    pub redundancy_threshold: f64,
    pub importance_boost_novel: f64,
    pub importance_penalty_redundant: f64,
    pub wavelet_weight: f64,
    pub hebbian_novelty_weight: f64,
}

impl Default for NoveltyConfig {
    fn default() -> Self {
        Self {
            high_threshold: NOVELTY_HIGH_THRESHOLD,
            low_threshold: NOVELTY_LOW_THRESHOLD,
            redundancy_threshold: 0.15,
            importance_boost_novel: 0.3,
            importance_penalty_redundant: 0.3,
            wavelet_weight: 0.3,
            hebbian_novelty_weight: 0.2,
        }
    }
}

#[derive(Clone)]
pub struct NoveltyDetector {
    config: NoveltyConfig,
}

impl Default for NoveltyDetector {
    fn default() -> Self {
        Self::new(NoveltyConfig::default())
    }
}

impl NoveltyDetector {
    pub fn new(config: NoveltyConfig) -> Self {
        Self { config }
    }

    pub fn assess(
        &self,
        data: &[f64],
        knn_distances: &[(f64, usize)],
        anchor: &Coord7D,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
    ) -> NoveltyReport {
        let raw_distance = knn_distances.first().map(|(d, _)| *d).unwrap_or(2.0);
        let nearest_distance = (raw_distance / 2.0).min(1.0);
        let nearest_idx = knn_distances.first().map(|(_, i)| *i);
        let nearest_anchor = nearest_idx.and_then(|i| memories.get(i).map(|m| *m.anchor()));

        let semantic_novelty = nearest_distance;

        let (wavelet_energy, detail_energy) = wavelet_decompose(data);
        let _data_length = data.len().max(1) as f64;
        let anomaly_score = if wavelet_energy > 0.0 {
            detail_energy / wavelet_energy
        } else {
            0.0
        };

        let neighbors = hebbian.get_neighbors(anchor);
        let hebbian_connectivity = neighbors.len() as f64;
        let hebbian_novelty = 1.0 / (1.0 + hebbian_connectivity * 0.1);

        let score = (semantic_novelty
            * (1.0 - self.config.wavelet_weight - self.config.hebbian_novelty_weight))
            + (anomaly_score.min(1.0) * self.config.wavelet_weight)
            + (hebbian_novelty * self.config.hebbian_novelty_weight);

        let level = if score > self.config.high_threshold {
            NoveltyLevel::HighlyNovel
        } else if score > self.config.low_threshold {
            NoveltyLevel::ModeratelyNovel
        } else if score > self.config.redundancy_threshold {
            NoveltyLevel::Familiar
        } else {
            NoveltyLevel::Redundant
        };

        let suggested_importance = match level {
            NoveltyLevel::HighlyNovel => (0.7 + self.config.importance_boost_novel).min(1.0),
            NoveltyLevel::ModeratelyNovel => 0.6,
            NoveltyLevel::Familiar => 0.4,
            NoveltyLevel::Redundant => (0.2 - self.config.importance_penalty_redundant).max(0.05),
        };

        NoveltyReport {
            score,
            level,
            nearest_distance,
            nearest_anchor,
            suggested_importance,
            wavelet_energy,
            detail_energy,
            anomaly_score,
        }
    }

    pub fn should_store(&self, report: &NoveltyReport) -> bool {
        !matches!(report.level, NoveltyLevel::Redundant)
    }
}

pub fn wavelet_decompose(data: &[f64]) -> (f64, f64) {
    if data.is_empty() {
        return (0.0, 0.0);
    }

    let mut approx = data.to_vec();
    let mut total_detail_energy = 0.0f64;
    let levels = HAAR_LEVELS.min(data.len().ilog2().max(1) as usize);

    for _ in 0..levels {
        if approx.len() < 2 {
            break;
        }
        let n = approx.len() / 2;
        let mut new_approx = Vec::with_capacity(n);
        let mut detail = Vec::with_capacity(n);

        for i in 0..n {
            let a = approx[2 * i];
            let b = approx[2 * i + 1];
            new_approx.push((a + b) * std::f64::consts::FRAC_1_SQRT_2);
            detail.push((a - b) * std::f64::consts::FRAC_1_SQRT_2);
        }

        for &d in &detail {
            total_detail_energy += d * d;
        }
        approx = new_approx;
    }

    let mut approx_energy = 0.0f64;
    for &v in &approx {
        approx_energy += v * v;
    }

    let total = approx_energy + total_detail_energy;
    (total, total_detail_energy)
}

pub fn wavelet_features(data: &[f64], output_dim: usize) -> Vec<f64> {
    if data.is_empty() {
        return vec![0.0; output_dim];
    }

    let mut features = Vec::with_capacity(output_dim);
    let mut approx = data.to_vec();
    let levels = HAAR_LEVELS.min(data.len().ilog2().max(1) as usize);

    let mut level_energies: Vec<f64> = Vec::new();
    let mut level_entropies: Vec<f64> = Vec::new();

    for _level in 0..levels {
        if approx.len() < 2 {
            break;
        }
        let n = approx.len() / 2;
        let mut new_approx = Vec::with_capacity(n);
        let mut detail = Vec::with_capacity(n);

        for i in 0..n {
            let a = approx[2 * i];
            let b = approx.get(2 * i + 1).copied().unwrap_or(0.0);
            new_approx.push((a + b) * std::f64::consts::FRAC_1_SQRT_2);
            detail.push((a - b) * std::f64::consts::FRAC_1_SQRT_2);
        }

        let energy: f64 = detail.iter().map(|d| d * d).sum();
        level_energies.push(energy);

        let total_energy: f64 = energy.max(1e-20);
        let entropy = -detail
            .iter()
            .map(|d| {
                let p = (d * d) / total_energy;
                if p > 1e-20 {
                    p * p.ln()
                } else {
                    0.0
                }
            })
            .sum::<f64>();
        level_entropies.push(entropy);

        approx = new_approx;
    }

    let final_approx_energy: f64 = approx.iter().map(|v| v * v).sum();
    features.push(final_approx_energy.sqrt());

    for e in &level_energies {
        features.push(e.sqrt());
    }
    for e in &level_entropies {
        features.push(*e);
    }

    let total_energy: f64 = level_energies.iter().sum();
    if total_energy > 0.0 {
        for e in &level_energies {
            features.push(*e / total_energy);
        }
    }

    features.truncate(output_dim);
    while features.len() < output_dim {
        features.push(0.0);
    }

    features
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::hebbian::HebbianMemory;

    #[test]
    fn novelty_high_for_unique_data() {
        let detector = NoveltyDetector::default();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let hebbian = HebbianMemory::new();
        let anchor = Coord7D::new_even([0; 7]);
        let knn: Vec<(f64, usize)> = vec![(1.8, 0)];

        let report = detector.assess(&data, &knn, &anchor, &hebbian, &[]);
        assert!(
            matches!(
                report.level,
                NoveltyLevel::HighlyNovel | NoveltyLevel::ModeratelyNovel
            ),
            "unique data should not be familiar/redundant: {:?}",
            report.level
        );
        assert!(report.suggested_importance > 0.5);
    }

    #[test]
    fn novelty_low_for_duplicate_data() {
        let detector = NoveltyDetector::default();
        let data = vec![5.0; 8];
        let hebbian = HebbianMemory::new();
        let anchor = Coord7D::new_even([0; 7]);
        let knn: Vec<(f64, usize)> = vec![(0.02, 0)];

        let report = detector.assess(&data, &knn, &anchor, &hebbian, &[]);
        assert!(
            matches!(
                report.level,
                NoveltyLevel::Redundant | NoveltyLevel::Familiar
            ),
            "duplicate data should not be novel: {:?}",
            report.level
        );
        assert!(report.suggested_importance < 0.5);
    }

    #[test]
    fn novelty_moderate_for_similar_data() {
        let detector = NoveltyDetector::default();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let hebbian = HebbianMemory::new();
        let anchor = Coord7D::new_even([0; 7]);
        let knn: Vec<(f64, usize)> = vec![(0.5, 0)];

        let report = detector.assess(&data, &knn, &anchor, &hebbian, &[]);
        assert_eq!(report.level, NoveltyLevel::ModeratelyNovel);
        assert!(detector.should_store(&report));
    }

    #[test]
    fn hebbian_isolation_increases_novelty() {
        let detector = NoveltyDetector::default();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let anchor = Coord7D::new_even([0; 7]);
        let knn: Vec<(f64, usize)> = vec![(0.5, 0)];

        let hebbian_empty = HebbianMemory::new();
        let report_isolated = detector.assess(&data, &knn, &anchor, &hebbian_empty, &[]);

        let mut hebbian_connected = HebbianMemory::new();
        for i in 1..10i32 {
            let n = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            hebbian_connected.record_path(&[anchor, n], 1.0);
        }
        let report_connected = detector.assess(&data, &knn, &anchor, &hebbian_connected, &[]);

        assert!(
            report_isolated.score > report_connected.score,
            "isolated node should be more novel: {} vs {}",
            report_isolated.score,
            report_connected.score
        );
    }

    #[test]
    fn wavelet_decompose_nonzero() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let (total, detail) = wavelet_decompose(&data);
        assert!(total > 0.0);
        assert!(detail > 0.0);
        assert!(detail < total);
    }

    #[test]
    fn wavelet_decompose_empty() {
        let (total, detail) = wavelet_decompose(&[]);
        assert_eq!(total, 0.0);
        assert_eq!(detail, 0.0);
    }

    #[test]
    fn wavelet_features_dimension() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let features = wavelet_features(&data, 8);
        assert_eq!(features.len(), 8);
    }

    #[test]
    fn wavelet_features_constant_data_zero_detail() {
        let data = vec![5.0; 16];
        let (_, detail) = wavelet_decompose(&data);
        assert!(
            detail < 1e-10,
            "constant signal should have zero detail energy"
        );
    }

    #[test]
    fn novelty_display_format() {
        assert_eq!(format!("{}", NoveltyLevel::HighlyNovel), "highly_novel");
        assert_eq!(format!("{}", NoveltyLevel::Redundant), "redundant");
    }
}
