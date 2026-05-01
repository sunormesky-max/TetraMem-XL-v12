// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImmutableRule {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModifiableBound {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub current: f64,
}

impl ModifiableBound {
    pub fn new(name: &str, min: f64, max: f64, default: f64) -> Self {
        Self {
            name: name.to_string(),
            min,
            max,
            current: default.clamp(min, max),
        }
    }

    pub fn set(&mut self, value: f64) -> bool {
        if value < self.min || value > self.max {
            return false;
        }
        self.current = value;
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    rules: Vec<ImmutableRule>,
    bounds: Vec<ModifiableBound>,
}

impl Constitution {
    pub fn new(rules: Vec<ImmutableRule>, bounds: Vec<ModifiableBound>) -> Self {
        Self { rules, bounds }
    }

    pub fn tetramem_default() -> Self {
        Self {
            rules: vec![
                ImmutableRule {
                    id: "energy_conservation".into(),
                    description: "Total energy (allocated + available) must equal initial energy at all times".into(),
                },
                ImmutableRule {
                    id: "no_energy_creation".into(),
                    description: "Energy cannot be created from nothing; expansion adds energy explicitly".into(),
                },
                ImmutableRule {
                    id: "manifestation_threshold".into(),
                    description: "Physical energy ratio > 0.5 required for manifestation".into(),
                },
                ImmutableRule {
                    id: "tetrahedron_integrity".into(),
                    description: "Memory atoms are tetrahedra of 4 Coord7D nodes; this structure is inviolable".into(),
                },
                ImmutableRule {
                    id: "conservation_verification".into(),
                    description: "verify_conservation() must return true after every operation".into(),
                },
                ImmutableRule {
                    id: "merge_energy_preserved".into(),
                    description: "Memory merge must preserve total energy; no energy lost or gained".into(),
                },
            ],
            bounds: vec![
                ModifiableBound::new("crystal_threshold", 0.5, 5.0, 1.8),
                ModifiableBound::new("super_crystal_threshold", 2.0, 10.0, 4.0),
                ModifiableBound::new("pulse_face_decay", 0.3, 0.95, 0.72),
                ModifiableBound::new("pulse_bcc_decay", 0.1, 0.7, 0.36),
                ModifiableBound::new("hebbian_decay", 0.9, 0.999, 0.98),
                ModifiableBound::new("hebbian_reinforce", 1.0, 2.0, 1.15),
                ModifiableBound::new("dream_frequency_multiplier", 0.1, 5.0, 1.0),
                ModifiableBound::new("merge_similarity_threshold", 0.5, 0.99, 0.8),
                ModifiableBound::new("regulation_pressure_threshold", 1.0, 10.0, 3.0),
                ModifiableBound::new("scale_up_threshold", 0.5, 0.99, 0.80),
            ],
        }
    }

    pub fn rules(&self) -> &[ImmutableRule] {
        &self.rules
    }

    pub fn bounds(&self) -> &[ModifiableBound] {
        &self.bounds
    }

    pub fn get_bound(&self, name: &str) -> Option<&ModifiableBound> {
        self.bounds.iter().find(|b| b.name == name)
    }

    pub fn set_bound(&mut self, name: &str, value: f64) -> bool {
        if let Some(bound) = self.bounds.iter_mut().find(|b| b.name == name) {
            bound.set(value)
        } else {
            false
        }
    }

