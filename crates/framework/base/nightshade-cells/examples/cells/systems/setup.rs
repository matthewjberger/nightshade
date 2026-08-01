use crate::commands::slab;
use crate::queries::{cost_here, in_bounds_here, walkable_here};
use crate::resources::Walk;
use crate::theme;
use nalgebra_glm::{IVec2, vec3};
use nightshade::prelude::*;
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_cells::CellSize;
use nightshade_cells::Cells;
use nightshade_cells::commands::{
    answer_in_bounds_with, answer_step_cost_with, answer_walkable_with,
};
use nightshade_cells::queries::center_of;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut cells: ResMut<Cells>,
    size: Res<CellSize>,
    mut walk: ResMut<Walk>,
    mut active_camera: ResMut<ActiveCamera>,
    mut materials: ResMut<MaterialRegistry>,
    mut commands: Commands,
) {
    answer_walkable_with(&mut cells, walkable_here);
    answer_in_bounds_with(&mut cells, in_bounds_here);
    answer_step_cost_with(&mut cells, cost_here);

    let camera = spawn_pan_orbit_camera_at(
        &mut commands,
        theme::CAMERA_TARGET,
        theme::CAMERA_BACK,
        theme::CAMERA_YAW,
        theme::CAMERA_PITCH,
        "Camera".to_string(),
    );
    active_camera.0 = Some(camera);

    let half = theme::SIDE / 2;
    let width = size.0;
    for down in -half..=half {
        for across in -half..=half {
            let cell = IVec2::new(across, down);
            let middle = center_of(&size, cell);
            let open = crate::queries::open(cell);
            slab(
                &mut commands,
                &mut materials,
                vec3(middle.x, if open { 0.1 } else { 0.7 }, middle.y),
                vec3(width, if open { 0.2 } else { 1.4 }, width),
                if open {
                    theme::OPEN_TINT
                } else {
                    theme::WALL_TINT
                },
            );
        }
    }

    let start = center_of(&size, theme::START);
    let marker = slab(
        &mut commands,
        &mut materials,
        vec3(start.x, 0.3, start.y),
        vec3(width * 0.5, 0.2, width * 0.5),
        theme::MARK_TINT,
    );
    let walker = slab(
        &mut commands,
        &mut materials,
        vec3(start.x, 0.8, start.y),
        vec3(0.9, 0.9, 0.9),
        theme::WALKER_TINT,
    );
    spawn_sun(&mut commands);

    walk.walker = Some(walker);
    walk.marker = Some(marker);
    walk.goal = theme::START;
}
