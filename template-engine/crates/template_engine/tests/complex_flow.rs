use std::collections::BTreeMap;

use serde_json::{json, Value};
use template_engine::{
    apply_edit_plan, apply_patch_bundle_forward, apply_patch_bundle_inverse, apply_template_edits_to_text,
    hash_json, BoundStatus, Edit, EditPlan, RenderMode, Segment, Span, Template, TextTarget, VarSpec,
};

#[test]
fn edit_plan_patch_bundle_round_trip() {
    let base = Template {
        wire_version: "1.0".into(),
        segments: vec![
            Segment::Text {
                value: "Hello ".into(),
                span: None,
            },
            Segment::Var {
                slug: "name".into(),
                kind: template_engine::VarKind::Runtime,
                source: None,
                schema: None,
                value: None,
                span: None,
            },
            Segment::Text {
                value: "!".into(),
                span: None,
            },
        ],
    };

    let plan = EditPlan {
        wire_version: "1.0".into(),
        fork_id: "agent:proposal:1".into(),
        edits: vec![
            Edit::ReplaceText {
                target: TextTarget {
                    segment_index: 0,
                    start: 6,
                    end: 6,
                },
                value: "dear ".into(),
            },
            Edit::InsertVar {
                target: TextTarget {
                    segment_index: 2,
                    start: 1,
                    end: 1,
                },
                var: VarSpec {
                    slug: "emoji".into(),
                    kind: template_engine::VarKind::Runtime,
                    source: None,
                    schema: None,
                },
            },
        ],
        suggestions: Vec::new(),
    };

    let result = apply_edit_plan(&base, &plan).expect("apply edit plan");
    let left: Value = serde_json::to_value(&base).expect("left json");
    let right: Value = serde_json::to_value(&result.new_template).expect("right json");

    let forward = apply_patch_bundle_forward(
        &left,
        &result.patch_bundle,
        template_engine::ApplyOptions {
            enforce_left_hash: true,
            enforce_right_hash: false,
        },
    )
    .expect("forward patch");
    assert_eq!(forward, right);

    let inverse = apply_patch_bundle_inverse(
        &right,
        &result.patch_bundle,
        template_engine::ApplyOptions {
            enforce_left_hash: false,
            enforce_right_hash: true,
        },
    )
    .expect("inverse patch");
    assert_eq!(inverse, left);
    assert_eq!(result.patch_bundle.left_hash, hash_json(&left));
    assert_eq!(result.patch_bundle.right_hash, hash_json(&right));
}

#[test]
fn compiled_binding_overlay_precedence() {
    let template = Template {
        wire_version: "1.0".into(),
        segments: vec![
            Segment::Text {
                value: "Fee: ".into(),
                span: None,
            },
            Segment::Var {
                slug: "fee".into(),
                kind: template_engine::VarKind::Runtime,
                source: None,
                schema: None,
                value: None,
                span: None,
            },
        ],
    };

    let compiled = template_engine::compile_from_template(&template).expect("compile");

    let mut env = BTreeMap::new();
    env.insert("fee".to_string(), json!("100"));

    let mut overlay = BTreeMap::new();
    overlay.insert("fee".to_string(), json!("95"));

    let binding = template_engine::bind_compiled(&compiled, &env, Some(&overlay)).expect("bind");
    let trace = template_engine::render_compiled(&compiled, &binding, RenderMode::Resolved)
        .expect("render");

    assert_eq!(trace.rendered, "Fee: 95");

    let bound = binding.values.values().next().expect("bound value");
    assert!(matches!(bound.status, BoundStatus::Resolved));
    assert_eq!(bound.raw.as_ref().unwrap(), &json!("95"));
}

#[test]
fn text_apply_rejects_overlap() {
    let original = "a:b";
    let base = Template {
        wire_version: "1.0".into(),
        segments: vec![
            Segment::Text {
                value: "a".into(),
                span: Some(Span {
                    start_byte: 0,
                    end_byte: 2,
                }),
            },
            Segment::Var {
                slug: "b".into(),
                kind: template_engine::VarKind::Runtime,
                source: None,
                schema: None,
                value: None,
                span: Some(Span {
                    start_byte: 1,
                    end_byte: 3,
                }),
            },
        ],
    };

    let updated = base.clone();
    let error = apply_template_edits_to_text(original, &base, &updated).expect_err("overlap");
    assert!(error.to_string().contains("overlapping spans"));
}
