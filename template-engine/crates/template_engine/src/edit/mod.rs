mod compile;
mod plan;

pub use compile::apply_edits;
pub use plan::{Edit, EditPlan, Suggestion, SuggestionStatus, TextTarget, VarPatch, VarSpec};

use crate::ast::Template;
use crate::error::EngineError;
use crate::patch::{make_patch_bundle_rfc6902, Patch, PatchBundle, Rfc6902Op};
use crate::semantic_diff::{diff_templates_from_patch, TemplateDiff};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApplyEditPlanResult {
    pub wire_version: String,
    pub new_template: Template,
    pub patch_bundle: PatchBundle,
    pub diff: TemplateDiff,
    pub suggestions: Vec<Suggestion>,
}

pub fn apply_edit_plan(base: &Template, plan: &EditPlan) -> Result<ApplyEditPlanResult, EngineError> {
    let new_template = compile::apply_edits(base, plan)?;

    let left: Value = serde_json::to_value(base)?;
    let right: Value = serde_json::to_value(&new_template)?;
    let bundle = make_patch_bundle_rfc6902(&left, &right)?;

    let ops = match &bundle.forward {
        Patch::Rfc6902 { ops } => ops.as_slice(),
        Patch::Merge7396 { .. } => &[] as &[Rfc6902Op],
    };
    let diff = diff_templates_from_patch(base, &new_template, ops);

    Ok(ApplyEditPlanResult {
        wire_version: "1.0".into(),
        new_template,
        patch_bundle: bundle,
        diff,
        suggestions: plan.suggestions.clone(),
    })
}
