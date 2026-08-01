use crate::resources::Farm;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn follow_the_farmer(
    time: Res<Time>,
    window: Res<Window>,
    farm: Res<Farm>,
    storage: &mut nightshade::ecs::Storage,
) {
    let (Some(camera), Some(farmer)) = (farm.camera, farm.farmer) else {
        return;
    };
    let Some(target) = storage
        .get::<LocalTransform>(farmer)
        .map(|transform| transform.translation)
    else {
        return;
    };

    if let Some((_, height)) = window.cached_viewport_size
        && let Some(camera_component) = storage.get_mut::<Camera>(camera)
    {
        let y_mag = height as f32 / theme::PIXELS_PER_UNIT / 2.0;
        camera_component.projection = Projection::Orthographic(OrthographicCamera {
            x_mag: None,
            y_mag: y_mag.max(1.0),
            z_near: 0.1,
            z_far: 100.0,
        });
    }

    let Some(transform) = storage.get_mut::<LocalTransform>(camera) else {
        return;
    };
    let share = (theme::CAMERA_FOLLOW * time.delta_time).clamp(0.0, 1.0);
    let followed_x = transform.translation.x + (target.x - transform.translation.x) * share;
    let followed_y = transform.translation.y + (target.y - transform.translation.y) * share;
    let texel = 1.0 / theme::PIXELS_PER_UNIT * 4.0;
    transform.translation.x = (followed_x / texel).round() * texel;
    transform.translation.y = (followed_y / texel).round() * texel;
}
