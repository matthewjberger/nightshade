use crate::commands::next_signed;
use crate::data;
use crate::queries::{half_height, pointer_world_position, viewport_rectangle};
use crate::resources::NeonStaticResources;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn follow(
    time: Res<Time>,
    input: Res<Input>,
    window: Res<Window>,
    viewport: Res<nightshade_render::resources::viewport::Viewport>,
    mut resources: ResMut<NeonStaticResources>,
    storage: &mut nightshade::ecs::Storage,
) {
    let resources = &mut *resources;
    let Some(camera) = resources.camera else {
        return;
    };
    let Some((origin, size)) = viewport_rectangle(&viewport, &window) else {
        return;
    };

    let fitted = half_height(size);
    if let Some(component) = storage.get_mut::<Camera>(camera) {
        component.projection = Projection::Orthographic(OrthographicCamera {
            x_mag: None,
            y_mag: fitted,
            z_near: 0.1,
            z_far: 100.0,
        });
    }

    let target = resources.player_position;
    let share = (data::CAMERA_FOLLOW * time.delta_time).clamp(0.0, 1.0);
    let anchor = resources.camera_anchor + (target - resources.camera_anchor) * share;
    resources.camera_anchor = anchor;

    let pointer = pointer_world_position(
        origin,
        size,
        resources.camera_position,
        input.mouse.position,
    );
    let mut lean = (pointer - target) * data::CAMERA_LEAN;
    let reach = lean.magnitude();
    if reach > data::CAMERA_LEAN_LIMIT {
        lean *= data::CAMERA_LEAN_LIMIT / reach;
    }
    let aimed = anchor + lean;
    resources.camera_position = aimed;

    let shake = resources.floor.shake;
    let offset = Vec2::new(
        next_signed(&mut resources.floor) * shake * data::SHAKE_REACH,
        next_signed(&mut resources.floor) * shake * data::SHAKE_REACH,
    );

    if let Some(transform) = storage.get_mut::<LocalTransform>(camera) {
        transform.translation = Vec3::new(aimed.x + offset.x, aimed.y + offset.y, 20.0);
    }
}
