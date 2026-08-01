use crate::data::config::{RenderAreaLightShape, RenderLightType};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
#[schema(string_enum)]
pub enum LightType {
    #[default]
    Directional,
    Point,
    Spot,
    Area,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
#[schema(string_enum)]
pub enum AreaLightShape {
    #[default]
    Rectangle,
    Disk,
    Sphere,
    Tube,
}

#[derive(Debug, Clone, Serialize, Deserialize, enum2schema::Schema)]
pub struct Light {
    pub light_type: LightType,
    #[schema(type = "array", items = "number", len = 3)]
    pub color: nalgebra_glm::Vec3,
    pub intensity: f32,
    pub range: f32,
    pub inner_cone_angle: f32,
    pub outer_cone_angle: f32,
    pub cast_shadows: bool,
    pub shadow_bias: f32,
    #[serde(default)]
    pub shadow_resolution: u32,
    #[serde(default)]
    pub shadow_distance: f32,
    #[serde(default)]
    pub cookie_texture: Option<String>,
    #[serde(default)]
    pub area_shape: AreaLightShape,
    #[serde(default = "default_area_extent")]
    pub area_width: f32,
    #[serde(default = "default_area_extent")]
    pub area_height: f32,
    #[serde(default = "default_area_extent")]
    pub area_radius: f32,
    #[serde(default)]
    pub area_two_sided: bool,
    #[serde(default)]
    pub area_emissive_texture: Option<String>,
    #[serde(default = "default_shadow_normal_bias")]
    pub shadow_normal_bias: f32,
    #[serde(default = "default_shadow_softness")]
    pub shadow_softness: f32,
}

fn default_area_extent() -> f32 {
    1.0
}

fn default_shadow_normal_bias() -> f32 {
    1.8
}

fn default_shadow_softness() -> f32 {
    1.0
}

impl Default for Light {
    fn default() -> Self {
        Self {
            light_type: LightType::default(),
            color: nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
            intensity: 0.0,
            range: 0.0,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: false,
            shadow_bias: 0.0,
            shadow_resolution: 0,
            shadow_distance: 0.0,
            cookie_texture: None,
            area_shape: AreaLightShape::Rectangle,
            area_width: default_area_extent(),
            area_height: default_area_extent(),
            area_radius: default_area_extent(),
            area_two_sided: false,
            area_emissive_texture: None,
            shadow_normal_bias: default_shadow_normal_bias(),
            shadow_softness: default_shadow_softness(),
        }
    }
}

impl Light {
    pub fn with_shadows(mut self, bias: f32) -> Self {
        self.cast_shadows = true;
        self.shadow_bias = bias;
        self
    }
}

#[derive(Clone, Default)]
pub struct RenderLightData {
    pub light_type: RenderLightType,
    pub color: nalgebra_glm::Vec3,
    pub intensity: f32,
    pub range: f32,
    pub inner_cone_angle: f32,
    pub outer_cone_angle: f32,
    pub cast_shadows: bool,
    pub shadow_bias: f32,
    pub shadow_resolution: u32,
    pub shadow_distance: f32,
    pub cookie_texture: Option<String>,
    pub area_shape: RenderAreaLightShape,
    pub area_width: f32,
    pub area_height: f32,
    pub area_radius: f32,
    pub area_two_sided: bool,
    pub area_emissive_texture: Option<String>,
    pub shadow_normal_bias: f32,
    pub shadow_softness: f32,
}

#[derive(Clone)]
pub struct RenderLight {
    pub entity: nightshade::ecs::Entity,
    pub light: RenderLightData,
    pub transform: nalgebra_glm::Mat4,
}

fn base_light_data(
    light_type: RenderLightType,
    color: nalgebra_glm::Vec3,
    intensity: f32,
    range: f32,
) -> RenderLightData {
    RenderLightData {
        light_type,
        color,
        intensity,
        range,
        inner_cone_angle: 0.0,
        outer_cone_angle: std::f32::consts::FRAC_PI_4,
        cast_shadows: false,
        shadow_bias: 0.0015,
        shadow_resolution: 2048,
        shadow_distance: 100.0,
        cookie_texture: None,
        area_shape: RenderAreaLightShape::Rectangle,
        area_width: 1.0,
        area_height: 1.0,
        area_radius: 0.5,
        area_two_sided: false,
        area_emissive_texture: None,
        shadow_normal_bias: 0.02,
        shadow_softness: 1.0,
    }
}

fn direction_transform(
    position: nalgebra_glm::Vec3,
    direction: nalgebra_glm::Vec3,
) -> nalgebra_glm::Mat4 {
    let forward = direction.normalize();
    let world_up = if forward.y.abs() > 0.99 {
        nalgebra_glm::vec3(0.0, 0.0, 1.0)
    } else {
        nalgebra_glm::vec3(0.0, 1.0, 0.0)
    };
    let right = forward.cross(&world_up).normalize();
    let up = right.cross(&forward);
    nalgebra_glm::Mat4::new(
        right.x, up.x, -forward.x, position.x, right.y, up.y, -forward.y, position.y, right.z,
        up.z, -forward.z, position.z, 0.0, 0.0, 0.0, 1.0,
    )
}

impl RenderLight {
    pub fn directional(
        entity: nightshade::ecs::Entity,
        direction: nalgebra_glm::Vec3,
        color: nalgebra_glm::Vec3,
        intensity: f32,
    ) -> Self {
        let mut light = base_light_data(RenderLightType::Directional, color, intensity, 0.0);
        light.cast_shadows = true;
        Self {
            entity,
            light,
            transform: direction_transform(nalgebra_glm::Vec3::zeros(), direction),
        }
    }

    pub fn point(
        entity: nightshade::ecs::Entity,
        position: nalgebra_glm::Vec3,
        color: nalgebra_glm::Vec3,
        intensity: f32,
        range: f32,
    ) -> Self {
        Self {
            entity,
            light: base_light_data(RenderLightType::Point, color, intensity, range),
            transform: nalgebra_glm::translation(&position),
        }
    }

    pub fn spot(
        entity: nightshade::ecs::Entity,
        position: nalgebra_glm::Vec3,
        direction: nalgebra_glm::Vec3,
        color: nalgebra_glm::Vec3,
        intensity: f32,
        range: f32,
        cone: [f32; 2],
    ) -> Self {
        let mut light = base_light_data(RenderLightType::Spot, color, intensity, range);
        light.inner_cone_angle = cone[0];
        light.outer_cone_angle = cone[1];
        Self {
            entity,
            light,
            transform: direction_transform(position, direction),
        }
    }
}
