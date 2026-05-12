use crate::ecs::{GRID_CELL, GameWorld};
use nightshade::prelude::*;

pub fn get_grid_position_from_mouse(_game_world: &GameWorld, world: &World) -> Option<(i32, i32)> {
    let mouse = &world.resources.input.mouse;
    let mouse_pos = nalgebra_glm::vec2(mouse.position.x, mouse.position.y);

    if let Some(intersection) = get_ground_position_from_screen(world, mouse_pos, 0.0) {
        let grid_x = intersection.x.round() as i32;
        let grid_z = intersection.z.round() as i32;

        if (-10..=10).contains(&grid_x) && (-10..=10).contains(&grid_z) {
            return Some((grid_x, grid_z));
        }
    }

    None
}

pub fn can_place_tower_at(game_world: &GameWorld, x: i32, z: i32) -> bool {
    if game_world.resources.path.is_empty() {
        return false;
    }
    let pos = nalgebra_glm::vec3(x as f32, 0.0, z as f32);

    let start_pos = game_world.resources.path[0];
    let start_half_size = 1.0;
    let is_start = (pos.x - start_pos.x).abs() <= start_half_size
        && (pos.z - start_pos.z).abs() <= start_half_size;

    let end_pos = game_world.resources.path.last().unwrap();
    let end_half_size = 1.0;
    let is_end =
        (pos.x - end_pos.x).abs() <= end_half_size && (pos.z - end_pos.z).abs() <= end_half_size;

    if is_start || is_end {
        return false;
    }

    if game_world
        .resources
        .towers_by_position
        .contains_key(&(x, z))
    {
        return false;
    }

    for entity in game_world.query_entities(GRID_CELL) {
        if let Some(cell) = game_world.get_grid_cell(entity)
            && cell.x == x
            && cell.z == z
        {
            return !cell.occupied;
        }
    }
    false
}

pub fn mark_cell_occupied(game_world: &mut GameWorld, x: i32, z: i32) {
    let entities: Vec<_> = game_world.query_entities(GRID_CELL).collect();
    for entity in entities {
        if let Some(cell) = game_world.get_grid_cell_mut(entity)
            && cell.x == x
            && cell.z == z
        {
            cell.occupied = true;
        }
    }
}
