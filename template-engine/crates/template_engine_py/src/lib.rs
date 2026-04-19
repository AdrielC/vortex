use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde_json::Value;

type PyResult<T> = Result<T, PyErr>;

#[pyfunction]
fn compile_template_json(template_json: &str) -> PyResult<String> {
    let template: template_engine::Template = serde_json::from_str(template_json)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    let compiled = template_engine::compile_from_template(&template)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    serde_json::to_string(&compiled).map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pyfunction]
#[pyo3(signature = (compiled_json, env_json, overlay_json=None))]
fn bind_compiled_json(compiled_json: &str, env_json: &str, overlay_json: Option<&str>) -> PyResult<String> {
    let compiled: template_engine::CompiledTemplate = serde_json::from_str(compiled_json)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    let env: std::collections::BTreeMap<String, Value> = serde_json::from_str(env_json)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    let overlay: Option<std::collections::BTreeMap<String, Value>> = match overlay_json {
        Some(json) => Some(
            serde_json::from_str(json).map_err(|err| PyValueError::new_err(err.to_string()))?,
        ),
        None => None,
    };

    let binding = template_engine::bind_compiled(&compiled, &env, overlay.as_ref())
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    serde_json::to_string(&binding).map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pyfunction]
fn render_compiled_json(compiled_json: &str, binding_json: &str, mode: &str) -> PyResult<String> {
    let compiled: template_engine::CompiledTemplate = serde_json::from_str(compiled_json)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    let binding: template_engine::CompiledBinding = serde_json::from_str(binding_json)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    let render_mode = match mode {
        "template" => template_engine::RenderMode::Template,
        "resolved" => template_engine::RenderMode::Resolved,
        "annotated" => template_engine::RenderMode::Annotated,
        _ => return Err(PyValueError::new_err("invalid render mode")),
    };

    let trace = template_engine::render_compiled(&compiled, &binding, render_mode)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    serde_json::to_string(&trace).map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pymodule]
fn template_engine_py(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(compile_template_json, module)?)?;
    module.add_function(wrap_pyfunction!(bind_compiled_json, module)?)?;
    module.add_function(wrap_pyfunction!(render_compiled_json, module)?)?;
    Ok(())
}
