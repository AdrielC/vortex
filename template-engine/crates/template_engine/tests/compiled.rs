use std::collections::BTreeMap;

use serde_json::json;
use template_engine::{
    bind_compiled, compile_from_template, render_compiled, BoundStatus, RenderMode, Segment,
    Template,
};

#[test]
fn compile_bind_render_round_trip() {
    let template = Template {
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
        ],
    };

    let compiled = compile_from_template(&template).expect("compile");

    let mut env = BTreeMap::new();
    env.insert("name".to_string(), json!("Ada"));
    let binding = bind_compiled(&compiled, &env, None).expect("bind");

    let trace = render_compiled(&compiled, &binding, RenderMode::Resolved).expect("render");
    assert_eq!(trace.rendered, "Hello Ada");
    assert_eq!(trace.spans.len(), compiled.items.len());
}

#[test]
fn render_missing_value_is_empty_in_resolved_mode() {
    let template = Template {
        wire_version: "1.0".into(),
        segments: vec![
            Segment::Text {
                value: "Hi ".into(),
                span: None,
            },
            Segment::Var {
                slug: "missing".into(),
                kind: template_engine::VarKind::Runtime,
                source: None,
                schema: None,
                value: None,
                span: None,
            },
        ],
    };

    let compiled = compile_from_template(&template).expect("compile");
    let env = BTreeMap::new();
    let binding = bind_compiled(&compiled, &env, None).expect("bind");

    let trace = render_compiled(&compiled, &binding, RenderMode::Resolved).expect("render");
    assert_eq!(trace.rendered, "Hi ");

    let (_, bound) = binding.values.iter().next().expect("bound var");
    assert!(matches!(bound.status, BoundStatus::MissingValue));
}
