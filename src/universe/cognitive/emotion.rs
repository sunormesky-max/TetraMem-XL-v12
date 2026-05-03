// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::node::DarkUniverse;
use serde::{Deserialize, Serialize};
use std::fmt;

const PLEASURE_DIM: usize = 3;
const AROUSAL_DIM: usize = 4;
const DOMINANCE_DIM: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PadVector {
    pub pleasure: f64,
    pub arousal: f64,
    pub dominance: f64,
}

impl PadVector {
    pub fn new(pleasure: f64, arousal: f64, dominance: f64) -> Self {
        Self {
            pleasure: pleasure.clamp(-1.0, 1.0),
            arousal: arousal.clamp(-1.0, 1.0),
            dominance: dominance.clamp(-1.0, 1.0),
        }
    }

    pub fn neutral() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn emotional_quadrant(&self) -> EmotionalQuadrant {
        match (self.pleasure, self.arousal) {
            (p, a) if p > 0.3 && a > 0.3 => EmotionalQuadrant::Excited,
            (p, a) if p > 0.3 && a < -0.3 => EmotionalQuadrant::Calm,
            (p, a) if p < -0.3 && a > 0.3 => EmotionalQuadrant::Agitated,
            (p, a) if p < -0.3 && a < -0.3 => EmotionalQuadrant::Bored,
            _ => EmotionalQuadrant::Neutral,
        }
    }

    pub fn dominance_label(&self) -> &'static str {
        if self.dominance > 0.3 {
            "in_control"
        } else if self.dominance < -0.3 {
            "uncertain"
        } else {
            "balanced"
        }
    }

    pub fn magnitude(&self) -> f64 {
        (self.pleasure * self.pleasure
            + self.arousal * self.arousal
            + self.dominance * self.dominance)
            .sqrt()
    }
}

impl fmt::Display for PadVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PAD[P:{:+.2} A:{:+.2} D:{:+.2}|{} {}]",
            self.pleasure,
            self.arousal,
            self.dominance,
            self.emotional_quadrant(),
            self.dominance_label()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmotionalQuadrant {
    Excited,
    Calm,
    Agitated,
    Bored,
    Neutral,
}

impl fmt::Display for EmotionalQuadrant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Excited => write!(f, "excited"),
            Self::Calm => write!(f, "calm"),
            Self::Agitated => write!(f, "agitated"),
            Self::Bored => write!(f, "bored"),
            Self::Neutral => write!(f, "neutral"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PulseStrategySuggestion {
    Reinforcing,
    Exploratory,
    Cascade,
    Balanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionReading {
    pub pad: PadVector,
    pub quadrant: EmotionalQuadrant,
    pub pulse_suggestion: PulseStrategySuggestion,
    pub dream_frequency_multiplier: f64,
    pub crystal_threshold_modifier: f64,
    pub energy_utilization: f64,
    pub manifested_ratio: f64,
}

impl fmt::Display for EmotionReading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Emotion[{} pulse={:?} dream_x{:.1} crystal_mod:{:+.2} util={:.1}%]",
            self.pad,
            self.pulse_suggestion,
            self.dream_frequency_multiplier,
            self.crystal_threshold_modifier,
            self.energy_utilization * 100.0
        )
    }
}

pub struct EmotionMapper {
    pub pleasure_scale: f64,
    pub arousal_scale: f64,
    pub dominance_scale: f64,
}

impl Default for EmotionMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl EmotionMapper {
    pub fn new() -> Self {
        Self {
            pleasure_scale: 1.0,
            arousal_scale: 1.0,
            dominance_scale: 1.0,
        }
    }

    pub fn read(universe: &DarkUniverse) -> EmotionReading {
        let stats = universe.stats();
        let mapper = Self::new();
        let pad = mapper.compute_pad(universe);

        let pulse_suggestion = Self::suggest_pulse(&pad);
        let dream_frequency_multiplier = Self::dream_multiplier(&pad);
        let crystal_threshold_modifier = Self::crystal_modifier(&pad);

        EmotionReading {
            quadrant: pad.emotional_quadrant(),
            pad,
            pulse_suggestion,
            dream_frequency_multiplier,
            crystal_threshold_modifier,
            energy_utilization: stats.utilization,
            manifested_ratio: if stats.active_nodes > 0 {
                stats.manifested_nodes as f64 / stats.active_nodes as f64
            } else {
                0.0
            },
        }
    }

    fn compute_pad(&self, universe: &DarkUniverse) -> PadVector {
        let mut dim_totals = [0.0f64; 7];
        let mut count = 0usize;

        for coord in universe.coords_iter() {
            if let Some(node) = universe.get_node(&coord) {
                for (d, &v) in node.energy().dims().iter().enumerate() {
                    dim_totals[d] += v;
                }
                count += 1;
            }
        }

        if count == 0 {
            return PadVector::neutral();
        }

        let e_total = dim_totals[PLEASURE_DIM];
        let s_total = dim_totals[AROUSAL_DIM];
        let t_total = dim_totals[DOMINANCE_DIM];

        let total_energy = dim_totals.iter().sum::<f64>();
        if total_energy == 0.0 {
            return PadVector::neutral();
        }

        let e_ratio = e_total / total_energy;
        let s_ratio = s_total / total_energy;
        let t_ratio = t_total / total_energy;

        let pleasure = ((e_ratio - 1.0 / 7.0) * 7.0 * self.pleasure_scale).clamp(-1.0, 1.0);
        let arousal = ((s_ratio - 1.0 / 7.0) * 7.0 * self.arousal_scale).clamp(-1.0, 1.0);
        let dominance = ((t_ratio - 1.0 / 7.0) * 7.0 * self.dominance_scale).clamp(-1.0, 1.0);

        PadVector::new(pleasure, arousal, dominance)
    }

