use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::EngineError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatchBundle {
    pub wire_version: String,
    pub left_hash: String,
    pub right_hash: String,
    pub forward: Patch,
    pub inverse: Patch,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Patch {
    #[serde(rename = "rfc6902")]
    Rfc6902 { ops: Vec<Rfc6902Op> },

    #[serde(rename = "fionn")]
    Fionn { patch: Value },

    #[serde(rename = "merge7396")]
    Merge7396 { patch: Value },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rfc6902Op {
    pub op: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

pub fn hash_json(value: &Value) -> String {
    format!(
        "blake3:{}",
        blake3::hash(value.to_string().as_bytes()).to_hex()
    )
}

pub fn make_patch_bundle_rfc6902(left: &Value, right: &Value) -> Result<PatchBundle, EngineError> {
    let forward_patch = json_patch::diff(left, right);
    let inverse_patch = json_patch::diff(right, left);

    let forward_ops = forward_patch
        .0
        .into_iter()
        .map(op_from_json_patch)
        .collect();
    let inverse_ops = inverse_patch
        .0
        .into_iter()
        .map(op_from_json_patch)
        .collect();

    Ok(PatchBundle {
        wire_version: "1.0".into(),
        left_hash: hash_json(left),
        right_hash: hash_json(right),
        forward: Patch::Rfc6902 { ops: forward_ops },
        inverse: Patch::Rfc6902 { ops: inverse_ops },
    })
}

fn op_from_json_patch(op: json_patch::PatchOperation) -> Rfc6902Op {
    use json_patch::PatchOperation::*;
    match op {
        Add(operation) => Rfc6902Op {
            op: "add".into(),
            path: operation.path.to_string(),
            from: None,
            value: Some(operation.value),
        },
        Remove(operation) => Rfc6902Op {
            op: "remove".into(),
            path: operation.path.to_string(),
            from: None,
            value: None,
        },
        Replace(operation) => Rfc6902Op {
            op: "replace".into(),
            path: operation.path.to_string(),
            from: None,
            value: Some(operation.value),
        },
        Move(operation) => Rfc6902Op {
            op: "move".into(),
            path: operation.path.to_string(),
            from: Some(operation.from.to_string()),
            value: None,
        },
        Copy(operation) => Rfc6902Op {
            op: "copy".into(),
            path: operation.path.to_string(),
            from: Some(operation.from.to_string()),
            value: None,
        },
        Test(operation) => Rfc6902Op {
            op: "test".into(),
            path: operation.path.to_string(),
            from: None,
            value: Some(operation.value),
        },
    }
}
