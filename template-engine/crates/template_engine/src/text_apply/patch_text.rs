use crate::ast::{Segment, Template};
use crate::error::EngineError;
use crate::text_apply::spans::validate_spans;

pub fn apply_template_edits_to_text(
    original: &str,
    base: &Template,
    updated: &Template,
) -> Result<String, EngineError> {
    if base.segments.len() != updated.segments.len() {
        return Err(EngineError::InvalidArgument(
            "segment count mismatch; cannot apply span patch".into(),
        ));
    }

    let spans = validate_spans(&base.segments)?;
    let mut result = original.to_string();

    for view in spans.into_iter().rev() {
        let base_segment = &base.segments[view.index];
        let updated_segment = &updated.segments[view.index];

        if base_segment == updated_segment {
            continue;
        }

        let replacement = render_segment(updated_segment);
        let start = view.span.start_byte as usize;
        let end = view.span.end_byte as usize;

        if end > result.len() {
            return Err(EngineError::InvalidArgument(
                "span extends beyond original text".into(),
            ));
        }

        result.replace_range(start..end, &replacement);
    }

    Ok(result)
}

fn render_segment(segment: &Segment) -> String {
    match segment {
        Segment::Text { value, .. } => value.clone(),
        Segment::Var { slug, .. } => format!(":{slug}:"),
    }
}
