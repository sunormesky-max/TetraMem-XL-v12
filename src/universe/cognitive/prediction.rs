// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::coord::Coord7D;
use crate::universe::memory::hebbian::HebbianMemory;
use std::collections::HashMap;

const PREDICTION_HORIZON: usize = 5;
const MAX_PREDICTIONS_PER_ANCHOR: usize = 3;
const SURPRISE_HISTORY_LEN: usize = 100;

#[derive(Debug, Clone)]
pub struct Prediction {
    pub source: Coord7D,
    pub predicted_next: Vec<(Coord7D, f64)>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct PredictionState {
    predictions: HashMap<[i32; 7], Prediction>,
    surprise_history: VecDeque<f64>,
    avg_surprise: f64,
    total_predictions: u64,
    correct_predictions: u64,
}

impl Default for PredictionState {
    fn default() -> Self {
        Self {
            predictions: HashMap::new(),
            surprise_history: VecDeque::with_capacity(SURPRISE_HISTORY_LEN),
            avg_surprise: 0.0,
            total_predictions: 0,
            correct_predictions: 0,
        }
    }
}

impl PredictionState {
    pub fn avg_surprise(&self) -> f64 {
        self.avg_surprise
    }

    pub fn prediction_accuracy(&self) -> f64 {
        if self.total_predictions == 0 {
            return 1.0;
        }
        self.correct_predictions as f64 / self.total_predictions as f64
    }

    pub fn active_prediction_count(&self) -> usize {
        self.predictions.len()
    }

    pub fn predictions(&self) -> &HashMap<[i32; 7], Prediction> {
        &self.predictions
    }

    pub fn record_surprise(&mut self, surprise: f64) {
        if self.surprise_history.len() >= SURPRISE_HISTORY_LEN {
            self.surprise_history.pop_front();
        }
        self.surprise_history.push_back(surprise);
        let sum: f64 = self.surprise_history.iter().sum();
        self.avg_surprise = sum / self.surprise_history.len() as f64;
    }
}

use std::collections::VecDeque;

pub struct PredictionEngine;

impl PredictionEngine {
    pub fn generate_predictions(
        hebbian: &HebbianMemory,
        active_anchors: &[Coord7D],
    ) -> Vec<Prediction> {
        let mut predictions = Vec::new();

        for anchor in active_anchors {
            let successors = hebbian.get_successors(anchor);
            let temporal = hebbian.get_temporal_sequence(anchor, PREDICTION_HORIZON);

            if successors.is_empty() && temporal.is_empty() {
                continue;
            }

            let mut predicted: Vec<(Coord7D, f64)> = Vec::new();
            let mut seen = std::collections::HashSet::new();

            for (coord, weight) in successors.iter().take(MAX_PREDICTIONS_PER_ANCHOR) {
                if seen.insert(coord.basis()) {
                    predicted.push((*coord, *weight));
                }
            }

            for (coord, strength) in temporal.iter().take(MAX_PREDICTIONS_PER_ANCHOR) {
                if seen.insert(coord.basis()) {
                    predicted.push((*coord, *strength));
                }
            }

            if predicted.is_empty() {
                continue;
            }

            let confidence = Self::compute_confidence(&predicted, &successors);

            predictions.push(Prediction {
                source: *anchor,
                predicted_next: predicted,
                confidence,
            });
        }

        predictions
    }

    pub fn compute_surprise(
        state: &mut PredictionState,
        observed_anchor: &Coord7D,
        actual_successors: &[(Coord7D, f64)],
    ) -> f64 {
        let prediction = state.predictions.get(&observed_anchor.basis());

        let surprise = match prediction {
            Some(pred) => {
                let predicted_set: std::collections::HashSet<[i32; 7]> =
                    pred.predicted_next.iter().map(|(c, _)| c.basis()).collect();

                let actual_set: std::collections::HashSet<[i32; 7]> =
                    actual_successors.iter().map(|(c, _)| c.basis()).collect();

                let intersection = predicted_set.intersection(&actual_set).count();
                let union = predicted_set.union(&actual_set).count();

                if union == 0 {
                    0.0
                } else {
                    1.0 - (intersection as f64 / union as f64)
                }
            }
            None => 0.5,
        };

        state.total_predictions += 1;
        if surprise < 0.3 {
            state.correct_predictions += 1;
        }

        state.record_surprise(surprise);
        surprise
    }

    pub fn update_predictions(state: &mut PredictionState, new_predictions: Vec<Prediction>) {
        for pred in new_predictions {
            state.predictions.insert(pred.source.basis(), pred);
        }

        if state.predictions.len() > 200 {
            let mut keys: Vec<[i32; 7]> = state.predictions.keys().cloned().collect();
            keys.sort_by(|a, b| {
                let ca = state
                    .predictions
                    .get(a)
                    .map(|p| p.confidence)
                    .unwrap_or(0.0);
                let cb = state
                    .predictions
                    .get(b)
                    .map(|p| p.confidence)
                    .unwrap_or(0.0);
                ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
            });
            while state.predictions.len() > 150 {
                if let Some(k) = keys.pop() {
                    state.predictions.remove(&k);
                } else {
                    break;
                }
            }
        }
    }

    pub fn find_high_uncertainty(state: &PredictionState) -> Vec<(Coord7D, f64)> {
        let mut result: Vec<(Coord7D, f64)> = state
            .predictions
            .values()
            .filter(|p| p.confidence < 0.5)
            .map(|p| (p.source, 1.0 - p.confidence))
            .collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result.truncate(10);
        result
    }

    fn compute_confidence(predicted: &[(Coord7D, f64)], all_successors: &[(Coord7D, f64)]) -> f64 {
        if predicted.is_empty() {
            return 0.0;
        }

        let avg_weight: f64 =
            predicted.iter().map(|(_, w)| *w).sum::<f64>() / predicted.len() as f64;

        let coverage = if all_successors.is_empty() {
            0.0
        } else {
            predicted.len() as f64 / all_successors.len() as f64
        };

        let weight_signal = (avg_weight / 2.0).min(1.0);

        0.6 * weight_signal + 0.4 * coverage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_has_no_predictions() {
        let state = PredictionState::default();
        assert_eq!(state.active_prediction_count(), 0);
        assert_eq!(state.prediction_accuracy(), 1.0);
        assert_eq!(state.avg_surprise(), 0.0);
    }

    #[test]
    fn surprise_history_truncates() {
        let mut state = PredictionState::default();
        for i in 0..150 {
            state.record_surprise(i as f64 * 0.01);
        }
        assert!(state.surprise_history.len() <= SURPRISE_HISTORY_LEN);
    }

    #[test]
    fn prediction_accuracy_updates() {
        let mut state = PredictionState::default();
        let anchor = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);

        PredictionEngine::compute_surprise(&mut state, &anchor, &[]);
        assert_eq!(state.total_predictions, 1);

        PredictionEngine::compute_surprise(&mut state, &anchor, &[]);
        assert_eq!(state.total_predictions, 2);
    }

    #[test]
    fn high_uncertainty_returns_low_confidence() {
        let mut state = PredictionState::default();
        let a = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let b = Coord7D::new_even([1, 1, 1, 1, 1, 1, 1]);

        state.predictions.insert(
            a.basis(),
            Prediction {
                source: a,
                predicted_next: vec![(b, 0.1)],
                confidence: 0.1,
            },
        );

        let uncertain = PredictionEngine::find_high_uncertainty(&state);
        assert_eq!(uncertain.len(), 1);
        assert!(uncertain[0].1 > 0.8);
    }
}
