use crate::components::Marker;
use crate::data::{SculptJob, Stage};
use crate::resources::Gallery;
use crate::theme;
use nalgebra_glm::{Vec3, quat_angle_axis};
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;
use nightshade_worker::{Workers, workers_waiting};
use std::collections::HashSet;

pub fn show_the_hands(
    time: Res<Time>,
    gallery: Res<Gallery>,
    crew: Res<Workers<SculptJob>>,
    mut markers: Query<(&Marker, &mut LocalTransform)>,
) {
    let queued: HashSet<usize> = workers_waiting(&crew).into_iter().collect();
    let angle = time.uptime_seconds as f32 * theme::MARKER_SPIN;
    let spin = quat_angle_axis(angle, &Vec3::new(0.0, 1.0, 0.0));
    markers.for_each_mut(|_entity, (marker, transform)| {
        let held = gallery.stage[marker.slot] == Stage::Asked && !queued.contains(&marker.slot);
        transform.scale = if held {
            Vec3::new(1.0, 1.0, 1.0)
        } else {
            Vec3::zeros()
        };
        transform.rotation = spin;
    });
}
