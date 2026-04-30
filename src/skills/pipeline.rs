use serde_json::Value;

use super::registry::SkillRegistry;
use super::types::{Skill, SkillContext, SkillError};

pub struct SkillPipeline {
    registry: SkillRegistry,
}

impl SkillPipeline {
    pub fn new(registry: SkillRegistry) -> Self {
        Self { registry }
    }

    pub fn registry(&self) -> &SkillRegistry {
        &self.registry
    }

    pub fn execute_chain(
        &self,
        steps: &[PipelineStep],
        ctx: &mut SkillContext,
    ) -> Result<Vec<PipelineResult>, PipelineError> {
        let mut results = Vec::new();
        let mut carry = Value::Null;

        for (i, step) in steps.iter().enumerate() {
            let skill = self.registry.get(&step.skill)
                .ok_or_else(|| PipelineError {
                    step: i,
                    message: format!("skill not found: {}", step.skill),
                })?;

            let args = if step.args.is_null() && !carry.is_null() {
                carry.clone()
            } else {
                step.args.clone()
            };

            match skill.execute(ctx, &args) {
                Ok(result) => {
                    carry = result.clone();
                    results.push(PipelineResult {
                        step: i,
                        skill: step.skill.clone(),
                        result,
                        success: true,
                    });
                }
                Err(e) => {
                    results.push(PipelineResult {
                        step: i,
                        skill: step.skill.clone(),
                        result: Value::Null,
                        success: false,
                    });
                    if step.required {
                        return Err(PipelineError {
                            step: i,
                            message: e.to_string(),
                        });
                    }
                }
            }
        }

        Ok(results)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineStep {
    pub skill: String,
    #[serde(default)]
    pub args: Value,
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PipelineResult {
    pub step: usize,
    pub skill: String,
    pub result: Value,
    pub success: bool,
}

#[derive(Debug)]
pub struct PipelineError {
    pub step: usize,
    pub message: String,
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PipelineError at step {}: {}", self.step, self.message)
    }
}

impl std::error::Error for PipelineError {}
