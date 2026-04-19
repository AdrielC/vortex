use serde_json::Value;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile_template_json(template_json: &str) -> Result<String, JsValue> {
    let template: template_engine::Template =
        serde_json::from_str(template_json).map_err(|err| JsValue::from_str(&err.to_string()))?;
    let compiled = template_engine::compile_from_template(&template)
        .map_err(|err| JsValue::from_str(&err.to_string()))?;
    serde_json::to_string(&compiled).map_err(|err| JsValue::from_str(&err.to_string()))
}

#[wasm_bindgen]
pub fn bind_compiled_json(
    compiled_json: &str,
    env_json: &str,
    overlay_json: Option<String>,
) -> Result<String, JsValue> {
    let compiled: template_engine::CompiledTemplate =
        serde_json::from_str(compiled_json).map_err(|err| JsValue::from_str(&err.to_string()))?;
    let env: std::collections::BTreeMap<String, Value> =
        serde_json::from_str(env_json).map_err(|err| JsValue::from_str(&err.to_string()))?;
    let overlay: Option<std::collections::BTreeMap<String, Value>> = match overlay_json {
        Some(json) => Some(serde_json::from_str(&json).map_err(|err| JsValue::from_str(&err.to_string()))?),
        None => None,
    };

    let binding = template_engine::bind_compiled(&compiled, &env, overlay.as_ref())
        .map_err(|err| JsValue::from_str(&err.to_string()))?;
    serde_json::to_string(&binding).map_err(|err| JsValue::from_str(&err.to_string()))
}

#[wasm_bindgen]
pub fn render_compiled_json(
    compiled_json: &str,
    binding_json: &str,
    mode: &str,
) -> Result<String, JsValue> {
    let compiled: template_engine::CompiledTemplate =
        serde_json::from_str(compiled_json).map_err(|err| JsValue::from_str(&err.to_string()))?;
    let binding: template_engine::CompiledBinding =
        serde_json::from_str(binding_json).map_err(|err| JsValue::from_str(&err.to_string()))?;
    let render_mode = match mode {
        "template" => template_engine::RenderMode::Template,
        "resolved" => template_engine::RenderMode::Resolved,
        "annotated" => template_engine::RenderMode::Annotated,
        _ => return Err(JsValue::from_str("invalid render mode")),
    };

    let trace = template_engine::render_compiled(&compiled, &binding, render_mode)
        .map_err(|err| JsValue::from_str(&err.to_string()))?;
    serde_json::to_string(&trace).map_err(|err| JsValue::from_str(&err.to_string()))
}
