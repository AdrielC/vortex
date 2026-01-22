use crate::ast::{Segment, Span};
use crate::error::EngineError;

#[derive(Clone, Debug)]
pub struct SpanView {
    pub index: usize,
    pub span: Span,
}

pub fn validate_spans(segments: &[Segment]) -> Result<Vec<SpanView>, EngineError> {
    let mut views = Vec::with_capacity(segments.len());
    let mut last_end: u32 = 0;

    for (index, segment) in segments.iter().enumerate() {
        let span = match segment {
            Segment::Text { span: Some(span), .. } => span,
            Segment::Var { span: Some(span), .. } => span,
            _ => {
                return Err(EngineError::InvalidArgument(
                    "missing span for segment".into(),
                ))
            }
        };

        if span.start_byte > span.end_byte {
            return Err(EngineError::InvalidArgument("invalid span order".into()));
        }
        if span.start_byte < last_end {
            return Err(EngineError::InvalidArgument(
                "overlapping spans are not supported".into(),
            ));
        }

        last_end = span.end_byte;
        views.push(SpanView {
            index,
            span: span.clone(),
        });
    }

    Ok(views)
}
