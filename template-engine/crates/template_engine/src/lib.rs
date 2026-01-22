mod ast;
mod edit;
mod error;
mod patch;
mod compiled;
mod semantic_diff;
mod text_apply;
mod ui_diff;

pub use ast::{Segment, Source, Span, Template, VarKind};
pub use compiled::{
    bind_compiled, compile_from_template, render_compiled, BoundStatus, BoundVar, CompiledBinding,
    CompiledTemplate, CompileStats, Item, RenderMode, RenderSpan, RenderTrace, VarRefLite,
};
pub use edit::{apply_edit_plan, ApplyEditPlanResult, EditPlan};
pub use error::EngineError;
pub use patch::{
    apply_patch, apply_patch_bundle_forward, apply_patch_bundle_inverse, hash_json,
    make_patch_bundle_rfc6902, ApplyOptions, Patch, PatchBundle, PatchEngine, Rfc6902Engine,
    Rfc6902Op,
};
pub use semantic_diff::{diff_templates_from_patch, SegmentChange, StringHunk, TemplateDiff, TextHunkDiff};
pub use text_apply::{apply_template_edits_to_text, SpanView};
pub use ui_diff::{SimpleUiDiffEngine, UiDiff, UiDiffEngine};
