/// Basis and translation extractors matching the engine transform component,
/// so light math works on plain world matrices.
pub fn transform_translation(matrix: &nalgebra_glm::Mat4) -> nalgebra_glm::Vec3 {
    nalgebra_glm::vec3(matrix[(0, 3)], matrix[(1, 3)], matrix[(2, 3)])
}

/// World-space right axis of a transform matrix, its first column.
pub fn transform_right(matrix: &nalgebra_glm::Mat4) -> nalgebra_glm::Vec3 {
    nalgebra_glm::vec3(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)])
}

/// World-space up axis of a transform matrix, its second column.
pub fn transform_up(matrix: &nalgebra_glm::Mat4) -> nalgebra_glm::Vec3 {
    nalgebra_glm::vec3(matrix[(0, 1)], matrix[(1, 1)], matrix[(2, 1)])
}

/// World-space forward axis of a transform matrix, the negated third column.
pub fn transform_forward(matrix: &nalgebra_glm::Mat4) -> nalgebra_glm::Vec3 {
    nalgebra_glm::vec3(-matrix[(0, 2)], -matrix[(1, 2)], -matrix[(2, 2)])
}

/// Orthographic projection with a reverse-Z depth range for a directional
/// light, mapping `near` to 1.0 and `far` to 0.0.
pub fn reverse_z_ortho_light(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> nalgebra_glm::Mat4 {
    let width = right - left;
    let height = top - bottom;
    let depth = far - near;

    nalgebra_glm::Mat4::new(
        2.0 / width,
        0.0,
        0.0,
        -(right + left) / width,
        0.0,
        2.0 / height,
        0.0,
        -(top + bottom) / height,
        0.0,
        0.0,
        1.0 / depth,
        -near / depth,
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

/// The eight world-space corner positions of a perspective view frustum,
/// near plane first then far plane.
pub fn get_frustum_corners_world_space(
    view: &nalgebra_glm::Mat4,
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
) -> [nalgebra_glm::Vec3; 8] {
    let inv_view = nalgebra_glm::inverse(view);
    let tan_half_fov = (fov / 2.0).tan();

    let near_height = near * tan_half_fov;
    let near_width = near_height * aspect;
    let far_height = far * tan_half_fov;
    let far_width = far_height * aspect;

    let corners_view = [
        nalgebra_glm::vec3(-near_width, -near_height, -near),
        nalgebra_glm::vec3(near_width, -near_height, -near),
        nalgebra_glm::vec3(near_width, near_height, -near),
        nalgebra_glm::vec3(-near_width, near_height, -near),
        nalgebra_glm::vec3(-far_width, -far_height, -far),
        nalgebra_glm::vec3(far_width, -far_height, -far),
        nalgebra_glm::vec3(far_width, far_height, -far),
        nalgebra_glm::vec3(-far_width, far_height, -far),
    ];

    let mut corners_world = [nalgebra_glm::vec3(0.0, 0.0, 0.0); 8];
    for (index, corner) in corners_view.iter().enumerate() {
        let world_pos = inv_view * nalgebra_glm::vec4(corner.x, corner.y, corner.z, 1.0);
        corners_world[index] = nalgebra_glm::vec3(world_pos.x, world_pos.y, world_pos.z);
    }
    corners_world
}

/// Output of [`calculate_cascade_view_projection`].
pub struct CascadeViewProjectionResult {
    /// Combined light view-projection matrix for the cascade.
    pub view_projection: nalgebra_glm::Mat4,
    /// Side length of the cascade's orthographic box in world units.
    pub cascade_diameter: f32,
}

/// Builds a stable, texel-snapped cascade view-projection matching the shadow
/// depth pass exactly. The ortho box is fit to the slice's bounding sphere so
/// its size is constant as the camera orbits, and the sphere center is snapped
/// to the shadow-map texel grid to keep shadow texels from crawling. The
/// returned `cascade_diameter` is the box side length, feeding the sampler's
/// world-space texel size for normal-bias scaling.
pub fn calculate_cascade_view_projection(
    frustum_corners: &[nalgebra_glm::Vec3; 8],
    light_direction: &nalgebra_glm::Vec3,
    cascade_resolution: f32,
    _cascade_far: f32,
) -> CascadeViewProjectionResult {
    let mut center = nalgebra_glm::vec3(0.0, 0.0, 0.0);
    for corner in frustum_corners {
        center += corner;
    }
    center /= 8.0;

    let mut radius = 0.0f32;
    for corner in frustum_corners {
        radius = radius.max(nalgebra_glm::length(&(corner - center)));
    }

    let up = if light_direction.y.abs() > 0.99 {
        nalgebra_glm::vec3(1.0, 0.0, 0.0)
    } else {
        nalgebra_glm::vec3(0.0, 1.0, 0.0)
    };

    let light_dir_normalized = light_direction.normalize();

    let light_view = nalgebra_glm::look_at(
        &(-light_dir_normalized),
        &nalgebra_glm::vec3(0.0, 0.0, 0.0),
        &up,
    );

    let center_light_space = light_view * nalgebra_glm::vec4(center.x, center.y, center.z, 1.0);

    let units_per_texel = (2.0 * radius) / cascade_resolution;
    let snapped_x = (center_light_space.x / units_per_texel).floor() * units_per_texel;
    let snapped_y = (center_light_space.y / units_per_texel).floor() * units_per_texel;

    let min_x = snapped_x - radius;
    let max_x = snapped_x + radius;
    let min_y = snapped_y - radius;
    let max_y = snapped_y + radius;

    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;
    for corner in frustum_corners {
        let light_space = light_view * nalgebra_glm::vec4(corner.x, corner.y, corner.z, 1.0);
        min_z = min_z.min(light_space.z);
        max_z = max_z.max(light_space.z);
    }

    let z_mult = 10.0;
    if min_z < 0.0 {
        min_z *= z_mult;
    } else {
        min_z /= z_mult;
    }
    if max_z < 0.0 {
        max_z /= z_mult;
    } else {
        max_z *= z_mult;
    }

    let light_projection = reverse_z_ortho_light(min_x, max_x, min_y, max_y, min_z, max_z);

    let cascade_diameter = radius * 2.0;

    CascadeViewProjectionResult {
        view_projection: light_projection * light_view,
        cascade_diameter,
    }
}

/// Whether a world-space sphere lies inside or intersects the frustum, false
/// only when it is fully outside at least one plane.
pub fn sphere_in_frustum(
    center: &nalgebra_glm::Vec3,
    radius: f32,
    frustum_planes: &[nalgebra_glm::Vec4; 6],
) -> bool {
    for plane in frustum_planes {
        let distance = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;
        if distance < -radius {
            return false;
        }
    }
    true
}

/// GPU representation of a punctual light in the clustered lighting buffer.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightData {
    /// World position, w lane unused.
    pub position: [f32; 4],
    /// Normalized world-space direction, w lane unused.
    pub direction: [f32; 4],
    /// Linear color premultiplied by intensity, w lane unused.
    pub color: [f32; 4],
    /// Light type discriminant matching the shader's enum.
    pub light_type: u32,
    /// Falloff range in world units.
    pub range: f32,
    /// Cosine of the inner spot cone angle.
    pub inner_cone: f32,
    /// Cosine of the outer spot cone angle.
    pub outer_cone: f32,
    /// Shadow atlas slot index, or -1 when the light casts no shadow.
    pub shadow_index: i32,
    /// World-space radius used for soft-shadow penumbra sizing.
    pub light_size: f32,
    /// Packed cookie texture layer, or [`COOKIE_LAYER_NONE`] when absent.
    pub cookie_layer: u32,
    /// Padding to keep the struct 16-byte aligned.
    pub _padding: f32,
}

/// Sentinel `cookie_layer` value meaning the light has no cookie texture.
pub const COOKIE_LAYER_NONE: u32 = 0xFFFFFFFF;

pub use crate::wgpu::passes::shadow_depth::{CASCADE_SPLIT_DISTANCES, scale_cascade_splits};

/// Maximum number of spotlight shadow slots in the depth atlas.
pub const MAX_SPOTLIGHT_SHADOWS: usize = 64;

/// Output of a punctual-light collection pass over the scene.
pub struct LightCollectionResult {
    /// GPU light records in buffer order.
    pub lights_data: Vec<LightData>,
    /// The active directional light and its world transform, if any.
    pub directional_light: Option<(crate::config::RenderLightData, nalgebra_glm::Mat4)>,
    /// Count of directional lights found.
    pub num_directional_lights: u32,
    /// Map from source entity to its index in `lights_data`.
    pub entity_to_index: std::collections::HashMap<nightshade_ecs::Entity, usize>,
}

/// Maximum number of area lights shaded per fragment. Area lights are few and
/// expensive, so they are iterated directly rather than clustered.
pub const MAX_AREA_LIGHTS: usize = 16;

/// Sentinel emissive-texture layer meaning "no texture" for an area light.
pub const AREA_EMISSIVE_LAYER_NONE: u32 = 0xFFFFFFFF;

/// Area-light shape discriminant for a rectangle emitter.
pub const AREA_SHAPE_RECTANGLE: u32 = 0;
/// Area-light shape discriminant for a disk emitter.
pub const AREA_SHAPE_DISK: u32 = 1;
/// Area-light shape discriminant for a sphere emitter.
pub const AREA_SHAPE_SPHERE: u32 = 2;
/// Area-light shape discriminant for a tube emitter.
pub const AREA_SHAPE_TUBE: u32 = 3;

/// GPU representation of an area light. The emitter normal (emission direction)
/// is packed into the w lanes of position/right/up to keep the struct at 96
/// bytes with 16-byte alignment.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AreaLightData {
    /// World position, w lane holds the emitter normal x.
    pub position: [f32; 4],
    /// Right basis vector, w lane holds the emitter normal y.
    pub right: [f32; 4],
    /// Up basis vector, w lane holds the emitter normal z.
    pub up: [f32; 4],
    /// Linear color premultiplied by intensity, w lane unused.
    pub color: [f32; 4],
    /// Shape discriminant, one of the `AREA_SHAPE_*` constants.
    pub shape: u32,
    /// Falloff range in world units.
    pub range: f32,
    /// Emitter radius or tube radius in world units.
    pub radius: f32,
    /// Non-zero when the emitter radiates from both faces.
    pub two_sided: u32,
    /// Shadow atlas slot index, or -1 when the light casts no shadow.
    pub shadow_index: i32,
    /// Packed emissive texture layer, or [`AREA_EMISSIVE_LAYER_NONE`] when absent.
    pub emissive_layer: u32,
    /// Padding to keep the struct 16-byte aligned.
    pub _pad0: f32,
    /// Padding to keep the struct 16-byte aligned.
    pub _pad1: f32,
}

/// Uniform paired with the area-light storage buffer carrying the live count.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AreaUniforms {
    /// Number of live area lights in the storage buffer.
    pub count: u32,
    /// Padding to keep the struct 16-byte aligned.
    pub _pad0: u32,
    /// Padding to keep the struct 16-byte aligned.
    pub _pad1: u32,
    /// Padding to keep the struct 16-byte aligned.
    pub _pad2: u32,
}

