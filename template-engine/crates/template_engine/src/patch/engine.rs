use serde_json::Value;

use crate::error::EngineError;
use crate::patch::Rfc6902Op;

pub trait PatchEngine {
    fn diff(&self, left: &Value, right: &Value) -> Result<Vec<Rfc6902Op>, EngineError>;
    fn apply(&self, doc: &Value, ops: &[Rfc6902Op]) -> Result<Value, EngineError>;
}
