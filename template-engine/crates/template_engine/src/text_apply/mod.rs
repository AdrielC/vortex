mod patch_text;
mod spans;

pub use patch_text::apply_template_edits_to_text;
pub use spans::{validate_spans, SpanView};
