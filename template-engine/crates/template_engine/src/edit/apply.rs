use serde_json::Value;

use crate::edit::patch::{hash_json, Patch, PatchBundle, Rfc6902Op};
use crate::error::EngineError;

#[derive(Clone, Debug)]
pub struct ApplyOptions {
    pub enforce_left_hash: bool,
    pub enforce_right_hash: bool,
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
        Patch::Rfc6902 { ops } => {
            let mut value = doc.clone();
            let patch = json_patch::Patch(ops.iter().map(op_to_json_patch).collect());
            json_patch::patch(&mut value, &patch)
                .map_err(|error| EngineError::InvalidArgument(error.to_string()))?;
            Ok(value)
        }
        Patch::Merge7396 { patch } => {
            let mut value = doc.clone();
            json_patch::merge(&mut value, patch);
            Ok(value)
        }
        Patch::Fionn { .. } => Err(EngineError::InvalidArgument(
            "fionn patch apply not implemented yet".into(),
        )),
    }
}

fn op_to_json_patch(op: &Rfc6902Op) -> json_patch::PatchOperation {
    use json_patch::*;

    match op.op.as_str() {
        "add" => PatchOperation::Add(AddOperation {
            path: op.path.parse().unwrap(),
            value: op.value.clone().unwrap(),
        }),
        "remove" => PatchOperation::Remove(RemoveOperation {
            path: op.path.parse().unwrap(),
        }),
        "replace" => PatchOperation::Replace(ReplaceOperation {
            path: op.path.parse().unwrap(),
            value: op.value.clone().unwrap(),
        }),
        "move" => PatchOperation::Move(MoveOperation {
            from: op.from.clone().unwrap().parse().unwrap(),
            path: op.path.parse().unwrap(),
        }),
        "copy" => PatchOperation::Copy(CopyOperation {
            from: op.from.clone().unwrap().parse().unwrap(),
            path: op.path.parse().unwrap(),
        }),
        "test" => PatchOperation::Test(TestOperation {
            path: op.path.parse().unwrap(),
            value: op.value.clone().unwrap(),
        }),
        other => panic!("unsupported op {other}"),
    }
}
