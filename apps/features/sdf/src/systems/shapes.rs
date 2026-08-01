use crate::components::{Axis, Bob, Drift, Height, Orbit, Phase, Spin};
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub type Movers<'world, 'item> = Query<
    'world,
    (
        &'item LocalTransform,
        Option<&'item Orbit>,
        Option<&'item Phase>,
        Option<&'item Drift>,
        Option<&'item Bob>,
        Option<&'item Height>,
        Option<&'item Spin>,
        Option<&'item Axis>,
    ),
>;

pub fn pose(time: Res<Time>, mut commands: Commands, movers: Movers) {
    let elapsed = time.uptime_milliseconds as f32 / 1000.0;
    movers.for_each(
        |entity, (transform, orbit, phase, drift, bob, height, spin, axis)| {
            if orbit.is_none() && spin.is_none() {
                return;
            }
            let start = phase.map_or(0.0, |phase| phase.0);
            let mut posed = *transform;
            if let (Some(orbit), Some(drift), Some(bob), Some(height)) = (orbit, drift, bob, height)
            {
                let angle = start + drift.0 * elapsed;
                posed.translation = Vec3::new(
                    orbit.0 * angle.cos(),
                    height.0 + bob.0 * (angle * 1.7 + start).sin(),
                    orbit.0 * angle.sin(),
                );
            }
            if let (Some(spin), Some(axis)) = (spin, axis) {
                posed.rotation = nalgebra_glm::quat_angle_axis(start + spin.0 * elapsed, &axis.0);
            }
            commands.set(entity, posed);
        },
    );
}
