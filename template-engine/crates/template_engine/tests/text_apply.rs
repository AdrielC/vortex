use template_engine::{apply_template_edits_to_text, Segment, Span, Template};

#[test]
fn apply_template_edits_preserves_unrelated_text() {
    let original = "greeting: :name:\nnotes: keep";
    let base = Template {
        wire_version: "1.0".into(),
        segments: vec![
            Segment::Text {
                value: "greeting: ".into(),
                span: Some(Span {
                    start_byte: 0,
                    end_byte: 10,
                }),
            },
            Segment::Var {
                slug: "name".into(),
                kind: template_engine::VarKind::Runtime,
                source: None,
                schema: None,
                value: None,
                span: Some(Span {
                    start_byte: 10,
                    end_byte: 16,
                }),
            },
            Segment::Text {
                value: "\nnotes: keep".into(),
                span: Some(Span {
                    start_byte: 16,
                    end_byte: 28,
                }),
            },
        ],
    };

    let updated = Template {
        wire_version: "1.0".into(),
        segments: vec![
            Segment::Text {
                value: "greeting: ".into(),
                span: None,
            },
            Segment::Var {
                slug: "first_name".into(),
                kind: template_engine::VarKind::Runtime,
                source: None,
                schema: None,
                value: None,
                span: None,
            },
            Segment::Text {
                value: "\nnotes: keep".into(),
                span: None,
            },
        ],
    };

    let patched = apply_template_edits_to_text(original, &base, &updated).expect("patched");
    assert_eq!(patched, "greeting: :first_name:\nnotes: keep");
}
