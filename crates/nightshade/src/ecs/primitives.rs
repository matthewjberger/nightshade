//! Small primitive component types and shared helpers that don't justify
//! their own module. Grouped here to keep the top-level `ecs/` tree
//! shallow.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

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

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CastsShadow;

impl enum2schema::Schema for CastsShadow {
    fn schema() -> enum2schema::serde_json::Value {
        enum2schema::serde_json::json!({ "type": "null" })
    }
}

/// Stable engine-side identity for entities that round-trip through a
/// save file or that other entities cross-reference. Opt-in: omit on
/// transient entities (particles, projectiles, popups). Engine mints
/// values from `world.resources.entities.next_guid`, which is itself
/// persisted in the scene file so freshly-loaded scenes don't collide
/// with the writer's ids.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Guid(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderLayer(pub u32);

impl enum2schema::Schema for RenderLayer {
    fn schema() -> enum2schema::serde_json::Value {
        enum2schema::serde_json::json!({ "type": "integer" })
    }
}

impl Default for RenderLayer {
    fn default() -> Self {
        Self(Self::WORLD)
    }
}

impl RenderLayer {
    pub const WORLD: u32 = 0;
    pub const OVERLAY: u32 = 1;

    pub fn new(layer: u32) -> Self {
        Self(layer)
    }

    pub fn is_in_layer(&self, layer: u32) -> bool {
        self.0 == layer
    }

    pub fn is_overlay(&self) -> bool {
        self.0 == Self::OVERLAY
    }
}

/// Bitmask of "culling layers" this entity belongs to. Independent
/// from [`RenderLayer`] (which selects WORLD vs OVERLAY rendering).
/// Cameras with a [`CameraCullingMask`] component skip entities
/// whose `(entity_layers & camera_mask) == 0`. The default for
/// entities without the component is bit 0 set (layer 0), and the
/// default for cameras without the component is all bits set, so
/// per-camera culling is purely opt-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CullingMask(pub u32);

impl Default for CullingMask {
    fn default() -> Self {
        Self(1)
    }
}

impl CullingMask {
    pub const DEFAULT_LAYER: u32 = 1;

    pub fn new(mask: u32) -> Self {
        Self(mask)
    }

    pub fn add_layer(&mut self, layer_index: u32) {
        self.0 |= 1u32 << layer_index;
    }

    pub fn remove_layer(&mut self, layer_index: u32) {
        self.0 &= !(1u32 << layer_index);
    }

    pub fn intersects(self, other: CullingMask) -> bool {
        (self.0 & other.0) != 0
    }
}

/// Camera-side counterpart to [`CullingMask`]. The renderer
/// resolves this into `EffectiveShading::culling_mask` per frame
/// and the mesh / skinned-mesh prepare passes skip any entity whose
/// own `CullingMask` doesn't intersect the camera's mask. Cameras
/// without this component render every entity (mask = `!0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraCullingMask(pub u32);

impl Default for CameraCullingMask {
    fn default() -> Self {
        Self(!0)
    }
}

impl CameraCullingMask {
    pub fn new(mask: u32) -> Self {
        Self(mask)
    }

    pub fn intersects(self, layer: CullingMask) -> bool {
        (self.0 & layer.0) != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum EasingFunction {
    #[default]
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuartIn,
    QuartOut,
    QuartInOut,
    QuintIn,
    QuintOut,
    QuintInOut,
    SineIn,
    SineOut,
    SineInOut,
    ExpoIn,
    ExpoOut,
    ExpoInOut,
    CircIn,
    CircOut,
    CircInOut,
    BackIn,
    BackOut,
    BackInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
}

impl EasingFunction {
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::QuadIn => t * t,
            Self::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
            Self::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Self::CubicIn => t * t * t,
            Self::CubicOut => 1.0 - (1.0 - t).powi(3),
            Self::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Self::QuartIn => t * t * t * t,
            Self::QuartOut => 1.0 - (1.0 - t).powi(4),
            Self::QuartInOut => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
                }
            }
            Self::QuintIn => t * t * t * t * t,
            Self::QuintOut => 1.0 - (1.0 - t).powi(5),
            Self::QuintInOut => {
                if t < 0.5 {
                    16.0 * t * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
                }
            }
            Self::SineIn => 1.0 - (t * PI / 2.0).cos(),
            Self::SineOut => (t * PI / 2.0).sin(),
            Self::SineInOut => -(PI * t).cos() / 2.0 + 0.5,
            Self::ExpoIn => {
                if t == 0.0 {
                    0.0
                } else {
                    (2.0f32).powf(10.0 * t - 10.0)
                }
            }
            Self::ExpoOut => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - (2.0f32).powf(-10.0 * t)
                }
            }
            Self::ExpoInOut => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    (2.0f32).powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - (2.0f32).powf(-20.0 * t + 10.0)) / 2.0
                }
            }
            Self::CircIn => 1.0 - (1.0 - t * t).sqrt(),
            Self::CircOut => (1.0 - (t - 1.0).powi(2)).sqrt(),
            Self::CircInOut => {
                if t < 0.5 {
                    (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
                } else {
                    ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
                }
            }
            Self::BackIn => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            Self::BackOut => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            Self::BackInOut => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (2.0 * t - 2.0) + c2) + 2.0) / 2.0
                }
            }
            Self::ElasticIn => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let c4 = 2.0 * PI / 3.0;
                    -(2.0f32).powf(10.0 * t - 10.0) * ((10.0 * t - 10.75) * c4).sin()
                }
            }
            Self::ElasticOut => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let c4 = 2.0 * PI / 3.0;
                    (2.0f32).powf(-10.0 * t) * ((10.0 * t - 0.75) * c4).sin() + 1.0
                }
            }
            Self::ElasticInOut => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let c5 = 2.0 * PI / 4.5;
                    if t < 0.5 {
                        -(2.0f32).powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin() / 2.0
                    } else {
                        (2.0f32).powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin() / 2.0
                            + 1.0
                    }
                }
            }
            Self::BounceIn => 1.0 - Self::BounceOut.evaluate(1.0 - t),
            Self::BounceOut => bounce_out(t),
            Self::BounceInOut => {
                if t < 0.5 {
                    (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0
                }
            }
        }
    }
}

fn bounce_out(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}
