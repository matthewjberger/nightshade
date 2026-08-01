use crate::contract::WorldSide;
use crate::data;
use crate::queries;
use crate::resources::{MashupResources, Phase};
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_portal::prelude::{Portal, PortalPlane};
use nightshade_portal::queries::carry_matrix;
use nightshade_scene::prelude::LocalTransform;

pub fn update(
    time: Res<Time>,
    mut resources: ResMut<MashupResources>,
    portals: Query<(&Portal, &WorldSide)>,
) {
    let mut placements: [Option<PortalPlane>; 2] = [None; 2];
    portals.for_each(|_entity, (portal, side)| {
        placements[side.index()] = Some(portal.0);
    });
    let (Some(meadow_gate), Some(quarry_gate)) = (placements[0], placements[1]) else {
        return;
    };

    let delta_time = time.delta_time.min(0.05);
    resources.crossing.hold = (resources.crossing.hold - delta_time).max(0.0);

    match resources.crossing.phase {
        Phase::Idle => {
            let (eye, look) = settled_pose(&resources);
            resources.view.eye = eye;
            resources.view.look = look;
            resources.view.field_of_view = queries::world_field_of_view(resources.here);
            if resources.crossing.hold > 0.0 {
                return;
            }
            let near = match resources.here {
                WorldSide::Meadow => meadow_gate,
                WorldSide::Quarry => quarry_gate,
            };
            let (point, heading) = queries::gate_point(&resources);
            if !queries::reached_portal(point, heading, &near) {
                return;
            }
            let (to_eye, to_look) = queries::dive_target(&near);
            let toward = resources.here.other();
            crate::commands::begin_dive(&mut resources, eye, look, to_eye, to_look, toward);
        }
        Phase::Diving => {
            resources.crossing.elapsed += delta_time;
            let share = (resources.crossing.elapsed / data::DIVE_SECONDS).clamp(0.0, 1.0);
            let eased = queries::eased(share);
            let eye = resources.crossing.from_eye
                + (resources.crossing.to_eye - resources.crossing.from_eye) * eased;
            let look = nalgebra_glm::quat_slerp(
                &resources.crossing.from_look,
                &resources.crossing.to_look,
                eased,
            );
            resources.view.eye = eye;
            resources.view.look = look;
            let toward = resources.here.other();
            resources.view.field_of_view = blend_view(resources.here, toward, eased);
            if share < 1.0 {
                return;
            }
            let (source, destination) = match resources.here {
                WorldSide::Meadow => (meadow_gate, quarry_gate),
                WorldSide::Quarry => (quarry_gate, meadow_gate),
            };
            let carried = carry_matrix(&source, &destination) * queries::pose_matrix(eye, look);
            let (arrival_eye, arrival_look) = queries::matrix_pose(&carried);
            let landing = resources.here.other();
            match landing {
                WorldSide::Meadow => crate::commands::arrive_in_meadow(
                    &mut resources,
                    arrival_eye,
                    arrival_look,
                    destination.normal.normalize(),
                ),
                WorldSide::Quarry => {
                    let ground = queries::standing_height(arrival_eye);
                    crate::commands::arrive_in_quarry(
                        &mut resources,
                        arrival_eye,
                        arrival_look,
                        ground,
                    )
                }
            }
            resources.view.eye = arrival_eye;
            resources.view.look = arrival_look;
            resources.view.field_of_view = queries::world_field_of_view(resources.here);
            let (wanted_eye, _) = settled_pose(&resources);
            crate::commands::begin_settle(&mut resources, arrival_eye, arrival_look, wanted_eye);
        }
        Phase::Settling => {
            resources.crossing.elapsed += delta_time;
            let span = resources.crossing.settle_time.max(0.01);
            let share = (resources.crossing.elapsed / span).clamp(0.0, 1.0);
            let eased = queries::eased(share);
            let (wanted_eye, wanted_look) = settled_pose(&resources);
            resources.view.eye =
                resources.crossing.from_eye + (wanted_eye - resources.crossing.from_eye) * eased;
            resources.view.look =
                nalgebra_glm::quat_slerp(&resources.crossing.from_look, &wanted_look, eased);
            resources.view.field_of_view = queries::world_field_of_view(resources.here);
            if share >= 1.0 {
                crate::commands::finish_crossing(&mut resources);
            }
        }
    }
}

pub fn place_camera(resources: Res<MashupResources>, mut commands: Commands) {
    if resources.camera == Entity::default() {
        return;
    }
    commands.set(
        resources.camera,
        LocalTransform {
            translation: resources.view.eye,
            rotation: resources.view.look,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(
        resources.camera,
        nightshade_camera::prelude::Camera {
            projection: nightshade_camera::prelude::Projection::Perspective(
                nightshade_camera::prelude::PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: resources.view.field_of_view,
                    z_far: Some(data::FAR_PLANE),
                    z_near: data::NEAR_PLANE,
                },
            ),
            smoothing: None,
        },
    );
}

fn blend_view(from: WorldSide, to: WorldSide, share: f32) -> f32 {
    let start = queries::world_field_of_view(from);
    let end = queries::world_field_of_view(to);
    start + (end - start) * share
}

fn settled_pose(resources: &MashupResources) -> (Vec3, Quat) {
    match resources.here {
        WorldSide::Meadow => queries::meadow_eye(&resources.meadow),
        WorldSide::Quarry => queries::quarry_eye(&resources.quarry),
    }
}
