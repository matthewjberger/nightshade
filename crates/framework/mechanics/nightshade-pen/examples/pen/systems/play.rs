use crate::resources::Yard;
use crate::theme;
use nalgebra_glm::{Vec3, vec2};
use nightshade::prelude::*;
use nightshade_pen::Pen;
use nightshade_pen::queries::inside;
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    time: Res<Time>,
    yard: Res<Yard>,
    mut bodies: Query<(&mut LocalTransform,)>,
) {
    let mut shove = Vec3::zeros();
    if input.keyboard.is_key_pressed(KeyCode::ArrowUp) {
        shove.z -= 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowDown) {
        shove.z += 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowLeft) {
        shove.x -= 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowRight) {
        shove.x += 1.0;
    }
    let home = input.keyboard.just_pressed(KeyCode::KeyR);
    if shove == Vec3::zeros() && !home {
        return;
    }
    let step = if shove == Vec3::zeros() {
        Vec3::zeros()
    } else {
        shove.normalize() * theme::SHOVE * time.delta_time
    };
    for body in [yard.walker, yard.free] {
        let Some(body) = body else {
            continue;
        };
        bodies.for_entity(body, |(transform,)| {
            if home {
                transform.translation.x = 0.0;
                transform.translation.z = 0.0;
                return;
            }
            transform.translation += step;
        });
    }
}

pub fn show_the_pen(
    yard: Res<Yard>,
    mut readout: ResMut<Readout>,
    mut penned: Query<(&LocalTransform, &Pen)>,
    mut loose: Query<(&LocalTransform,)>,
) {
    let mut asked = None;
    if let Some(free) = yard.free {
        loose.for_entity(free, |(transform,)| {
            asked = Some(vec2(transform.translation.x, transform.translation.z));
        });
    }
    let mut spot = None;
    let mut fence = None;
    if let Some(walker) = yard.walker {
        penned.for_entity(walker, |(transform, pen)| {
            spot = Some(vec2(transform.translation.x, transform.translation.z));
            fence = Some(pen.0);
        });
    }

    readout.notes.clear();
    if let Some(fence) = fence {
        readout
            .notes
            .push(format!("a pen of {:.0} by {:.0}", fence.x, fence.y));
    }
    if let Some(asked) = asked {
        readout
            .notes
            .push(format!("asked for {:.1}, {:.1}", asked.x, asked.y));
    }
    if let Some(spot) = spot {
        readout
            .notes
            .push(format!("held at {:.1}, {:.1}", spot.x, spot.y));
    }
    if let Some(asked) = asked
        && let Some(fence) = fence
    {
        readout.notes.push(if inside(fence, asked) {
            "inside the pen".to_string()
        } else {
            "past the wall".to_string()
        });
    }
}
