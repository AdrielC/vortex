use crate::ast::{Segment, Template};
use crate::edit::plan::{Edit, EditPlan, TextTarget, VarSpec};
use crate::error::EngineError;

pub fn apply_edits(base: &Template, plan: &EditPlan) -> Result<Template, EngineError> {
    let mut template = base.clone();

    let mut edits = plan.edits.clone();
    edits.sort_by(|a, b| sort_key(a).cmp(&sort_key(b)));

    for edit in edits {
        apply_one(&mut template, edit)?;
    }

    Ok(template)
}

fn sort_key(edit: &Edit) -> (usize, usize, u8) {
    match edit {
        Edit::ReplaceText { target, .. } => (target.segment_index, usize::MAX - target.start, 0),
        Edit::InsertVar { target, .. } => (target.segment_index, usize::MAX - target.start, 1),
        Edit::UpdateVar { segment_index, .. } => (*segment_index, 0, 2),
        Edit::DeleteVar { segment_index } => (*segment_index, 0, 3),
    }
}

fn apply_one(template: &mut Template, edit: Edit) -> Result<(), EngineError> {
    match edit {
        Edit::ReplaceText { target, value } => replace_text(template, target, value),
        Edit::InsertVar { target, var } => insert_var(template, target, var),
        Edit::DeleteVar { segment_index } => delete_var(template, segment_index),
        Edit::UpdateVar { segment_index, patch } => update_var(template, segment_index, patch),
    }
}

fn replace_text(template: &mut Template, target: TextTarget, value: String) -> Result<(), EngineError> {
    let segment = template
        .segments
        .get_mut(target.segment_index)
        .ok_or_else(|| EngineError::InvalidArgument("segment_index out of bounds".into()))?;

    let Segment::Text {
        value: ref mut text,
        span,
    } = segment
    else {
        return Err(EngineError::InvalidArgument(
            "ReplaceText target must be text segment".into(),
        ));
    };

    let char_count = text.chars().count();
    if target.start > target.end || target.end > char_count {
        return Err(EngineError::InvalidArgument("invalid text range".into()));
    }

    let start = char_to_byte_index(text, target.start)?;
    let end = char_to_byte_index(text, target.end)?;

    text.replace_range(start..end, &value);
    *span = None;
    Ok(())
}

fn insert_var(template: &mut Template, target: TextTarget, var: VarSpec) -> Result<(), EngineError> {
    let segment = template
        .segments
        .get_mut(target.segment_index)
        .ok_or_else(|| EngineError::InvalidArgument("segment_index out of bounds".into()))?;

    let Segment::Text {
        value: ref mut text,
        span,
    } = segment
    else {
        return Err(EngineError::InvalidArgument(
            "InsertVar target must be text segment".into(),
        ));
    };

    let char_count = text.chars().count();
    if target.start > char_count {
        return Err(EngineError::InvalidArgument("insert offset out of bounds".into()));
    }

    let start = char_to_byte_index(text, target.start)?;

    let right = text.split_off(start);
    let left = std::mem::take(text);
    *span = None;

    template.segments[target.segment_index] = Segment::Text {
        value: left,
        span: None,
    };
    template.segments.insert(
        target.segment_index + 1,
        Segment::Var {
            slug: var.slug,
            kind: var.kind,
            source: var.source,
            schema: var.schema,
            value: None,
            span: None,
        },
    );
    template.segments.insert(
        target.segment_index + 2,
        Segment::Text {
            value: right,
            span: None,
        },
    );

    Ok(())
}

fn delete_var(template: &mut Template, segment_index: usize) -> Result<(), EngineError> {
    let segment = template
        .segments
        .get(segment_index)
        .ok_or_else(|| EngineError::InvalidArgument("segment_index out of bounds".into()))?;

    match segment {
        Segment::Var { .. } => {
            template.segments.remove(segment_index);
            Ok(())
        }
        _ => Err(EngineError::InvalidArgument(
            "DeleteVar target must be var segment".into(),
        )),
    }
}

fn update_var(
    template: &mut Template,
    segment_index: usize,
    patch: crate::edit::plan::VarPatch,
) -> Result<(), EngineError> {
    let segment = template
        .segments
        .get_mut(segment_index)
        .ok_or_else(|| EngineError::InvalidArgument("segment_index out of bounds".into()))?;

    let Segment::Var {
        kind,
        source,
        schema,
        value,
        ..
    } = segment
    else {
        return Err(EngineError::InvalidArgument(
            "UpdateVar target must be var segment".into(),
        ));
    };

    if let Some(new_kind) = patch.kind {
        *kind = new_kind;
    }
    if let Some(new_source) = patch.source {
        *source = new_source;
    }
    if let Some(new_schema) = patch.schema {
        *schema = new_schema;
    }
    if let Some(new_value) = patch.value {
        *value = new_value;
    }

    Ok(())
}

fn char_to_byte_index(text: &str, char_index: usize) -> Result<usize, EngineError> {
    if char_index == text.chars().count() {
        return Ok(text.len());
    }

    text.char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
        .ok_or_else(|| EngineError::InvalidArgument("char index out of bounds".into()))
}
