use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use super::types::{Skill, SkillDescriptor, SkillError, SkillSignature};

pub struct SkillRegistry {
    skills: HashMap<String, Arc<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: HashMap::new() }
    }

    pub fn register(&mut self, skill: impl Skill + 'static) {
        let sig = skill.signature();
        self.skills.insert(sig.name.clone(), Arc::new(skill));
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Skill>> {
        self.skills.get(name).cloned()
    }

    pub fn list(&self) -> Vec<SkillDescriptor> {
        self.skills.values().map(|s| {
            let sig = s.signature();
            let cat = infer_category(&sig.name);
            SkillDescriptor {
                name: sig.name,
                version: sig.version,
                description: sig.description,
                category: cat,
            }
        }).collect()
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
