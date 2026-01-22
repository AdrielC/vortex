mod ast;
mod edit;
mod error;

pub use ast::{Segment, Source, Template, VarKind};
pub use edit::{apply_edit_plan, diff_templates, ApplyEditPlanResult, EditPlan};
pub use error::EngineError;
