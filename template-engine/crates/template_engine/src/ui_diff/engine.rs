use serde_json::Value;

use crate::error::EngineError;
use crate::ui_diff::UiDiff;

pub trait UiDiffEngine {
    fn diff(&self, left: &Value, right: &Value) -> Result<UiDiff, EngineError>;
}
