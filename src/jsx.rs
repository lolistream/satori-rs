//! Element tree compatible with React-style `{ type, props: { children, ... } }`
//! input that satori accepts.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    #[serde(rename = "type")]
    pub tag: String,
    #[serde(default)]
    pub props: Props,
    #[serde(default)]
    pub key: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Props {
    #[serde(default)]
    pub children: Children,
    #[serde(flatten)]
    pub other: IndexMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Child {
    Text(String),
    Element(Element),
    Many(Vec<Child>),
    Null,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Children(pub Vec<Child>);

impl Element {
    pub fn from_json(v: Value) -> serde_json::Result<Self> {
        serde_json::from_value(v)
    }
}