    fn suggest_pulse(pad: &PadVector) -> PulseStrategySuggestion {
        match (pad.arousal, pad.pleasure) {
            (a, _) if a > 0.5 => PulseStrategySuggestion::Exploratory,
            (_, p) if p > 0.3 => PulseStrategySuggestion::Reinforcing,
            (a, _) if a < -0.3 => PulseStrategySuggestion::Balanced,
            _ => PulseStrategySuggestion::Balanced,
        }
    }

    fn dream_multiplier(pad: &PadVector) -> f64 {
        if pad.pleasure < -0.3 {
            1.5
        } else if pad.pleasure > 0.5 {
            0.5
        } else {
            1.0
        }
    }

    fn crystal_modifier(pad: &PadVector) -> f64 {
        if pad.dominance > 0.3 {
            -0.2
        } else if pad.dominance < -0.3 {
            0.3
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionReport {
    pub pad: PadVector,
    pub quadrant: EmotionalQuadrant,
    pub pulse_suggestion: PulseStrategySuggestion,
    pub dream_frequency_multiplier: f64,
    pub crystal_threshold_modifier: f64,
    pub energy_utilization: f64,
    pub manifested_ratio: f64,
    pub functional_cluster: String,
    pub functional_valence: String,
    pub functional_arousal: String,
    pub is_positive: bool,
    pub is_high_arousal: bool,
}

impl EmotionReport {
    pub fn analyze(universe: &DarkUniverse) -> Self {
        let reading = EmotionMapper::read(universe);
        let func = crate::universe::cognitive::functional_emotion::FunctionalEmotion::from_pad(
            reading.pad,
            crate::universe::cognitive::functional_emotion::EmotionSource::Functional,
        );
        Self {
            pad: reading.pad,
            quadrant: reading.quadrant,
            pulse_suggestion: reading.pulse_suggestion,
            dream_frequency_multiplier: reading.dream_frequency_multiplier,
            crystal_threshold_modifier: reading.crystal_threshold_modifier,
            energy_utilization: reading.energy_utilization,
            manifested_ratio: reading.manifested_ratio,
            functional_cluster: func.cluster.name().to_string(),
            functional_valence: format!("{:?}", func.valence),
            functional_arousal: format!("{:?}", func.arousal),
            is_positive: func.is_positive(),
            is_high_arousal: func.is_high_arousal(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::coord::Coord7D;

    #[test]
    fn pad_neutral() {
        let pad = PadVector::neutral();
        assert_eq!(pad.pleasure, 0.0);
        assert_eq!(pad.arousal, 0.0);
        assert_eq!(pad.dominance, 0.0);
        assert_eq!(pad.emotional_quadrant(), EmotionalQuadrant::Neutral);
    }

    #[test]
    fn pad_clamp() {
        let pad = PadVector::new(5.0, -5.0, 0.5);
        assert_eq!(pad.pleasure, 1.0);
        assert_eq!(pad.arousal, -1.0);
        assert_eq!(pad.dominance, 0.5);
    }

    #[test]
    fn emotional_quadrants() {
        assert_eq!(
            PadVector::new(0.5, 0.5, 0.0).emotional_quadrant(),
            EmotionalQuadrant::Excited
        );
        assert_eq!(
            PadVector::new(0.5, -0.5, 0.0).emotional_quadrant(),
            EmotionalQuadrant::Calm
        );
        assert_eq!(
            PadVector::new(-0.5, 0.5, 0.0).emotional_quadrant(),
            EmotionalQuadrant::Agitated
        );
        assert_eq!(
            PadVector::new(-0.5, -0.5, 0.0).emotional_quadrant(),
            EmotionalQuadrant::Bored
        );
        assert_eq!(
            PadVector::new(0.1, 0.1, 0.0).emotional_quadrant(),
            EmotionalQuadrant::Neutral
        );
    }

    #[test]
    fn emotion_reading_from_universe() {
        let mut u = DarkUniverse::new(1_000_000.0);
        for i in 0..20i32 {
            let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            u.materialize_biased(c, 100.0, 0.6).unwrap();
        }
        let reading = EmotionMapper::read(&u);
        assert!(reading.pad.magnitude() >= 0.0);
        assert!(reading.dream_frequency_multiplier > 0.0);
    }

    #[test]
    fn pulse_suggestions() {
        assert_eq!(
            EmotionMapper::suggest_pulse(&PadVector::new(0.0, 0.6, 0.0)),
            PulseStrategySuggestion::Exploratory
        );
        assert_eq!(
            EmotionMapper::suggest_pulse(&PadVector::new(0.5, 0.0, 0.0)),
            PulseStrategySuggestion::Reinforcing
        );
        assert_eq!(
            EmotionMapper::suggest_pulse(&PadVector::new(0.0, -0.5, 0.0)),
            PulseStrategySuggestion::Balanced
        );
    }

    #[test]
    fn dream_multiplier_low_pleasure() {
        let pad = PadVector::new(-0.5, 0.0, 0.0);
        assert_eq!(EmotionMapper::dream_multiplier(&pad), 1.5);
    }

    #[test]
    fn crystal_modifier_high_dominance() {
        let pad = PadVector::new(0.0, 0.0, 0.5);
        let mod_val = EmotionMapper::crystal_modifier(&pad);
        assert!(mod_val < 0.0);
    }

    #[test]
    fn emotion_display() {
        let pad = PadVector::new(0.5, -0.5, 0.3);
        let s = format!("{}", pad);
        assert!(s.contains("P:+0.5") && s.contains("calm"));
        assert!(s.contains("calm"));
    }
}
