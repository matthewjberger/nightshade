use crate::components::Motion;
use crate::data::ask::Ask;
use blocker_blocks::resources::VoxelViewer;
use blocker_figure::components::{Eye, Gaze};
use blocker_figure::resources::Nameplates;
use blocker_weather::resources::Overhead;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_scene::prelude::LocalTransform;

pub fn aim_the_viewer(
    mut viewer: ResMut<VoxelViewer>,
    mut overhead: ResMut<Overhead>,
    mut bodies: Query<(&Eye, &Gaze, &Motion, &LocalTransform)>,
) {
    bodies.for_each_mut(|_, (eye, gaze, motion, transform)| {
        viewer.eye = eye.0;
        viewer.forward = gaze.0;
        viewer.whole = *motion == Motion::Watch;
        overhead.0 = transform.translation;
    });
}

pub fn toggle_the_nameplates(actions: Res<Actions<Ask>>, mut nameplates: ResMut<Nameplates>) {
    if actions.just_pressed(Ask::Nameplates) {
        nameplates.shown = !nameplates.shown;
    }
}