/// Output of an area-light collection pass over the scene.
pub struct AreaLightCollectionResult {
    /// GPU area-light records in buffer order.
    pub area_lights_data: Vec<AreaLightData>,
    /// Map from source entity to its index in `area_lights_data`.
    pub entity_to_index: std::collections::HashMap<nightshade_ecs::Entity, usize>,
}

/// Writes each area light's resolved shadow slot into its GPU record.
pub fn apply_area_light_shadow_indices(
    area_lights_data: &mut [AreaLightData],
    entity_to_shadow_index: &std::collections::HashMap<nightshade_ecs::Entity, i32>,
    entity_to_area_index: &std::collections::HashMap<nightshade_ecs::Entity, usize>,
) {
    for (entity, &shadow_index) in entity_to_shadow_index {
        if let Some(&area_index) = entity_to_area_index.get(entity)
            && area_index < area_lights_data.len()
        {
            area_lights_data[area_index].shadow_index = shadow_index;
        }
    }
}

/// The sun's cascaded-shadow matrices and parameters resolved for one frame.
pub struct CascadeShadowResult {
    /// World-to-light-clip matrix per cascade.
    pub cascade_view_projections: [[[f32; 4]; 4]; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    /// Each cascade's frustum diameter in world units.
    pub cascade_diameters: [f32; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    /// View-space distance to each cascade's far split.
    pub cascade_split_distances: [f32; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    /// World-to-light-clip matrix for the whole light, used where a single
    /// projection is needed.
    pub light_view_projection: [[f32; 4]; 4],
    /// Constant depth bias along the light direction.
    pub shadow_bias: f32,
    /// Depth bias scaled by the shadow-map texel size in world units.
    pub shadow_normal_bias: f32,
    /// The light's world-space radius, driving soft-shadow penumbra width.
    pub light_size: f32,
    /// `1.0` when the sun casts shadows this frame, `0.0` otherwise.
    pub shadows_enabled: f32,
}

/// The default shadow normal bias, in shadow-map texels.
pub fn default_shadow_normal_bias_value() -> f32 {
    1.8
}

/// Resolves each spotlight's cookie texture path to a bindless material layer,
/// writing the packed layer into its GPU record. Lights without a registered
/// cookie keep the [`COOKIE_LAYER_NONE`] sentinel.
pub fn resolve_cookie_layers(
    configs: &crate::wgpu::render_configs::RenderInputs,
    lights_data: &mut [LightData],
    entity_to_index: &std::collections::HashMap<nightshade_ecs::Entity, usize>,
    material_layer_map: &std::collections::HashMap<
        crate::asset_id::TextureId,
        crate::wgpu::material_texture_arrays::MaterialTextureLayer,
    >,
    texture_registry: &crate::generational_registry::GenerationalRegistry<()>,
) {
    for (entity, &index) in entity_to_index {
        let Some(light) = configs
            .scene_world
            .get::<crate::config::RenderLightData>(*entity)
        else {
            continue;
        };
        let Some(cookie_name) = light.cookie_texture.as_deref() else {
            continue;
        };
        if cookie_name.is_empty() {
            continue;
        }
        let Some((texture_index, _)) =
            crate::generational_registry::registry_lookup_index(texture_registry, cookie_name)
        else {
            continue;
        };
        let texture_id = crate::asset_id::TextureId::new(texture_index, 0);
        if let Some(entry) = material_layer_map.get(&texture_id)
            && let Some(slot) = lights_data.get_mut(index)
        {
            slot.cookie_layer = entry.packed();
        }
    }
}

/// Resolves each area light's emissive texture path to a bindless material
/// layer, mirroring [`resolve_cookie_layers`]. The texture must already be
/// registered as a material texture. Lights without a texture keep the
/// `AREA_EMISSIVE_LAYER_NONE` sentinel.
pub fn resolve_area_emissive_layers(
    configs: &crate::wgpu::render_configs::RenderInputs,
    area_lights_data: &mut [AreaLightData],
    entity_to_index: &std::collections::HashMap<nightshade_ecs::Entity, usize>,
    material_layer_map: &std::collections::HashMap<
        crate::asset_id::TextureId,
        crate::wgpu::material_texture_arrays::MaterialTextureLayer,
    >,
    texture_registry: &crate::generational_registry::GenerationalRegistry<()>,
) {
    for (entity, &index) in entity_to_index {
        let Some(light) = configs
            .scene_world
            .get::<crate::config::RenderLightData>(*entity)
        else {
            continue;
        };
        let Some(texture_name) = light.area_emissive_texture.as_deref() else {
            continue;
        };
        if texture_name.is_empty() {
            continue;
        }
        let Some((texture_index, _)) =
            crate::generational_registry::registry_lookup_index(texture_registry, texture_name)
        else {
            continue;
        };
        let texture_id = crate::asset_id::TextureId::new(texture_index, 0);
        if let Some(entry) = material_layer_map.get(&texture_id)
            && let Some(slot) = area_lights_data.get_mut(index)
        {
            slot.emissive_layer = entry.packed();
        }
    }
}

/// Output of [`collect_spotlight_shadows`].
pub struct SpotlightShadowResult {
    /// Per-slot shadow sampling data for the spotlight and area-light atlas.
    pub shadow_data: Vec<crate::wgpu::passes::SpotlightShadowData>,
    /// Map from source entity to its atlas slot index.
    pub entity_to_shadow_index: std::collections::HashMap<nightshade_ecs::Entity, i32>,
}

/// Whether a light is shadowed through the spotlight depth atlas. Spotlights
/// always are. Area lights are when their emitter has a fixed normal, which
/// rules out spheres and tubes. Both the sampling-data collection and the
/// atlas render pass call this so they assign the same lights to the same slots.
pub fn light_casts_atlas_shadow(light: &crate::config::RenderLightData) -> bool {
    match light.light_type {
        crate::config::RenderLightType::Spot => true,
        crate::config::RenderLightType::Area => matches!(
            light.area_shape,
            crate::config::RenderAreaLightShape::Rectangle
                | crate::config::RenderAreaLightShape::Disk
        ),
        _ => false,
    }
}

/// Field of view for a light's atlas shadow projection. Area lights emit across
/// a wide hemisphere, so they use a fixed wide angle instead of a cone angle.
pub fn atlas_shadow_fov(light: &crate::config::RenderLightData) -> f32 {
    match light.light_type {
        crate::config::RenderLightType::Area => 2.4,
        _ => light.outer_cone_angle * 2.0,
    }
}

/// Collects the spotlight and area-light shadow sampling data and entity to
/// slot mapping. This delegates to the same atlas assignment the shadow depth
/// pass renders from, so the sampled slots always match the rendered slots.
pub fn collect_spotlight_shadows(
    configs: &crate::wgpu::render_configs::RenderInputs,
) -> SpotlightShadowResult {
    let assignment = &configs.shadow_atlas;
    SpotlightShadowResult {
        shadow_data: assignment.data.clone(),
        entity_to_shadow_index: assignment.entity_to_index.clone(),
    }
}

/// Writes each spotlight's resolved shadow slot into its GPU light record.
pub fn apply_spotlight_shadow_indices(
    lights_data: &mut [LightData],
    entity_to_shadow_index: &std::collections::HashMap<nightshade_ecs::Entity, i32>,
    entity_to_lights_index: &std::collections::HashMap<nightshade_ecs::Entity, usize>,
) {
    for (entity, &shadow_index) in entity_to_shadow_index {
        if let Some(&lights_index) = entity_to_lights_index.get(entity)
            && lights_index < lights_data.len()
        {
            lights_data[lights_index].shadow_index = shadow_index;
        }
    }
}

/// Maximum number of point-light cube shadows rendered per frame.
pub const MAX_POINT_LIGHT_SHADOWS: usize = 16;

/// Selects the closest shadow-casting point lights, writes their slot index
/// into each GPU light record, and returns the per-slot cube shadow data.
pub fn collect_point_light_shadows(
    configs: &crate::wgpu::render_configs::RenderInputs,
    camera_position: nalgebra_glm::Vec3,
    lights_data: &mut [LightData],
    entity_to_lights_index: &std::collections::HashMap<nightshade_ecs::Entity, usize>,
) -> Vec<crate::wgpu::passes::shadow_depth::PointLightShadowData> {
    let mut point_light_candidates: Vec<(
        nightshade_ecs::Entity,
        f32,
        crate::config::RenderLightData,
        nalgebra_glm::Mat4,
    )> = Vec::new();

    let lights: Vec<(
        nightshade_ecs::Entity,
        crate::config::RenderLightData,
        nalgebra_glm::Mat4,
    )> = configs
        .scene_world
        .query_ref::<(
            &crate::config::RenderLightData,
            &crate::render_world::Transform,
        )>()
        .iter()
        .map(|(entity, (light, transform))| (entity, light.clone(), transform.0))
        .collect();
    for (entity, light, transform) in &lights {
        let entity = *entity;
        if matches!(light.light_type, crate::config::RenderLightType::Point) && light.cast_shadows {
            let light_pos =
                nalgebra_glm::vec3(transform[(0, 3)], transform[(1, 3)], transform[(2, 3)]);
            let distance_sq = nalgebra_glm::length2(&(light_pos - camera_position));
            point_light_candidates.push((entity, distance_sq, light.clone(), *transform));
        }
    }

    point_light_candidates
        .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut shadow_data: Vec<crate::wgpu::passes::shadow_depth::PointLightShadowData> = Vec::new();

    for (slot_index, (entity, _distance, light, transform)) in point_light_candidates
        .iter()
        .take(MAX_POINT_LIGHT_SHADOWS)
        .enumerate()
    {
        let light_position =
            nalgebra_glm::vec3(transform[(0, 3)], transform[(1, 3)], transform[(2, 3)]);

        shadow_data.push(crate::wgpu::passes::shadow_depth::PointLightShadowData {
            position: [light_position.x, light_position.y, light_position.z],
            range: light.range.max(0.1),
            bias: light.shadow_bias,
            shadow_index: slot_index as i32,
            _padding: [0.0; 2],
        });

        if let Some(&lights_index) = entity_to_lights_index.get(entity)
            && lights_index < lights_data.len()
        {
            lights_data[lights_index].shadow_index = slot_index as i32;
        }
    }

    shadow_data
}
