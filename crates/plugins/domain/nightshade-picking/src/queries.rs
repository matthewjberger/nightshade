use crate::contract::{PickingOptions, PickingRay, PickingResult};
use nalgebra_glm::{Mat4, Vec2, Vec4};
use nightshade::ecs::{Entity, QueryRef};
use nightshade_camera::components::{Camera, Projection};
use nightshade_camera::resources::ActiveCamera;
use nightshade_platform::contract::window::Window;
use nightshade_render::data::bounding_volume::BoundingVolume;
use nightshade_render::data::config::ViewportRect;
use nightshade_render::resources::viewport::Viewport;
use nightshade_scene::contract::GlobalTransform;
use nightshade_scene::contract::primitives::Visibility;

pub type PickableRows<'world> = QueryRef<
    'world,
    (
        &'world BoundingVolume,
        &'world GlobalTransform,
        Option<&'world Visibility>,
    ),
>;

pub type CameraRows<'world> = QueryRef<'world, (&'world Camera, &'world GlobalTransform)>;

#[derive(Debug, Clone, Copy)]
pub struct CameraView {
    pub projection: Projection,
    pub transform: Mat4,
}

impl CameraView {
    pub fn new(camera: &Camera, global_transform: &GlobalTransform) -> Self {
        Self {
            projection: camera.projection,
            transform: global_transform.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScreenArea {
    pub window_size: Vec2,
    pub viewport_rect: Option<ViewportRect>,
}

impl ScreenArea {
    pub fn new(window: &Window, viewport: &Viewport) -> Option<Self> {
        let (width, height) = window.cached_viewport_size?;
        Some(Self {
            window_size: Vec2::new(width as f32, height as f32),
            viewport_rect: viewport.active_viewport_rect,
        })
    }

    fn resolve(&self, screen_pos: Vec2) -> (Vec2, f32, f32) {
        let Some(viewport_rect) = &self.viewport_rect else {
            return (screen_pos, self.window_size.x, self.window_size.y);
        };
        let local = viewport_rect.to_local(screen_pos);
        let scale_x = self.window_size.x / viewport_rect.width;
        let scale_y = self.window_size.y / viewport_rect.height;
        (
            Vec2::new(local.x * scale_x, local.y * scale_y),
            self.window_size.x,
            self.window_size.y,
        )
    }
}

impl PickingRay {
    pub fn from_screen_position(
        area: ScreenArea,
        camera: CameraView,
        screen_pos: Vec2,
    ) -> Option<Self> {
        let (local_pos, viewport_width, viewport_height) = area.resolve(screen_pos);

        let ndc_x = (2.0 * local_pos.x) / viewport_width - 1.0;
        let ndc_y = 1.0 - (2.0 * local_pos.y) / viewport_height;

        let view_matrix = camera.transform.try_inverse()?;
        let aspect_ratio = viewport_width / viewport_height;
        let projection_matrix = camera.projection.matrix_with_aspect(aspect_ratio);
        let inverse_vp = (projection_matrix * view_matrix).try_inverse()?;

        let (origin, direction) = match camera.projection {
            Projection::Perspective { .. } => {
                let camera_pos = camera.transform.column(3).xyz();
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
}

pub fn screen_ray(
    window: &Window,
    viewport: &Viewport,
    active_camera: &ActiveCamera,
    cameras: CameraRows<'_>,
    screen_pos: Vec2,
) -> Option<PickingRay> {
    let area = ScreenArea::new(window, viewport)?;
    let active = active_camera.0?;
    let (_, (camera, global_transform)) = cameras.iter().find(|(entity, _)| *entity == active)?;
    PickingRay::from_screen_position(area, CameraView::new(camera, global_transform), screen_pos)
}

pub fn pick_entities(
    rows: PickableRows<'_>,
    ray: PickingRay,
    options: PickingOptions,
) -> Vec<PickingResult> {
    let mut results = Vec::new();

    for (entity, (bounding_volume, global_transform, visibility)) in rows.iter() {
        if options.ignore_invisible && visibility.is_some_and(|visible| !visible.visible) {
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

pub fn pick_closest_entity(rows: PickableRows<'_>, ray: PickingRay) -> Option<PickingResult> {
    pick_entities(rows, ray, PickingOptions::default())
        .into_iter()
        .next()
}

pub fn pick_entities_in_frustum(
    rows: PickableRows<'_>,
    camera: CameraView,
    window_size: Vec2,
    entities: &[Entity],
) -> Vec<Entity> {
    let Some(view_matrix) = camera.transform.try_inverse() else {
        return Vec::new();
    };
    if window_size.y <= 0.0 {
        return Vec::new();
    }
    let aspect_ratio = window_size.x / window_size.y;
    let view_projection = camera.projection.matrix_with_aspect(aspect_ratio) * view_matrix;

    let wanted: std::collections::HashSet<Entity> = entities.iter().copied().collect();
    let mut visible_entities = Vec::new();

    for (entity, (bounding_volume, global_transform, _)) in rows.iter() {
        if !wanted.contains(&entity) {
            continue;
        }

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
