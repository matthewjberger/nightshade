use nalgebra_glm::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IkChain {
    pub root_bone: String,
    pub mid_bone: String,
    pub tip_bone: String,
    pub target: Vec3,
    pub pole: Option<Vec3>,
    pub weight: f32,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IkChainMulti {
    pub bones: Vec<String>,
    pub target: Vec3,
    pub weight: f32,
    pub enabled: bool,
    pub iterations: u32,
    pub max_angle: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AimConstraint {
    pub bones: Vec<String>,
    pub target: Vec3,
    pub forward: Vec3,
    pub weight: f32,
    pub enabled: bool,
    pub cone_angle: f32,
}
