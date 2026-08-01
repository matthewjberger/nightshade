use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
#[serde(default)]
pub struct Lifetime {
    pub remaining: f32,
}
