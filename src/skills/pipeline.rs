// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use serde_json::Value;

use super::registry::SkillRegistry;
use super::types::SkillContext;

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
            let skill = self
                .registry
                .get(&step.skill)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::builtin;
    use crate::skills::registry::SkillRegistry;
    use crate::skills::types::SkillContext;
    use crate::universe::crystal::CrystalEngine;
    use crate::universe::hebbian::HebbianMemory;
    use crate::universe::memory::MemoryAtom;
    use crate::universe::node::DarkUniverse;
    use serde_json::json;

    fn make_context() -> SkillContext<'static> {
        let u = Box::new(DarkUniverse::new(10000.0));
        let h = Box::new(HebbianMemory::new());
        let m = Box::new(Vec::<MemoryAtom>::new());
        let c = Box::new(CrystalEngine::new());
        SkillContext {
            universe: Box::leak(u),
            hebbian: Box::leak(h),
            memories: Box::leak(m),
            crystal: Box::leak(c),
        }
    }

    fn make_pipeline() -> SkillPipeline {
        let mut reg = SkillRegistry::new();
        builtin::register_all(&mut reg);
        SkillPipeline::new(reg)
    }

    #[test]
    fn pipeline_step_deserialize() {
        let step: PipelineStep = serde_json::from_str(
            r#"{"skill": "check_conservation", "args": null, "required": false}"#,
        )
        .unwrap();
        assert_eq!(step.skill, "check_conservation");
        assert!(!step.required);
    }

    #[test]
    fn pipeline_step_default_required_is_true() {
        let step: PipelineStep =
            serde_json::from_str(r#"{"skill": "check_conservation"}"#).unwrap();
        assert!(step.required);
    }

    #[test]
    fn execute_single_check_conservation() {
        let pipeline = make_pipeline();
        let mut ctx = make_context();
        let steps = vec![PipelineStep {
            skill: "check_conservation".into(),
            args: json!({}),
            required: true,
        }];
        let results = pipeline.execute_chain(&steps, &mut ctx).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }

    #[test]
    fn execute_chain_encode_then_check() {
        let pipeline = make_pipeline();
        let mut ctx = make_context();
        let steps = vec![
            PipelineStep {
                skill: "encode_memory".into(),
                args: json!({"anchor": [1, 2, 3], "data": [1.0, 2.0]}),
                required: true,
            },
            PipelineStep {
                skill: "check_conservation".into(),
                args: json!({}),
                required: true,
            },
        ];
        let results = pipeline.execute_chain(&steps, &mut ctx).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
    }

    #[test]
    fn execute_chain_carry_forward() {
        let pipeline = make_pipeline();
        let mut ctx = make_context();
        let steps = vec![
            PipelineStep {
                skill: "check_conservation".into(),
                args: json!({}),
                required: true,
            },
            PipelineStep {
                skill: "check_conservation".into(),
                args: Value::Null,
                required: true,
            },
        ];
        let results = pipeline.execute_chain(&steps, &mut ctx).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn execute_chain_unknown_skill_fails() {
        let pipeline = make_pipeline();
        let mut ctx = make_context();
        let steps = vec![PipelineStep {
            skill: "nonexistent".into(),
            args: json!({}),
            required: true,
        }];
        let err = pipeline.execute_chain(&steps, &mut ctx).unwrap_err();
        assert_eq!(err.step, 0);
    }

    #[test]
    fn execute_chain_optional_failure_continues() {
        let pipeline = make_pipeline();
        let mut ctx = make_context();
        let steps = vec![
            PipelineStep {
                skill: "encode_memory".into(),
                args: json!({"anchor": [1, 2, 3], "data": [99.0]}),
                required: false,
            },
            PipelineStep {
                skill: "check_conservation".into(),
                args: json!({}),
                required: true,
            },
        ];
        let results = pipeline.execute_chain(&steps, &mut ctx).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[1].success);
    }

    #[test]
    fn pipeline_error_display() {
        let err = PipelineError {
            step: 3,
            message: "boom".into(),
        };
        assert_eq!(format!("{}", err), "PipelineError at step 3: boom");
    }

    #[test]
    fn pipeline_result_serde() {
        let pr = PipelineResult {
            step: 0,
            skill: "test".into(),
            result: json!(true),
            success: true,
        };
        let s = serde_json::to_string(&pr).unwrap();
        assert!(s.contains("\"success\":true"));
    }
}
