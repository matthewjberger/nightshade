use crate::ecs::{
    camera::components::Projection,
    world::{Entity, Vec2, Vec3, Vec4, World},
};

#[derive(Debug, Clone, Copy)]
pub struct PickingRay {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl PickingRay {
    pub fn from_screen_position(world: &World, screen_pos: Vec2) -> Option<Self> {
        let window = &world.resources.window;

        let (local_pos, viewport_width, viewport_height) =
            if let Some(viewport_rect) = &window.active_viewport_rect {
                let local = viewport_rect.to_local(screen_pos);
                (local, viewport_rect.width, viewport_rect.height)
            } else {
                let (width, height) = window.cached_viewport_size?;
                (screen_pos, width as f32, height as f32)
            };

        let camera_entity = world.resources.active_camera?;

        let camera = world.get_camera(camera_entity)?;
        let global_transform = world.get_global_transform(camera_entity)?;

        let ndc_x = (2.0 * local_pos.x) / viewport_width - 1.0;
        let ndc_y = 1.0 - (2.0 * local_pos.y) / viewport_height;

        let view_matrix = global_transform.0.try_inverse()?;
        let aspect_ratio = viewport_width / viewport_height;
        let projection_matrix = camera.projection.matrix_with_aspect(aspect_ratio);
        let inverse_vp = (projection_matrix * view_matrix).try_inverse()?;

        let (origin, direction) = match camera.projection {
            Projection::Perspective { .. } => {
                let camera_pos = global_transform.0.column(3).xyz();
                let clip_pos = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
                let world_pos = inverse_vp * clip_pos;
                let world_pos = world_pos.xyz() / world_pos.w;
                let direction = (world_pos - camera_pos).normalize();
                (camera_pos, direction)
            }
            Projection::Orthographic { .. } => {
                let near_clip = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
                let far_clip = Vec4::new(ndc_x, ndc_y, 0.0, 1.0);

                let near_world = inverse_vp * near_clip;
                let far_world = inverse_vp * far_clip;

                let near_point = near_world.xyz() / near_world.w;
                let far_point = far_world.xyz() / far_world.w;

                let direction = (far_point - near_point).normalize();
                (near_point, direction)
            }
        };

        Some(Self { origin, direction })
    }

    pub fn intersect_plane(&self, plane_normal: Vec3, plane_distance: f32) -> Option<Vec3> {
        let denom = nalgebra_glm::dot(&plane_normal, &self.direction);
        if denom.abs() < 1e-6 {
            return None;
        }

        let t = -(nalgebra_glm::dot(&plane_normal, &self.origin) + plane_distance) / denom;
        if t < 0.0 {
            return None;
        }

        Some(self.origin + self.direction * t)
    }

    pub fn intersect_ground_plane(&self, y_level: f32) -> Option<Vec3> {
        self.intersect_plane(Vec3::new(0.0, 1.0, 0.0), -y_level)
    }
}

#[derive(Debug, Clone)]
pub struct PickingResult {
    pub entity: Entity,
    pub distance: f32,
    pub world_position: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct PickingOptions {
    pub max_distance: f32,
    pub ignore_invisible: bool,
}

impl Default for PickingOptions {
    fn default() -> Self {
        Self {
            max_distance: f32::INFINITY,
            ignore_invisible: true,
        }
    }
}

pub fn pick_entities(
    world: &World,
    screen_pos: Vec2,
    options: PickingOptions,
) -> Vec<PickingResult> {
    let ray = match PickingRay::from_screen_position(world, screen_pos) {
        Some(ray) => ray,
        None => return Vec::new(),
    };

    let mut results = Vec::new();

    for entity in world.query_entities(crate::ecs::world::BOUNDING_VOLUME) {
        let bounding_volume = match world.get_bounding_volume(entity) {
            Some(bv) => bv,
            None => continue,
        };

        let global_transform = match world.get_global_transform(entity) {
            Some(gt) => gt,
            None => continue,
        };

        if options.ignore_invisible
            && let Some(visible) = world.get_visibility(entity)
            && !visible.visible
        {
            continue;
        }

        let transformed_bv = bounding_volume.transform(&global_transform.0);

        let to_center = transformed_bv.obb.center - ray.origin;
        let projection = nalgebra_glm::dot(&to_center, &ray.direction);
        let closest_point = if projection < 0.0 {
            ray.origin
        } else {
            ray.origin + ray.direction * projection
        };

        let distance_to_sphere = nalgebra_glm::distance(&closest_point, &transformed_bv.obb.center);
        if distance_to_sphere > transformed_bv.sphere_radius {
            continue;
        }

        if let Some(distance) = transformed_bv.obb.intersect_ray(ray.origin, ray.direction)
            && distance <= options.max_distance
        {
            let world_position = ray.origin + ray.direction * distance;
            results.push(PickingResult {
                entity,
                distance,
                world_position,
            });
        }
    }

    results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    results
}

pub fn pick_closest_entity(world: &World, screen_pos: Vec2) -> Option<PickingResult> {
    pick_entities(world, screen_pos, PickingOptions::default())
        .into_iter()
        .next()
}

pub fn get_ground_position_from_screen(
    world: &World,
    screen_pos: Vec2,
    y_level: f32,
) -> Option<Vec3> {
    let ray = PickingRay::from_screen_position(world, screen_pos)?;
    ray.intersect_ground_plane(y_level)
}

pub fn pick_entities_in_frustum(world: &World, entities: &[Entity]) -> Vec<Entity> {
    let camera_entity = match world.resources.active_camera {
        Some(entity) => entity,
        None => return Vec::new(),
    };

    let camera = match world.get_camera(camera_entity) {
        Some(cam) => cam,
        None => return Vec::new(),
    };

    let global_transform = match world.get_global_transform(camera_entity) {
        Some(gt) => gt,
        None => return Vec::new(),
    };

    let view_matrix = match global_transform.0.try_inverse() {
        Some(vm) => vm,
        None => return Vec::new(),
    };

    let window = &world.resources.window;
    let (viewport_width, viewport_height) = match window.cached_viewport_size {
        Some(size) => size,
        None => return Vec::new(),
    };
    let aspect_ratio = viewport_width as f32 / viewport_height as f32;
    let projection_matrix = camera.projection.matrix_with_aspect(aspect_ratio);
    let view_projection = projection_matrix * view_matrix;

    let mut visible_entities = Vec::new();

    for &entity in entities {
        let bounding_volume = match world.get_bounding_volume(entity) {
            Some(bv) => bv,
            None => continue,
        };

        let global_transform = match world.get_global_transform(entity) {
            Some(gt) => gt,
            None => continue,
        };

        let transformed_bv = bounding_volume.transform(&global_transform.0);
        let center = transformed_bv.obb.center;

        let clip_pos = view_projection * Vec4::new(center.x, center.y, center.z, 1.0);
        if clip_pos.w <= 0.0 {
            continue;
        }

        let ndc = clip_pos.xyz() / clip_pos.w;
        let sphere_radius_ndc = transformed_bv.sphere_radius / clip_pos.w;

        if ndc.x + sphere_radius_ndc >= -1.0
            && ndc.x - sphere_radius_ndc <= 1.0
            && ndc.y + sphere_radius_ndc >= -1.0
            && ndc.y - sphere_radius_ndc <= 1.0
            && ndc.z + sphere_radius_ndc >= 0.0
            && ndc.z - sphere_radius_ndc <= 1.0
        {
            visible_entities.push(entity);
        }
    }

    visible_entities
}
