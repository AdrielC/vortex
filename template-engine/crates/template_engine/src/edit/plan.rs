use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ast::{Source, VarKind};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EditPlan {
    pub wire_version: String,
    pub fork_id: String,
    pub edits: Vec<Edit>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Edit {
    #[serde(rename = "replace_text")]
    ReplaceText { target: TextTarget, value: String },

    #[serde(rename = "insert_var")]
    InsertVar { target: TextTarget, var: VarSpec },

    #[serde(rename = "delete_var")]
    DeleteVar { segment_index: usize },

    #[serde(rename = "update_var")]
    UpdateVar { segment_index: usize, patch: VarPatch },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextTarget {
    pub segment_index: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VarSpec {
    pub slug: String,
    pub kind: VarKind,
    pub source: Option<Source>,
    pub schema: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct VarPatch {
    pub source: Option<Option<Source>>,
    pub schema: Option<Option<String>>,
    pub kind: Option<VarKind>,
    pub value: Option<Option<Value>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Suggestion {
    #[serde(rename = "var_value")]
    VarValue {
        suggestion_id: String,
        slug: String,
        source: Source,
        schema: Option<String>,
        proposed_value: Value,
        status: SuggestionStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionStatus {
    PendingUser,
    AutoAccepted,
    Rejected,
}
