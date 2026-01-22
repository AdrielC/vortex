use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::ast::{Segment, Template};
use crate::error::EngineError;
use crate::patch::hash_json;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledTemplate {
    pub wire_version: String,
    pub template_hash: String,
    pub items: Vec<Item>,
    pub stats: CompileStats,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Item {
    #[serde(rename = "text")]
    Text {
        text: String,
        item_id: u32,
    },

    #[serde(rename = "var")]
    Var {
        slug: String,
        item_id: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        var_ref: Option<VarRefLite>,
        #[serde(skip_serializing_if = "Option::is_none")]
        schema: Option<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompileStats {
    pub text_items: u32,
    pub var_items: u32,
    pub total_text_bytes: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VarRefLite {
    pub ns: String,
    pub key: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_hint: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledBinding {
    pub wire_version: String,
    pub template_hash: String,
    pub values: BTreeMap<u32, BoundVar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundVar {
    pub status: BoundStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundStatus {
    Resolved,
    MissingValue,
    TypeError,
    CodecError,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderTrace {
    pub rendered: String,
    pub spans: Vec<RenderSpan>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RenderSpan {
    #[serde(rename = "text")]
    Text { range: [u32; 2], item_id: u32 },

    #[serde(rename = "var")]
    Var {
        range: [u32; 2],
        item_id: u32,
        slug: String,
        status: BoundStatus,
        #[serde(skip_serializing_if = "Option::is_none")]
        env_path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        formatted: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    Template,
    Resolved,
    Annotated,
}

pub fn compile_from_template(template: &Template) -> Result<CompiledTemplate, EngineError> {
    let mut items = Vec::with_capacity(template.segments.len());
    let mut text_items = 0u32;
    let mut var_items = 0u32;
    let mut total_text_bytes = 0u32;

    for (index, segment) in template.segments.iter().enumerate() {
        let item_id = stable_item_id(index as u32, segment);
        match segment {
            Segment::Text { value, .. } => {
                text_items += 1;
                total_text_bytes += value.len() as u32;
                items.push(Item::Text {
                    text: value.clone(),
                    item_id,
                });
            }
            Segment::Var { slug, schema, .. } => {
                var_items += 1;
                items.push(Item::Var {
                    slug: slug.clone(),
                    item_id,
                    var_ref: None,
                    schema: schema.clone(),
                });
            }
        }
    }

    let template_hash = hash_items(&items)?;

    Ok(CompiledTemplate {
        wire_version: "1.0".into(),
        template_hash,
        items,
        stats: CompileStats {
            text_items,
            var_items,
            total_text_bytes,
        },
    })
}

pub fn bind_compiled(
    compiled: &CompiledTemplate,
    env: &BTreeMap<String, Value>,
    overlay: Option<&BTreeMap<String, Value>>,
) -> Result<CompiledBinding, EngineError> {
    let mut values = BTreeMap::new();

    for item in &compiled.items {
        if let Item::Var { slug, item_id, .. } = item {
            let (raw, status, formatted) = match overlay.and_then(|map| map.get(slug)) {
                Some(value) => (Some(value.clone()), BoundStatus::Resolved, format_value(value)),
                None => match env.get(slug) {
                    Some(value) => (Some(value.clone()), BoundStatus::Resolved, format_value(value)),
                    None => (None, BoundStatus::MissingValue, None),
                },
            };

            values.insert(
                *item_id,
                BoundVar {
                    status,
                    raw,
                    formatted,
                    codec: None,
                    env_path: Some(slug.clone()),
                },
            );
        }
    }

    Ok(CompiledBinding {
        wire_version: "1.0".into(),
        template_hash: compiled.template_hash.clone(),
        values,
        report: None,
    })
}

pub fn render_compiled(
    compiled: &CompiledTemplate,
    binding: &CompiledBinding,
    mode: RenderMode,
) -> Result<RenderTrace, EngineError> {
    if compiled.template_hash != binding.template_hash {
        return Err(EngineError::InvalidArgument(
            "template_hash mismatch".into(),
        ));
    }

    let mut rendered = String::new();
    let mut spans = Vec::with_capacity(compiled.items.len());

    for item in &compiled.items {
        match item {
            Item::Text { text, item_id } => {
                let start = rendered.len() as u32;
                rendered.push_str(text);
                let end = rendered.len() as u32;
                spans.push(RenderSpan::Text {
                    range: [start, end],
                    item_id: *item_id,
                });
            }
            Item::Var { slug, item_id, .. } => {
                let start = rendered.len() as u32;
                let bound = binding.values.get(item_id);
                let rendered_piece = match (mode, bound) {
                    (RenderMode::Template, _) => format!(":{slug}:"),
                    (RenderMode::Resolved, Some(value)) => match value.status {
                        BoundStatus::Resolved => value.formatted.clone().unwrap_or_default(),
                        _ => "".into(),
                    },
                    (RenderMode::Annotated, Some(value)) => match value.status {
                        BoundStatus::Resolved => {
                            format!("⟦{}⟧", value.formatted.clone().unwrap_or_default())
                        }
                        _ => format!("⟦UNRESOLVED:{slug}⟧"),
                    },
                    (_, None) => match mode {
                        RenderMode::Template => format!(":{slug}:"),
                        RenderMode::Resolved => "".into(),
                        RenderMode::Annotated => format!("⟦UNBOUND:{slug}⟧"),
                    },
                };

                rendered.push_str(&rendered_piece);
                let end = rendered.len() as u32;

                let (status, env_path, formatted) = bound
                    .map(|value| {
                        (
                            value.status.clone(),
                            value.env_path.clone(),
                            value.formatted.clone(),
                        )
                    })
                    .unwrap_or((BoundStatus::MissingValue, None, None));

                spans.push(RenderSpan::Var {
                    range: [start, end],
                    item_id: *item_id,
                    slug: slug.clone(),
                    status,
                    env_path,
                    formatted,
                });
            }
        }
    }

    Ok(RenderTrace { rendered, spans })
}

fn stable_item_id(position: u32, segment: &Segment) -> u32 {
    let tag = match segment {
        Segment::Text { .. } => 0u32,
        Segment::Var { .. } => 1u32,
    };

    position.wrapping_mul(0x9E37_79B1).wrapping_add(tag)
}

fn hash_items(items: &[Item]) -> Result<String, EngineError> {
    let json = serde_json::to_value(items)?;
    Ok(hash_json(&json))
}

fn format_value(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        _ => Some(value.to_string()),
    }
}
