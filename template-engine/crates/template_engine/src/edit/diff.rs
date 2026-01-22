use serde::{Deserialize, Serialize};

use crate::ast::{Segment, Template};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TemplateDiff {
    pub wire_version: String,
    pub segment_changes: Vec<SegmentChange>,
    pub text_hunks: Vec<TextHunkDiff>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum SegmentChange {
    #[serde(rename = "insert_var")]
    InsertVar { at: usize, slug: String },

    #[serde(rename = "delete_var")]
    DeleteVar { at: usize, slug: String },

    #[serde(rename = "update_var")]
    UpdateVar { at: usize, slug: String, fields: Vec<String> },

    #[serde(rename = "replace_text")]
    ReplaceText {
        at: usize,
        before_len: usize,
        after_len: usize,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextHunkDiff {
    pub segment_index: usize,
    pub before: String,
    pub after: String,
    pub hunks: Vec<StringHunk>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StringHunk {
    pub op: String,
    pub text: String,
}

pub fn diff_templates(left: &Template, right: &Template) -> TemplateDiff {
    let mut diff = TemplateDiff {
        wire_version: "1.0".into(),
        ..Default::default()
    };

    let max_len = left.segments.len().max(right.segments.len());
    for index in 0..max_len {
        match (left.segments.get(index), right.segments.get(index)) {
            (Some(Segment::Var { slug: left_slug, .. }), Some(Segment::Var { slug: right_slug, .. }))
                if left_slug == right_slug => {}
            (Some(Segment::Var { slug, .. }), None) => diff
                .segment_changes
                .push(SegmentChange::DeleteVar {
                    at: index,
                    slug: slug.clone(),
                }),
            (None, Some(Segment::Var { slug, .. })) => diff
                .segment_changes
                .push(SegmentChange::InsertVar {
                    at: index,
                    slug: slug.clone(),
                }),
            (Some(Segment::Text { value: left_text }), Some(Segment::Text { value: right_text }))
                if left_text != right_text =>
            {
                diff.segment_changes.push(SegmentChange::ReplaceText {
                    at: index,
                    before_len: left_text.len(),
                    after_len: right_text.len(),
                });
                diff.text_hunks.push(TextHunkDiff {
                    segment_index: index,
                    before: left_text.clone(),
                    after: right_text.clone(),
                    hunks: vec![
                        StringHunk {
                            op: "delete".into(),
                            text: left_text.clone(),
                        },
                        StringHunk {
                            op: "insert".into(),
                            text: right_text.clone(),
                        },
                    ],
                });
            }
            _ => {}
        }
    }

    diff
}
