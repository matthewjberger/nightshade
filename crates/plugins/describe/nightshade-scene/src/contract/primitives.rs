use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct Name(pub String);

impl enum2schema::Schema for Name {
    fn schema() -> enum2schema::serde_json::Value {
        enum2schema::serde_json::json!({ "type": "string" })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
#[serde(default)]
pub struct Visibility {
    pub visible: bool,
}

impl Default for Visibility {
    fn default() -> Self {
        Self { visible: true }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Guid(pub u64);
