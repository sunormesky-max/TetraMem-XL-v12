// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
// Inspired by Anthropic's "Emotion Concepts and their Function in a Large Language Model" (2026)
use crate::universe::cognitive::emotion::{EmotionalQuadrant, PadVector};
use serde::{Deserialize, Serialize};
use std::fmt;

const NUM_CLUSTERS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Valence {
    Positive,
    Negative,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArousalLevel {
    High,
    Low,
    Medium,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmotionCluster {
    JoyExcitement,
    ContentmentSerenity,
    LoveAffection,
    PrideAccomplishment,
    SadnessGrief,
    FearAnxiety,
    AngerHostility,
    GuiltShame,
    SurpriseAwe,
    DesperationPanic,
}

impl EmotionCluster {
    pub fn all() -> &'static [EmotionCluster; NUM_CLUSTERS] {
        &[
            EmotionCluster::JoyExcitement,
            EmotionCluster::ContentmentSerenity,
            EmotionCluster::LoveAffection,
            EmotionCluster::PrideAccomplishment,
            EmotionCluster::SadnessGrief,
            EmotionCluster::FearAnxiety,
            EmotionCluster::AngerHostility,
            EmotionCluster::GuiltShame,
            EmotionCluster::SurpriseAwe,
            EmotionCluster::DesperationPanic,
        ]
    }

    pub fn valence(self) -> Valence {
        match self {
            EmotionCluster::JoyExcitement
            | EmotionCluster::ContentmentSerenity
            | EmotionCluster::LoveAffection
            | EmotionCluster::PrideAccomplishment => Valence::Positive,
            EmotionCluster::SadnessGrief
            | EmotionCluster::FearAnxiety
            | EmotionCluster::AngerHostility
            | EmotionCluster::GuiltShame
            | EmotionCluster::DesperationPanic => Valence::Negative,
            EmotionCluster::SurpriseAwe => Valence::Neutral,
        }
    }

    pub fn arousal(self) -> ArousalLevel {
        match self {
            EmotionCluster::JoyExcitement
            | EmotionCluster::FearAnxiety
            | EmotionCluster::AngerHostility
            | EmotionCluster::DesperationPanic
            | EmotionCluster::SurpriseAwe => ArousalLevel::High,
            EmotionCluster::ContentmentSerenity | EmotionCluster::GuiltShame => ArousalLevel::Low,
            EmotionCluster::LoveAffection
            | EmotionCluster::PrideAccomplishment
            | EmotionCluster::SadnessGrief => ArousalLevel::Medium,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            EmotionCluster::JoyExcitement => "joy/excitement",
            EmotionCluster::ContentmentSerenity => "contentment/serenity",
            EmotionCluster::LoveAffection => "love/affection",
            EmotionCluster::PrideAccomplishment => "pride/accomplishment",
            EmotionCluster::SadnessGrief => "sadness/grief",
            EmotionCluster::FearAnxiety => "fear/anxiety",
            EmotionCluster::AngerHostility => "anger/hostility",
            EmotionCluster::GuiltShame => "guilt/shame",
            EmotionCluster::SurpriseAwe => "surprise/awe",
            EmotionCluster::DesperationPanic => "desperation/panic",
        }
    }

    pub fn from_pad(pad: &PadVector) -> Self {
        let quadrant = pad.emotional_quadrant();
        match quadrant {
            EmotionalQuadrant::Excited => {
                if pad.dominance > 0.3 {
                    EmotionCluster::PrideAccomplishment
                } else {
                    EmotionCluster::JoyExcitement
                }
            }
            EmotionalQuadrant::Calm => {
                if pad.dominance > 0.0 {
                    EmotionCluster::LoveAffection
                } else {
                    EmotionCluster::ContentmentSerenity
                }
            }
            EmotionalQuadrant::Agitated => {
                if pad.pleasure < -0.6 {
                    EmotionCluster::DesperationPanic
                } else if pad.dominance < -0.3 {
                    EmotionCluster::FearAnxiety
                } else {
                    EmotionCluster::AngerHostility
                }
            }
            EmotionalQuadrant::Bored => {
                if pad.dominance < -0.3 {
                    EmotionCluster::GuiltShame
                } else {
                    EmotionCluster::SadnessGrief
                }
            }
            EmotionalQuadrant::Neutral => EmotionCluster::SurpriseAwe,
        }
    }
}

impl fmt::Display for EmotionCluster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalEmotion {
    pub pad: PadVector,
    pub cluster: EmotionCluster,
    pub valence: Valence,
    pub arousal: ArousalLevel,
    pub source: EmotionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmotionSource {
    Perceived,
    Functional,
}

impl FunctionalEmotion {
    pub fn from_pad(pad: PadVector, source: EmotionSource) -> Self {
        let cluster = EmotionCluster::from_pad(&pad);
        Self {
            valence: cluster.valence(),
            arousal: cluster.arousal(),
            cluster,
            pad,
            source,
        }
    }

    pub fn is_positive(&self) -> bool {
        matches!(self.valence, Valence::Positive)
    }

    pub fn is_high_arousal(&self) -> bool {
        matches!(self.arousal, ArousalLevel::High)
    }

    pub fn steered_profile_weights(&self) -> [f64; 7] {
        let base: [f64; 7] = [1.0, 1.0, 1.0, 1.2, 1.0, 0.8, 1.5];
        let valence_mod: [f64; 7] = match self.valence {
            Valence::Positive => [0.0, 0.0, 0.0, 0.2, 0.1, 0.0, 0.1],
            Valence::Negative => [0.0, 0.0, 0.0, -0.2, -0.1, 0.0, -0.1],
            Valence::Neutral => [0.0; 7],
        };
        let arousal_mod: [f64; 7] = match self.arousal {
            ArousalLevel::High => [0.1, 0.1, 0.1, 0.0, 0.0, 0.0, 0.0],
            ArousalLevel::Low => [-0.1, -0.1, -0.1, 0.0, 0.0, 0.0, 0.0],
            ArousalLevel::Medium => [0.0; 7],
        };
        let mut result = [0.0f64; 7];
        for i in 0..7 {
            result[i] = (base[i] + valence_mod[i] + arousal_mod[i]).max(0.1);
        }
        result
    }
}

impl fmt::Display for FunctionalEmotion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FuncEmotion[{} {:?} {:?} {:?} {}]",
            self.cluster.name(),
            self.valence,
            self.arousal,
            self.source,
            self.pad,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster_from_pad_excited() {
        let pad = PadVector::new(0.8, 0.8, 0.0);
        let cluster = EmotionCluster::from_pad(&pad);
        assert_eq!(cluster, EmotionCluster::JoyExcitement);
        assert_eq!(cluster.valence(), Valence::Positive);
        assert_eq!(cluster.arousal(), ArousalLevel::High);
    }

    #[test]
    fn cluster_from_pad_calm() {
        let pad = PadVector::new(0.8, -0.8, 0.0);
        let cluster = EmotionCluster::from_pad(&pad);
        assert_eq!(cluster.valence(), Valence::Positive);
        assert_eq!(cluster.arousal(), ArousalLevel::Low);
    }

    #[test]
    fn cluster_from_pad_agitated_desperate() {
        let pad = PadVector::new(-0.8, 0.8, 0.0);
        let cluster = EmotionCluster::from_pad(&pad);
        assert_eq!(cluster.valence(), Valence::Negative);
    }

    #[test]
    fn cluster_from_pad_bored_sad() {
        let pad = PadVector::new(-0.8, -0.8, 0.0);
        let cluster = EmotionCluster::from_pad(&pad);
        assert_eq!(cluster, EmotionCluster::SadnessGrief);
    }

    #[test]
    fn functional_emotion_from_pad() {
        let pad = PadVector::new(0.6, 0.6, 0.4);
        let fe = FunctionalEmotion::from_pad(pad, EmotionSource::Functional);
        assert!(fe.is_positive());
        assert_eq!(fe.cluster, EmotionCluster::PrideAccomplishment);
        assert_eq!(fe.arousal, ArousalLevel::Medium);
        assert_eq!(fe.source, EmotionSource::Functional);
    }

    #[test]
    fn steered_profile_weights_positive() {
        let pad = PadVector::new(0.8, 0.8, 0.0);
        let fe = FunctionalEmotion::from_pad(pad, EmotionSource::Functional);
        let weights = fe.steered_profile_weights();
        assert!(
            weights[3] > 1.2,
            "E dimension should be boosted for positive valence"
        );
    }

    #[test]
    fn steered_profile_weights_negative() {
        let pad = PadVector::new(-0.8, -0.8, 0.0);
        let fe = FunctionalEmotion::from_pad(pad, EmotionSource::Perceived);
        let weights = fe.steered_profile_weights();
        assert!(
            weights[3] < 1.2,
            "E dimension should be reduced for negative valence"
        );
    }

    #[test]
    fn all_clusters_covered() {
        for cluster in EmotionCluster::all() {
            let _name = cluster.name();
            let _valence = cluster.valence();
            let _arousal = cluster.arousal();
        }
    }

    #[test]
    fn emotion_display() {
        let pad = PadVector::new(0.5, 0.5, 0.0);
        let fe = FunctionalEmotion::from_pad(pad, EmotionSource::Functional);
        let s = format!("{}", fe);
        assert!(s.contains("joy/excitement"));
    }

    #[test]
    fn perceived_vs_functional_source() {
        let pad = PadVector::new(0.0, 0.0, 0.0);
        let p = FunctionalEmotion::from_pad(pad, EmotionSource::Perceived);
        let f = FunctionalEmotion::from_pad(pad, EmotionSource::Functional);
        assert_eq!(p.source, EmotionSource::Perceived);
        assert_eq!(f.source, EmotionSource::Functional);
    }
}
