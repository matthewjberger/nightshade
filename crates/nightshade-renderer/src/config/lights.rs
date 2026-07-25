//! Lights as the renderer sees them: the light list, per-light GPU data,
//! the area-light shapes, and the shadow caster set.

/// Scene lighting the renderer collects once per dispatch and every lit pass
/// reads, replacing per-pass `collect_lights`, `collect_area_lights`,
/// `calculate_cascade_shadows`, and `query_sun` calls. The base light list is
/// carried without per-pass shadow-index application; each pass still resolves
/// its own spotlight, point, and cookie indices against its texture arrays.
#[cfg(feature = "wgpu")]
#[derive(Clone, Default)]
pub struct RenderLighting {
    /// GPU-ready light records for every non-area light.
    pub lights_data: Vec<crate::wgpu::passes::geometry::projection::LightData>,
    /// Number of directional lights in the scene.
    pub num_directional_lights: u32,
    /// Primary directional light direction in world space.
    pub directional_light_direction: [f32; 4],
    /// True when at least one directional light is present.
    pub has_directional_light: bool,
    /// Entity to index map into `lights_data`.
    pub entity_to_lights_index: std::collections::HashMap<nightshade_ecs::Entity, usize>,
    /// Shadow view-projection for each directional cascade.
    pub cascade_view_projections: [[[f32; 4]; 4]; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    /// World-space diameter each cascade covers.
    pub cascade_diameters: [f32; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    /// View-space far split distance for each cascade.
    pub cascade_split_distances: [f32; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    /// Directional light view-projection for single-map shadows.
    pub light_view_projection: [[f32; 4]; 4],
    /// Depth bias applied when sampling shadow maps.
    pub shadow_bias: f32,
    /// Normal-scaled bias applied when sampling shadow maps.
    pub shadow_normal_bias: f32,
    /// Angular size of the directional light for soft shadows.
    pub directional_light_size: f32,
    /// Shadow enable flag, 1.0 on and 0.0 off.
    pub shadows_enabled: f32,
    /// GPU-ready records for every area light.
    pub area_lights_data: Vec<crate::wgpu::passes::geometry::projection::AreaLightData>,
    /// Entity to index map into `area_lights_data`.
    pub area_entity_to_index: std::collections::HashMap<nightshade_ecs::Entity, usize>,
    /// Resolved sun direction in world space.
    pub sun_direction: nalgebra_glm::Vec3,
    /// Resolved sun color.
    pub sun_color: nalgebra_glm::Vec3,
}

/// One scene light with its world transform, snapshotted once per frame so the
/// lighting, shadow, and projection passes read lights here instead of walking
/// the ECS themselves.
/// A light's kind, mirrored from the `LightType` component enum so the render
/// passes do not read that type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RenderLightType {
    /// Infinitely distant light with parallel rays.
    #[default]
    Directional,
    /// Omnidirectional point light.
    Point,
    /// Cone-shaped spotlight.
    Spot,
    /// Shaped area light.
    Area,
}

/// An area light's shape, mirrored from the `AreaLightShape` component enum so
/// the render passes do not read that type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RenderAreaLightShape {
    /// Flat rectangle.
    #[default]
    Rectangle,
    /// Flat disk.
    Disk,
    /// Sphere.
    Sphere,
    /// Capsule-like tube.
    Tube,
}

/// A light's render parameters snapshotted once per frame from the `Light`
/// component, so the lighting, shadow, and projection passes read them here
/// instead of that component.
#[derive(Clone, Default)]
pub struct RenderLightData {
    /// Kind of light.
    pub light_type: RenderLightType,
    /// Linear RGB color.
    pub color: nalgebra_glm::Vec3,
    /// Brightness multiplier.
    pub intensity: f32,
    /// Reach distance in world units for point and spot lights.
    pub range: f32,
    /// Spotlight inner cone half-angle in radians.
    pub inner_cone_angle: f32,
    /// Spotlight outer cone half-angle in radians.
    pub outer_cone_angle: f32,
    /// Whether the light casts shadows.
    pub cast_shadows: bool,
    /// Depth bias applied when sampling this light's shadow map.
    pub shadow_bias: f32,
    /// Shadow map edge length in texels.
    pub shadow_resolution: u32,
    /// Maximum shadow distance in world units.
    pub shadow_distance: f32,
    /// Optional projected cookie texture name.
    pub cookie_texture: Option<String>,
    /// Area light shape.
    pub area_shape: RenderAreaLightShape,
    /// Area light width in world units.
    pub area_width: f32,
    /// Area light height in world units.
    pub area_height: f32,
    /// Area light radius in world units.
    pub area_radius: f32,
    /// Whether the area light emits from both faces.
    pub area_two_sided: bool,
    /// Optional emissive texture name for the area light.
    pub area_emissive_texture: Option<String>,
    /// Normal-scaled bias applied when sampling this light's shadow map.
    pub shadow_normal_bias: f32,
    /// Shadow penumbra softness.
    pub shadow_softness: f32,
}

/// One scene light with its world transform, snapshotted once per frame so the
/// lighting, shadow, and projection passes read lights here instead of walking
/// the ECS themselves.
#[derive(Clone)]
pub struct RenderLight {
    /// The scene entity this light belongs to.
    pub entity: nightshade_ecs::Entity,
    /// The light's render parameters.
    pub light: RenderLightData,
    /// World transform, whose forward axis is read as `-column2`.
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

/// A world transform whose forward axis (which the passes read as `-column2`)
/// is `direction`, positioned at `position`.
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
    /// A shadow-casting directional light (a sun) pointing along `direction`.
    pub fn directional(
        entity: nightshade_ecs::Entity,
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

    /// A point light at `position` reaching out to `range`.
    pub fn point(
        entity: nightshade_ecs::Entity,
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

    /// A spot light at `position` aimed along `direction`, with `[inner, outer]`
    /// cone half-angles in radians.
    pub fn spot(
        entity: nightshade_ecs::Entity,
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

/// The skinned shadow-casting entities collected once per frame, so the shadow
/// pass enumerates skinned occluders from here. Static and instanced casters
/// carry the [`crate::render_world::ShadowCaster`] tag in the scene world.
#[derive(Clone, Default)]
pub struct RenderShadowCasters {
    /// Skinned entities that cast shadows.
    pub skinned: Vec<nightshade_ecs::Entity>,
}
