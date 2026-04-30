use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::universe::crystal::CrystalEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSignature {
    pub name: String,
    pub version: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Value,
}

pub trait Skill: Send + Sync {
    fn signature(&self) -> SkillSignature;
    fn execute(&self, ctx: &mut SkillContext, args: &Value) -> Result<Value, SkillError>;
}

#[derive(Debug, Clone)]
pub struct SkillError {
    pub skill: String,
    pub message: String,
}

impl std::fmt::Display for SkillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SkillError[{}]: {}", self.skill, self.message)
    }
}

impl std::error::Error for SkillError {}

impl SkillError {
    pub fn new(skill: &str, message: impl Into<String>) -> Self {
        Self { skill: skill.into(), message: message.into() }
    }
}

pub struct SkillContext<'a> {
    pub universe: &'a mut DarkUniverse,
    pub hebbian: &'a mut HebbianMemory,
    pub memories: &'a mut Vec<MemoryAtom>,
    pub crystal: &'a mut CrystalEngine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDescriptor {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: SkillCategory,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SkillCategory {
    Memory,
    Cognitive,
    Analysis,
    System,
    Learning,
}
