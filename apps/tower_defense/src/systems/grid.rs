use crate::ecs::{GRID_CELL, GameWorld, GridCell};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

pub fn initialize_grid(game_world: &mut GameWorld) {
    let grid_size = 12;
    for x in -grid_size / 2..=grid_size / 2 {
        for z in -grid_size / 2..=grid_size / 2 {
            let game_entity = game_world.spawn_entities(GRID_CELL, 1)[0];
            game_world.set_grid_cell(
                game_entity,
                GridCell {
                    x,
                    z,
                    occupied: false,
                    is_path: false,
                },
            );
        }
    }
}

pub fn spawn_grid_tiles(game_world: &mut GameWorld, world: &mut World) {
    let grid_size = 12;
    for x in -grid_size / 2..=grid_size / 2 {
        for z in -grid_size / 2..=grid_size / 2 {
            let pos = nalgebra_glm::vec3(x as f32, 0.0, z as f32);
            let is_path = game_world.resources.path.windows(2).any(|w| {
                let seg_start = w[0];
                let seg_end = w[1];
                let min_x = seg_start.x.min(seg_end.x);
                let max_x = seg_start.x.max(seg_end.x);
                let min_z = seg_start.z.min(seg_end.z);
                let max_z = seg_start.z.max(seg_end.z);
                pos.x >= min_x && pos.x <= max_x && pos.z >= min_z && pos.z <= max_z
            });

            let start_pos = game_world.resources.path[0];
            let start_half_size = 1.0;
            let is_start = (pos.x - start_pos.x).abs() <= start_half_size
                && (pos.z - start_pos.z).abs() <= start_half_size;

            let end_pos = game_world.resources.path.last().unwrap();
            let end_half_size = 1.0;
            let is_end = (pos.x - end_pos.x).abs() <= end_half_size
                && (pos.z - end_pos.z).abs() <= end_half_size;

            if is_start || is_end {
                continue;
            }

            let color = if is_path {
                nalgebra_glm::vec4(0.5, 0.3, 0.1, 1.0)
            } else {
                nalgebra_glm::vec4(0.1, 0.3, 0.1, 1.0)
            };

            let tile_entity = spawn_mesh(
                world,
                "Cube",
                nalgebra_glm::vec3(x as f32, -0.5, z as f32),
                nalgebra_glm::vec3(0.9, 0.1, 0.9),
            );

            let material_name = format!("GridTile_{}", tile_entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: color.into(),
                    ..Default::default()
                },
            );
            if let Some(&index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(index);
            }
            world.set_material_ref(tile_entity, MaterialRef::new(material_name));

            game_world.resources.grid_tiles.insert((x, z), tile_entity);
            game_world
                .resources
                .tile_original_colors
                .insert((x, z), color);
        }
    }
}

pub fn create_path(game_world: &mut GameWorld, world: &mut World) {
    let path = vec![
        nalgebra_glm::vec3(-6.0, 0.0, 0.0),
        nalgebra_glm::vec3(-3.0, 0.0, 0.0),
        nalgebra_glm::vec3(-3.0, 0.0, -4.0),
        nalgebra_glm::vec3(3.0, 0.0, -4.0),
        nalgebra_glm::vec3(3.0, 0.0, 2.0),
        nalgebra_glm::vec3(-1.0, 0.0, 2.0),
        nalgebra_glm::vec3(-1.0, 0.0, 5.0),
        nalgebra_glm::vec3(6.0, 0.0, 5.0),
    ];

    game_world.resources.path = path.clone();

    for index in 0..path.len() - 1 {
        let start = path[index];
        let end = path[index + 1];
        let steps = 20;

        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let pos = start + (end - start) * t;
            let grid_x = pos.x.round() as i32;
            let grid_z = pos.z.round() as i32;

            let entities: Vec<_> = game_world.query_entities(GRID_CELL).collect();
            for entity in entities {
                if let Some(cell) = game_world.get_grid_cell_mut(entity)
                    && cell.x == grid_x
                    && cell.z == grid_z
                {
                    cell.is_path = true;
                    cell.occupied = true;
                }
            }
        }
    }

    for index in 0..path.len() - 1 {
        let start = path[index];
        let end = path[index + 1];
        let steps = 10;

        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let pos = start + (end - start) * t;

            let is_at_start = (pos - path[0]).magnitude() < 0.5;
            let is_at_end = (pos - path[path.len() - 1]).magnitude() < 0.5;

            if is_at_start || is_at_end {
                continue;
            }

            let path_segment = spawn_mesh(
                world,
                "Cube",
                pos + nalgebra_glm::vec3(0.0, -0.5, 0.0),
                nalgebra_glm::vec3(0.9, 0.1, 0.9),
            );

            let material_name = format!("PathSegment_{}", path_segment.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.5, 0.3, 0.1, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(index);
            }
            world.set_material_ref(path_segment, MaterialRef::new(material_name));
        }
    }

    let start_marker = spawn_mesh(
        world,
        "Cube",
        path[0] + nalgebra_glm::vec3(0.0, 0.0, 0.0),
        nalgebra_glm::vec3(1.5, 1.0, 1.5),
    );

    let material_name = format!("StartMarker_{}", start_marker.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: [1.0, 0.5, 0.0, 1.0],
            emissive_factor: [0.5, 0.25, 0.0],
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(start_marker, MaterialRef::new(material_name));

    let end_marker = spawn_mesh(
        world,
        "Cube",
        path[path.len() - 1] + nalgebra_glm::vec3(0.0, 0.25, 0.0),
        nalgebra_glm::vec3(2.0, 1.5, 2.0),
    );

    let material_name = format!("EndMarker_{}", end_marker.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: [0.2, 0.2, 0.8, 1.0],
            emissive_factor: [0.1, 0.1, 0.4],
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(end_marker, MaterialRef::new(material_name));
}

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
