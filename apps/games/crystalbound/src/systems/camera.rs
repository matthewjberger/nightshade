use crate::resources::Rig;
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub type Lenses<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item mut Camera, &'item mut LocalTransform)>;

pub fn move_the_camera(
    time: Res<Time>,
    window: Res<Window>,
    mut rig: ResMut<Rig>,
    mut lenses: Lenses<'_, '_>,
) {
    let Some(camera) = rig.camera else {
        return;
    };
    let delta_time = time.delta_time;
    rig.shake = (rig.shake - delta_time * 2.6).max(0.0);
    rig.shake_phase += delta_time;

    let (width, height) = window
        .cached_viewport_size
        .map(|(width, height)| (width as f32, height as f32))
        .unwrap_or((1280.0, 720.0));
    let wanted_magnitude = (height / theme::PIXELS_PER_UNIT / 2.0).max(1.0) / rig.zoom;

    let focus = held_inside(rig.focus, rig.bounds, wanted_magnitude, width / height);
    let snap = rig.snap;
    let share = (theme::CAMERA_FOLLOW * delta_time).clamp(0.0, 1.0);
    let wobble = if rig.shake > 0.0 {
        Vec2::new(
            (rig.shake_phase * 47.0).sin() * rig.shake,
            (rig.shake_phase * 61.0).cos() * rig.shake,
        )
    } else {
        Vec2::zeros()
    };
    let locked = snap && rig.shake <= 0.0 && (rig.zoom - 1.0).abs() < 0.01;

    lenses.for_entity(camera, |(lens, transform)| {
        let magnitude = match lens.projection {
            Projection::Orthographic(existing) => {
                existing.y_mag + (wanted_magnitude - existing.y_mag) * share
            }
            _ => wanted_magnitude,
        };
        lens.projection = Projection::Orthographic(OrthographicCamera {
            x_mag: None,
            y_mag: magnitude,
            z_near: 0.1,
            z_far: 1000.0,
        });

        let eased_x = transform.translation.x + (focus.x - transform.translation.x) * share;
        let eased_y = transform.translation.y + (focus.y - transform.translation.y) * share;
        if locked {
            let texel = 4.0 / theme::PIXELS_PER_UNIT;
            transform.translation.x = (eased_x / texel).round() * texel;
            transform.translation.y = (eased_y / texel).round() * texel;
        } else {
            transform.translation.x = eased_x + wobble.x;
            transform.translation.y = eased_y + wobble.y;
        }
        transform.translation.z = theme::CAMERA_DEPTH;
    });
}

fn held_inside(focus: Vec2, bounds: Option<(Vec2, Vec2)>, half_height: f32, aspect: f32) -> Vec2 {
    let Some((low, high)) = bounds else {
        return focus;
    };
    let half_width = half_height * aspect;
    let middle = (low + high) * 0.5;
    let held_x = if high.x - low.x <= half_width * 2.0 {
        middle.x
    } else {
        focus.x.clamp(low.x + half_width, high.x - half_width)
    };
    let held_y = if high.y - low.y <= half_height * 2.0 {
        middle.y
    } else {
        focus.y.clamp(low.y + half_height, high.y - half_height)
    };
    Vec2::new(held_x, held_y)
}

pub fn follow_the_walker(
    mut rig: ResMut<Rig>,
    field: Res<crate::resources::Field>,
    walkers: Query<(&LocalTransform,), With<crate::components::Facing>>,
) {
    let Some(entity) = rig.walker else {
        return;
    };
    let mut spot = None;
    walkers.for_each(|found, (transform,)| {
        if found == entity {
            spot = Some(Vec2::new(transform.translation.x, transform.translation.y));
        }
    });
    let Some(spot) = spot else {
        return;
    };
    let low = field.origin();
    rig.bounds = Some((
        low,
        low + Vec2::new(field.columns as f32, field.rows as f32),
    ));
    rig.focus = spot;
    rig.zoom = 1.0;
    rig.snap = true;
}
