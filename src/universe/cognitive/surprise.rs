// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use super::prediction::PredictionState;
use crate::universe::coord::Coord7D;
use crate::universe::memory::hebbian::HebbianMemory;
use crate::universe::memory::semantic::SemanticEngine;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurpriseLevel {
    Expected,
    Mild,
    Strong,
    Violation,
}

impl std::fmt::Display for SurpriseLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expected => write!(f, "expected"),
            Self::Mild => write!(f, "mild_surprise"),
            Self::Strong => write!(f, "strong_surprise"),
            Self::Violation => write!(f, "violation"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SurpriseReport {
    pub surprise: f64,
    pub level: SurpriseLevel,
    pub novelty_contribution: f64,
    pub structural_contribution: f64,
    pub prediction_error_contribution: f64,
    pub hebbian_adjustments: usize,
}

impl Default for SurpriseReport {
    fn default() -> Self {
        Self {
            surprise: 0.0,
            level: SurpriseLevel::Expected,
            novelty_contribution: 0.0,
            structural_contribution: 0.0,
            prediction_error_contribution: 0.0,
            hebbian_adjustments: 0,
        }
    }
}

pub struct SurpriseComputer;

impl SurpriseComputer {
    pub fn compute_memory_surprise(
        anchor: &Coord7D,
        data: &[f64],
        hebbian: &HebbianMemory,
        semantic: &SemanticEngine,
        prediction_state: &mut PredictionState,
    ) -> SurpriseReport {
        let novelty = Self::compute_semantic_novelty(data, semantic);
        let structural = Self::compute_structural_surprise(anchor, hebbian);
        let prediction_error = PredictionEngine::compute_surprise(
            prediction_state,
            anchor,
            &hebbian.get_successors(anchor),
        );

        let surprise = novelty * 0.3 + structural * 0.3 + prediction_error * 0.4;

        let level = if surprise > 0.7 {
            SurpriseLevel::Violation
        } else if surprise > 0.45 {
            SurpriseLevel::Strong
        } else if surprise > 0.2 {
            SurpriseLevel::Mild
        } else {
            SurpriseLevel::Expected
        };

        SurpriseReport {
            surprise,
            level,
            novelty_contribution: novelty,
            structural_contribution: structural,
            prediction_error_contribution: prediction_error,
            hebbian_adjustments: 0,
        }
    }

    pub fn apply_prediction_error_correction(
        hebbian: &mut HebbianMemory,
        prediction_state: &PredictionState,
        memories: &[crate::universe::memory::MemoryAtom],
    ) -> usize {
        let mut adjustments = 0;

        for pred in prediction_state.predictions().values() {
            if pred.confidence < 0.2 {
                continue;
            }

            let actual_successors = hebbian.get_successors(&pred.source);
            if actual_successors.is_empty() {
                continue;
            }

            let predicted_coords: std::collections::HashSet<[i32; 7]> =
                pred.predicted_next.iter().map(|(c, _)| c.basis()).collect();

            for (coord, actual_weight) in &actual_successors {
                let was_predicted = predicted_coords.contains(&coord.basis());

                if was_predicted && *actual_weight > 0.3 {
                    hebbian.boost_edge(&pred.source, coord, 0.05);
                    adjustments += 1;
                } else if !was_predicted && *actual_weight > 1.0 {
                    let exists = memories.iter().any(|m| m.anchor() == coord);
                    if exists {
                        hebbian.boost_edge(&pred.source, coord, 0.02);
                        adjustments += 1;
                    }
                }
            }
        }

        adjustments
    }

    fn compute_semantic_novelty(data: &[f64], semantic: &SemanticEngine) -> f64 {
        if data.is_empty() {
            return 0.5;
        }
        let knn = semantic.search_similar(data, 1);
        match knn.first() {
            Some(nearest) => (nearest.distance / 2.0).min(1.0),
            None => 1.0,
        }
    }

    fn compute_structural_surprise(anchor: &Coord7D, hebbian: &HebbianMemory) -> f64 {
        let neighbors = hebbian.get_neighbors(anchor);
        if neighbors.is_empty() {
            return 0.8;
        }
        let avg_weight: f64 =
            neighbors.iter().map(|(_, w)| *w).sum::<f64>() / neighbors.len() as f64;
        let strong_ratio =
            neighbors.iter().filter(|(_, w)| *w > 0.5).count() as f64 / neighbors.len() as f64;
        1.0 - (strong_ratio * 0.6 + (avg_weight / 2.0).min(1.0) * 0.4)
    }

    pub fn should_dream_from_surprise(prediction_state: &PredictionState) -> bool {
        prediction_state.avg_surprise() > 0.4
    }

    pub fn dream_urgency_from_surprise(prediction_state: &PredictionState) -> f64 {
        (prediction_state.avg_surprise() * 2.0).min(1.0)
    }
}

use super::prediction::PredictionEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surprise_level_display() {
        assert_eq!(SurpriseLevel::Expected.to_string(), "expected");
        assert_eq!(SurpriseLevel::Mild.to_string(), "mild_surprise");
        assert_eq!(SurpriseLevel::Strong.to_string(), "strong_surprise");
        assert_eq!(SurpriseLevel::Violation.to_string(), "violation");
    }

    #[test]
    fn default_report_is_expected() {
        let report = SurpriseReport::default();
        assert_eq!(report.level, SurpriseLevel::Expected);
        assert!((report.surprise - 0.0).abs() < 1e-10);
    }

    #[test]
    fn dream_urgency_scales_with_surprise() {
        let mut state = PredictionState::default();
        assert!(!SurpriseComputer::should_dream_from_surprise(&state));
        assert!((SurpriseComputer::dream_urgency_from_surprise(&state) - 0.0).abs() < 1e-10);

        for _ in 0..10 {
            state.record_surprise(0.6);
        }
        assert!(SurpriseComputer::should_dream_from_surprise(&state));
        assert!(SurpriseComputer::dream_urgency_from_surprise(&state) > 0.5);
    }
}
