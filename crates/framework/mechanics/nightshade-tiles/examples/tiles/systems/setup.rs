use crate::commands::slab;
use crate::resources::Field;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_cells::Cells;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_tiles::OpenTile;
use nightshade_tiles::commands::answer_the_cells;
use nightshade_tiles::queries::read;
use nightshade_tiles::queries::{at, cells_of};

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut field: ResMut<Field>,
    mut cells_held: ResMut<Cells>,
    mut commands: Commands,
) {
    let camera = {
        let commands = &mut commands;

        spawn_pan_orbit_camera_at(
            commands,
            theme::CAMERA_TARGET,
            theme::CAMERA_BACK,
            theme::CAMERA_YAW,
            theme::CAMERA_PITCH,
            "Camera".to_string(),
        )
    };
    active_camera.0 = Some(camera);

    let map = read(&theme::MAP);
    let width = map.width as f32;
    let height = map.height as f32;
    let mut cells = Vec::new();
    let cursor;
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        for down in 0..map.height {
            for across in 0..map.width {
                let byte = at(&map, nalgebra_glm::IVec2::new(across as i32, down as i32));
                let wall = byte == theme::WALL;
                let spot = vec3(
                    (across as f32 - width * 0.5) * theme::STEP,
                    if wall { 0.6 } else { 0.1 },
                    (down as f32 - height * 0.5) * theme::STEP,
                );
                let tile = slab(
                    commands,
                    materials,
                    spot,
                    vec3(theme::STEP, if wall { 1.2 } else { 0.2 }, theme::STEP),
                    if wall {
                        theme::WALL_TINT
                    } else {
                        theme::OPEN_TINT
                    },
                );
                cells.push((nalgebra_glm::IVec2::new(across as i32, down as i32), tile));
            }
        }
        cursor = Some(slab(
            commands,
            materials,
            vec3(0.0, theme::CURSOR_HEIGHT, 0.0),
            vec3(theme::STEP * 0.5, theme::STEP * 0.5, theme::STEP * 0.5),
            theme::CURSOR_TINT,
        ));
        commands.spawn((map.clone(), OpenTile(theme::OPEN)));
        spawn_sun(commands);
    };
    let start = cells_of(&map, theme::OPEN)
        .first()
        .copied()
        .unwrap_or_default();
    field.cells = cells;
    field.cursor = cursor;
    field.at = start;
    answer_the_cells(&mut cells_held);
}