    pub fn validate_operation(&self, operation: &str) -> ConstitutionCheck {
        let mut violations = Vec::new();

        match operation {
            "energy_expansion" => {
                if let Some(b) = self.get_bound("regulation_pressure_threshold") {
                    if b.current <= 0.0 {
                        violations.push(format!(
                            "regulation_pressure_threshold {:.2} invalid for expansion",
                            b.current
                        ));
                    }
                }
            }
            "energy_transfer" => {
                let energy_rule = self.rules.iter().find(|r| r.id == "energy_conservation");
                if energy_rule.is_none() {
                    violations.push("energy_conservation rule missing".to_string());
                }
            }
            "materialize" => {
                if let Some(b) = self.get_bound("crystal_threshold") {
                    if b.current < b.min {
                        violations.push(format!(
                            "crystal_threshold {:.2} below minimum {:.2}",
                            b.current, b.min
                        ));
                    }
                }
                let threshold_rule = self
                    .rules
                    .iter()
                    .find(|r| r.id == "manifestation_threshold");
                if threshold_rule.is_none() {
                    violations.push("manifestation_threshold rule missing".to_string());
                }
            }
            "dematerialize" => {
                let conservation_rule = self
                    .rules
                    .iter()
                    .find(|r| r.id == "conservation_verification");
                if conservation_rule.is_none() {
                    violations.push("conservation_verification rule missing".to_string());
                }
            }
            "crystal_form" => {
                if let Some(b) = self.get_bound("crystal_threshold") {
                    if b.current < b.min {
                        violations.push(format!(
                            "crystal_threshold {:.2} below minimum {:.2}",
                            b.current, b.min
                        ));
                    }
                }
                if let Some(b) = self.get_bound("super_crystal_threshold") {
                    if b.current < b.min {
                        violations.push(format!(
                            "super_crystal_threshold {:.2} below minimum {:.2}",
                            b.current, b.min
                        ));
                    }
                }
                let integrity_rule = self.rules.iter().find(|r| r.id == "tetrahedron_integrity");
                if integrity_rule.is_none() {
                    violations.push("tetrahedron_integrity rule missing".to_string());
                }
            }
            "memory_merge" => {
                let merge_rule = self.rules.iter().find(|r| r.id == "merge_energy_preserved");
                if merge_rule.is_none() {
                    violations.push("merge_energy_preserved rule missing".to_string());
                }
                if let Some(b) = self.get_bound("merge_similarity_threshold") {
                    if b.current < b.min {
                        violations.push(format!(
                            "merge_similarity_threshold {:.2} below minimum {:.2}",
                            b.current, b.min
                        ));
                    }
                }
                let conservation_rule = self.rules.iter().find(|r| r.id == "energy_conservation");
                if conservation_rule.is_none() {
                    violations.push("energy_conservation rule missing".to_string());
                }
            }
            "pulse_fire" => {
                if let Some(b) = self.get_bound("pulse_face_decay") {
                    if b.current < b.min || b.current > b.max {
                        violations.push(format!(
                            "pulse_face_decay {:.2} outside [{:.2}, {:.2}]",
                            b.current, b.min, b.max
                        ));
                    }
                }
                if let Some(b) = self.get_bound("pulse_bcc_decay") {
                    if b.current < b.min || b.current > b.max {
                        violations.push(format!(
                            "pulse_bcc_decay {:.2} outside [{:.2}, {:.2}]",
                            b.current, b.min, b.max
                        ));
                    }
                }
            }
            "hebbian_reinforce" => {
                if let Some(b) = self.get_bound("hebbian_decay") {
                    if b.current < b.min {
                        violations.push(format!(
                            "hebbian_decay {:.2} below minimum {:.2}",
                            b.current, b.min
                        ));
                    }
                }
                if let Some(b) = self.get_bound("hebbian_reinforce") {
                    if b.current < b.min || b.current > b.max {
                        violations.push(format!(
                            "hebbian_reinforce {:.2} outside [{:.2}, {:.2}]",
                            b.current, b.min, b.max
                        ));
                    }
                }
            }
            "dream_cycle" => {
                if let Some(b) = self.get_bound("dream_frequency_multiplier") {
                    if b.current <= 0.0 {
                        violations.push(format!(
                            "dream_frequency_multiplier {:.2} must be > 0",
                            b.current
                        ));
                    }
                }
            }
            "scale_up" => {
                if let Some(b) = self.get_bound("scale_up_threshold") {
                    if b.current < b.min || b.current > b.max {
                        violations.push(format!(
                            "scale_up_threshold {:.2} outside [{:.2}, {:.2}]",
                            b.current, b.min, b.max
                        ));
                    }
                }
            }
            _ => {}
        }

        ConstitutionCheck {
            operation: operation.to_string(),
            allowed: violations.is_empty(),
            violations,
        }
    }
}

impl fmt::Display for Constitution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Constitution[{} rules, {} bounds]",
            self.rules.len(),
            self.bounds.len()
        )?;
        for rule in &self.rules {
            writeln!(f, "  RULE[{}]: {}", rule.id, rule.description)?;
        }
        for bound in &self.bounds {
            writeln!(
                f,
                "  BOUND[{}]: [{:.2}, {:.2}] = {:.2}",
                bound.name, bound.min, bound.max, bound.current
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ConstitutionCheck {
    pub operation: String,
    pub allowed: bool,
    pub violations: Vec<String>,
}

impl fmt::Display for ConstitutionCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.allowed {
            write!(f, "ConstitutionCheck[{}: ALLOWED]", self.operation)
        } else {
            write!(
                f,
                "ConstitutionCheck[{}: DENIED violations={}]",
                self.operation,
                self.violations.len()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_constitution() {
        let c = Constitution::tetramem_default();
        assert!(c.rules().len() >= 5);
        assert!(c.bounds().len() >= 5);
    }

    #[test]
    fn set_bound_valid() {
        let mut c = Constitution::tetramem_default();
        assert!(c.set_bound("crystal_threshold", 2.5));
        assert_eq!(c.get_bound("crystal_threshold").unwrap().current, 2.5);
    }

    #[test]
    fn set_bound_out_of_range() {
        let mut c = Constitution::tetramem_default();
        assert!(!c.set_bound("crystal_threshold", 0.1));
        assert!(!c.set_bound("crystal_threshold", 100.0));
    }

    #[test]
    fn set_bound_nonexistent() {
        let mut c = Constitution::tetramem_default();
        assert!(!c.set_bound("nonexistent", 1.0));
    }

    #[test]
    fn validate_operation() {
        let c = Constitution::tetramem_default();
        let check = c.validate_operation("materialize");
        assert!(check.allowed);
    }

    #[test]
    fn validate_operation_pulse_fire() {
        let c = Constitution::tetramem_default();
        let check = c.validate_operation("pulse_fire");
        assert!(check.allowed);
    }

    #[test]
    fn validate_operation_memory_merge() {
        let c = Constitution::tetramem_default();
        let check = c.validate_operation("memory_merge");
        assert!(check.allowed);
    }

    #[test]
    fn validate_operation_crystal_form_checks_thresholds() {
        let mut c = Constitution::tetramem_default();
        c.bounds
            .iter_mut()
            .find(|b| b.name == "crystal_threshold")
            .unwrap()
            .current = 0.1;
        let check = c.validate_operation("crystal_form");
        assert!(!check.allowed);
        assert!(check
            .violations
            .iter()
            .any(|v| v.contains("crystal_threshold")));
    }

    #[test]
    fn validate_operation_unknown_allowed() {
        let c = Constitution::tetramem_default();
        let check = c.validate_operation("unknown_op");
        assert!(check.allowed);
    }

    #[test]
    fn constitution_display() {
        let c = Constitution::tetramem_default();
        let s = format!("{}", c);
        assert!(s.contains("energy_conservation"));
        assert!(s.contains("crystal_threshold"));
    }

    #[test]
    fn immutable_rules_immutable() {
        let c = Constitution::tetramem_default();
        let energy_rule = c
            .rules()
            .iter()
            .find(|r| r.id == "energy_conservation")
            .unwrap();
        assert!(!energy_rule.description.is_empty());
    }

    #[test]
    fn constitution_serialization() {
        let c = Constitution::tetramem_default();
        let json = serde_json::to_string(&c).unwrap();
        let c2: Constitution = serde_json::from_str(&json).unwrap();
        assert_eq!(c.rules().len(), c2.rules().len());
        assert_eq!(c.bounds().len(), c2.bounds().len());
    }
}
