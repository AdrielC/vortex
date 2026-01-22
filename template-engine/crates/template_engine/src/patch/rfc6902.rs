use serde_json::Value;

use crate::error::EngineError;
use crate::patch::{PatchEngine, Rfc6902Op};

#[derive(Default)]
pub struct Rfc6902Engine;

impl PatchEngine for Rfc6902Engine {
    fn diff(&self, left: &Value, right: &Value) -> Result<Vec<Rfc6902Op>, EngineError> {
        let patch = json_patch::diff(left, right);
        Ok(patch.0.into_iter().map(op_from_json_patch).collect())
    }

    fn apply(&self, doc: &Value, ops: &[Rfc6902Op]) -> Result<Value, EngineError> {
        apply_ops(doc, ops)
    }
}

pub fn apply_ops(doc: &Value, ops: &[Rfc6902Op]) -> Result<Value, EngineError> {
    let mut value = doc.clone();
    let patch = json_patch::Patch(ops.iter().map(op_to_json_patch).collect());
    json_patch::patch(&mut value, &patch)
        .map_err(|error| EngineError::InvalidArgument(error.to_string()))?;
    Ok(value)
}

pub fn op_from_json_patch(op: json_patch::PatchOperation) -> Rfc6902Op {
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

pub fn op_to_json_patch(op: &Rfc6902Op) -> json_patch::PatchOperation {
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
