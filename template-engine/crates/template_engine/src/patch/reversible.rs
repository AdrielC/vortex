use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::EngineError;
use crate::patch::rfc6902::{apply_ops, op_from_json_patch};

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

#[derive(Clone, Debug)]
pub struct ApplyOptions {
    pub enforce_left_hash: bool,
    pub enforce_right_hash: bool,
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

pub fn apply_patch_bundle_forward(
    doc: &Value,
    bundle: &PatchBundle,
    opts: ApplyOptions,
) -> Result<Value, EngineError> {
    if opts.enforce_left_hash && hash_json(doc) != bundle.left_hash {
        return Err(EngineError::InvalidArgument("left_hash mismatch".into()));
    }
    apply_patch(doc, &bundle.forward)
}

pub fn apply_patch_bundle_inverse(
    doc: &Value,
    bundle: &PatchBundle,
    opts: ApplyOptions,
) -> Result<Value, EngineError> {
    if opts.enforce_right_hash && hash_json(doc) != bundle.right_hash {
        return Err(EngineError::InvalidArgument("right_hash mismatch".into()));
    }
    apply_patch(doc, &bundle.inverse)
}

pub fn apply_patch(doc: &Value, patch: &Patch) -> Result<Value, EngineError> {
    match patch {
        Patch::Rfc6902 { ops } => apply_ops(doc, ops),
        Patch::Merge7396 { patch } => {
            let mut value = doc.clone();
            json_patch::merge(&mut value, patch);
            Ok(value)
        }
    }
}
