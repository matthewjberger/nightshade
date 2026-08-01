use crate::commands::Scene;
use crate::components::{GridCoord, Occupied};
use crate::resources::TowerDefenseResources;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct MouseView<'a> {
    pub window: &'a nightshade_platform::contract::window::Window,
    pub viewport: &'a nightshade_render::resources::viewport::Viewport,
    pub active_camera: &'a nightshade_camera::resources::ActiveCamera,
}

pub fn get_grid_position_from_mouse(
    input: &Input,
    view: &MouseView<'_>,
    scene: &Scene<'_, '_>,
) -> Option<(i32, i32)> {
    let mouse = &input.mouse;
    let mouse_pos = nalgebra_glm::vec2(mouse.position.x, mouse.position.y);

    let intersection = nightshade_picking::screen_ray(
        view.window,
        view.viewport,
        view.active_camera,
        scene.storage.query_ref(),
        mouse_pos,
    )
    .and_then(|ray| ray.intersect_ground_plane(0.0));
    if let Some(intersection) = intersection {
        let grid_x = intersection.x.round() as i32;
        let grid_z = intersection.z.round() as i32;

        if (-10..=10).contains(&grid_x) && (-10..=10).contains(&grid_z) {
            return Some((grid_x, grid_z));
        }
    }

    None
}

pub fn can_place_tower_at(
    game_world: &TowerDefenseResources,
    scene: &Scene<'_, '_>,
    x: i32,
    z: i32,
) -> bool {
    if game_world.path.is_empty() {
        return false;
    }
    let pos = nalgebra_glm::vec3(x as f32, 0.0, z as f32);

    let start_pos = game_world.path[0];
    let start_half_size = 1.0;
    let is_start = (pos.x - start_pos.x).abs() <= start_half_size
        && (pos.z - start_pos.z).abs() <= start_half_size;

    let end_pos = game_world.path.last().unwrap();
    let end_half_size = 1.0;
    let is_end =
        (pos.x - end_pos.x).abs() <= end_half_size && (pos.z - end_pos.z).abs() <= end_half_size;

    if is_start || is_end {
        return false;
    }

    if game_world.towers_by_position.contains_key(&(x, z)) {
        return false;
    }

    for (entity, coord) in scene.storage.query_ref::<&GridCoord>().iter() {
        if *coord == GridCoord(x, z) {
            let occupied = scene
                .storage
                .get::<Occupied>(entity)
                .copied()
                .unwrap_or_default();
            return !occupied.0;
        }
    }
    false
}

pub fn mark_cell_occupied(scene: &mut Scene<'_, '_>, x: i32, z: i32) {
    let entities: Vec<Entity> = scene.storage.query_entities::<(GridCoord,)>().collect();
    for entity in entities {
        if scene.storage.get::<GridCoord>(entity).copied() == Some(GridCoord(x, z)) {
            scene.storage.set(entity, Occupied(true));
        }
    }
}
