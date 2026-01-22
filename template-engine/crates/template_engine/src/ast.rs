use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Template {
    pub wire_version: String,
    pub segments: Vec<Segment>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub start_byte: u32,
    pub end_byte: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum Segment {
    #[serde(rename = "text")]
    Text { value: String, span: Option<Span> },

    #[serde(rename = "var")]
    Var {
        slug: String,
        kind: VarKind,
        source: Option<Source>,
        schema: Option<String>,
        value: Option<Value>,
        span: Option<Span>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum VarKind {
    Preset,
    Runtime,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    AgentConfig,
    SectorConfigs,
}
