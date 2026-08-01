use crate::commands::slab;
use crate::queries::{height_of, hill_height, solid_here};
use crate::resources::Probes;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::prelude::*;
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_ground::Ground;
use nightshade_ground::commands::{answer_height_with, answer_solid_with};
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut ground: ResMut<Ground>,
    mut probes: ResMut<Probes>,
    mut active_camera: ResMut<ActiveCamera>,
    mut materials: ResMut<MaterialRegistry>,
    mut commands: Commands,
) {
    answer_height_with(&mut ground, hill_height);
    answer_solid_with(&mut ground, solid_here);

    let camera = spawn_pan_orbit_camera_at(
        &mut commands,
        theme::CAMERA_TARGET,
        theme::CAMERA_BACK,
        theme::CAMERA_YAW,
        theme::CAMERA_PITCH,
        "Camera".to_string(),
    );
    active_camera.0 = Some(camera);

    let half = theme::COLUMNS / 2;
    for down in -half..half {
        for across in -half..half {
            let spot_across = across as f32 * theme::STEP;
            let spot_along = down as f32 * theme::STEP;
            let high = height_of(spot_across, spot_along);
            slab(
                &mut commands,
                &mut materials,
                vec3(spot_across, high * 0.5, spot_along),
                vec3(theme::STEP, high.max(0.2), theme::STEP),
                theme::HILL_TINT,
            );
        }
    }
    slab(
        &mut commands,
        &mut materials,
        vec3(0.0, theme::BLOCK, 0.0),
        vec3(theme::BLOCK * 2.0, theme::BLOCK * 2.0, theme::BLOCK * 2.0),
        theme::SOLID_TINT,
    );

    let mut standing = Vec::new();
    for step in 0..theme::PROBES {
        let angle = step as f32 / theme::PROBES as f32 * std::f32::consts::TAU;
        standing.push(slab(
            &mut commands,
            &mut materials,
            vec3(angle.cos() * theme::RING, 6.0, angle.sin() * theme::RING),
            vec3(0.5, 0.5, 0.5),
            theme::RESTING_TINT,
        ));
    }
    let walker = slab(
        &mut commands,
        &mut materials,
        vec3(0.0, 6.0, theme::RING * 0.6),
        vec3(0.8, 0.8, 0.8),
        theme::PROBE_TINT,
    );
    standing.push(walker);
    spawn_sun(&mut commands);

    probes.probes = standing;
    probes.walker = Some(walker);
}
