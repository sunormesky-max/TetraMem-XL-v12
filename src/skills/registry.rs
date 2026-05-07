// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::collections::HashMap;
use std::sync::Arc;

use super::types::{Skill, SkillDescriptor, SkillSignature};

pub struct SkillRegistry {
    skills: HashMap<String, Arc<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: impl Skill + 'static) {
        let sig = skill.signature();
        self.skills.insert(sig.name.clone(), Arc::new(skill));
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Skill>> {
        self.skills.get(name).cloned()
    }

    pub fn list(&self) -> Vec<SkillDescriptor> {
        self.skills
            .values()
            .map(|s| {
                let sig = s.signature();
                let cat = infer_category(&sig.name);
                SkillDescriptor {
                    name: sig.name,
                    version: sig.version,
                    description: sig.description,
                    category: cat,
                }
            })
            .collect()
    }

    pub fn signatures(&self) -> Vec<SkillSignature> {
        self.skills.values().map(|s| s.signature()).collect()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.skills.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

fn infer_category(name: &str) -> super::types::SkillCategory {
    use super::types::SkillCategory::*;
    match name {
        n if n.contains("encode") || n.contains("decode") || n.contains("erase") => Memory,
        n if n.contains("pulse") || n.contains("dream") || n.contains("reason") => Cognitive,
        n if n.contains("topology") || n.contains("stats") || n.contains("health") => Analysis,
        n if n.contains("regulate") || n.contains("scale") || n.contains("conservation") => System,
        n if n.contains("hebbian") || n.contains("crystal") || n.contains("perception") => Learning,
        _ => Analysis,
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::types::{Skill, SkillContext, SkillError, SkillSignature};
    use serde_json::{json, Value};

    struct DummySkill;
    impl Skill for DummySkill {
        fn signature(&self) -> SkillSignature {
            SkillSignature {
                name: "dummy".into(),
                version: "1.0".into(),
                description: "test".into(),
                input_schema: json!({}),
                output_schema: json!({}),
            }
        }
        fn execute(&self, _ctx: &mut SkillContext, _args: &Value) -> Result<Value, SkillError> {
            Ok(json!({"ok": true}))
        }
    }

    #[test]
    fn new_registry_is_empty() {
        let r = SkillRegistry::new();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn register_and_get() {
        let mut r = SkillRegistry::new();
        r.register(DummySkill);
        assert_eq!(r.len(), 1);
        assert!(r.contains("dummy"));
        assert!(!r.contains("nonexistent"));
        let skill = r.get("dummy").unwrap();
        let sig = skill.signature();
        assert_eq!(sig.name, "dummy");
    }

    #[test]
    fn list_returns_descriptors() {
        let mut r = SkillRegistry::new();
        r.register(DummySkill);
        let list = r.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "dummy");
    }

    #[test]
    fn signatures_returns_all() {
        let mut r = SkillRegistry::new();
        r.register(DummySkill);
        let sigs = r.signatures();
        assert_eq!(sigs.len(), 1);
    }

    #[test]
    fn builtin_register_all() {
        let mut r = SkillRegistry::new();
        crate::skills::builtin::register_all(&mut r);
        assert!(r.len() >= 8);
        assert!(r.contains("encode_memory"));
        assert!(r.contains("decode_memory"));
        assert!(r.contains("fire_pulse"));
        assert!(r.contains("run_dream"));
        assert!(r.contains("analyze_topology"));
        assert!(r.contains("regulate_dimensions"));
        assert!(r.contains("trace_associations"));
        assert!(r.contains("check_conservation"));
    }
}
