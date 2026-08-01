use crate::components::Velocity;
use crate::resources::{CoinfallResources, Screen};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn follow(
    time: Res<Time>,
    window: Res<Window>,
    mut resources: ResMut<CoinfallResources>,
    storage: &mut nightshade::ecs::Storage,
) {
    let (Some(camera), Some(player)) = (resources.camera, resources.player) else {
        return;
    };
    let Some((target, drift)) = storage.get::<Velocity>(player).and_then(|body| {
        storage
            .get::<LocalTransform>(player)
            .map(|transform| (transform.translation, body.0.x))
    }) else {
        return;
    };

    let mut half_width = 0.0;
    let mut half_height = 0.0;
    if let Some((width, height)) = window.cached_viewport_size {
        half_height = height as f32 / theme::PIXELS_PER_UNIT / 2.0;
        half_width = width as f32 / theme::PIXELS_PER_UNIT / 2.0;
        if let Some(lens) = storage.get_mut::<Camera>(camera) {
            lens.projection = Projection::Orthographic(OrthographicCamera {
                x_mag: None,
                y_mag: half_height.max(1.0),
                z_near: 0.1,
                z_far: 100.0,
            });
        }
    }

    let columns = resources.solids.width as f32;
    let attract = resources.run.screen != Screen::Playing && resources.run.screen != Screen::Paused;
    let Some(transform) = storage.get_mut::<LocalTransform>(camera) else {
        return;
    };

    let (mut eased_x, mut eased_y) = if attract {
        let at = transform.translation.x;
        let end = (columns - half_width).max(theme::ATTRACT_START);
        let swept = if at < theme::ATTRACT_START || at > end {
            theme::ATTRACT_START
        } else {
            at + theme::ATTRACT_SPEED * time.delta_time
        };
        (swept, theme::GROUND_HEIGHT + theme::ATTRACT_RISE)
    } else {
        let wanted_x = target.x + drift * theme::CAMERA_LEAD;
        let wanted_y = target.y + theme::CAMERA_RISE;
        let share = (theme::CAMERA_FOLLOW * time.delta_time).clamp(0.0, 1.0);
        (
            transform.translation.x + (wanted_x - transform.translation.x) * share,
            transform.translation.y + (wanted_y - transform.translation.y) * share,
        )
    };

    if columns > half_width * 2.0 {
        eased_x = eased_x.clamp(half_width, columns - half_width);
    }
    eased_y = eased_y.max(half_height);

    let texel = 1.0 / theme::TILE_TEXELS;
    transform.translation.x = (eased_x / texel).round() * texel;
    transform.translation.y = (eased_y / texel).round() * texel;
    let settled = transform.translation.x;
    resources.camera_x = settled;
}
