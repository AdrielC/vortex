mod patch_text;
mod spans;

pub use patch_text::apply_template_edits_to_text;
#[allow(unused_imports)]
pub use spans::{validate_spans, SpanView};
