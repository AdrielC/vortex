use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::EngineError;
use crate::ui_diff::UiDiffEngine;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UiDiff {
    pub wire_version: String,
    pub payload: Value,
}

#[derive(Default)]
pub struct SimpleUiDiffEngine;

impl UiDiffEngine for SimpleUiDiffEngine {
    fn diff(&self, left: &Value, right: &Value) -> Result<UiDiff, EngineError> {
        let payload = serde_json::json!({
            "left": left,
            "right": right,
        });

        Ok(UiDiff {
            wire_version: "1.0".into(),
            payload,
        })
    }
}
