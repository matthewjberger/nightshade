use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Twilight {
    pub dawn_start: f32,
    pub dawn_end: f32,
    pub dusk_start: f32,
    pub dusk_end: f32,
}

impl Default for Twilight {
    fn default() -> Self {
        Self {
            dawn_start: 5.5,
            dawn_end: 7.0,
            dusk_start: 17.5,
            dusk_end: 19.5,
        }
    }
}
