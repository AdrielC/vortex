mod apply;
mod compile;
mod diff;
mod patch;
mod plan;

pub use apply::{apply_patch, apply_patch_bundle_forward, apply_patch_bundle_inverse, ApplyOptions};
pub use compile::apply_edits;
pub use diff::{diff_templates, SegmentChange, StringHunk, TemplateDiff, TextHunkDiff};
pub use patch::{make_patch_bundle_rfc6902, Patch, PatchBundle, Rfc6902Op};
pub use plan::{Edit, EditPlan, Suggestion, SuggestionStatus, TextTarget, VarPatch, VarSpec};

use crate::ast::Template;
use crate::error::EngineError;
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

    let diff = diff::diff_templates(base, &new_template);

    let left: Value = serde_json::to_value(base)?;
    let right: Value = serde_json::to_value(&new_template)?;
    let bundle = patch::make_patch_bundle_rfc6902(&left, &right)?;

    Ok(ApplyEditPlanResult {
        wire_version: "1.0".into(),
        new_template,
        patch_bundle: bundle,
        diff,
        suggestions: plan.suggestions.clone(),
    })
}
