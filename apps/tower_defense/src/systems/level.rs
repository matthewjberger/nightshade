use crate::ecs::{GridCell, TowerDefenseResources};
use nightshade::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
pub const LEVEL_PATH: &str = "apps/tower_defense/tower_defense.nsmap";

pub const TILE_TAG: &str = "td_tile";
pub const WAYPOINT_TAG: &str = "td_waypoint";
pub const START_MARKER_TAG: &str = "td_start_marker";
pub const END_MARKER_TAG: &str = "td_end_marker";

const GRID_HALF_EXTENT: i32 = 6;

const PATH_TILE_COLOR: [f32; 4] = [0.5, 0.3, 0.1, 1.0];

#[cfg(not(target_arch = "wasm32"))]
pub fn load_or_build_level(game_world: &mut TowerDefenseResources, world: &mut World) {
    use nightshade::ecs::scene::commands::{load_scene, spawn_scene};
    use std::path::Path;

    let path = Path::new(LEVEL_PATH);
    if path.exists() {
        match load_scene(path) {
            Ok(scene) => match spawn_scene(world, &scene, None) {
                Ok(_) => {}
                Err(error) => {
                    tracing::error!("failed to spawn scene from {LEVEL_PATH}: {error}");
                    build_default_level(world);
                    save_current_world_as_level(world);
                }
            },
            Err(error) => {
                tracing::error!("failed to load {LEVEL_PATH}: {error}; rebuilding");
                build_default_level(world);
                save_current_world_as_level(world);
            }
        }
    } else {
        tracing::info!("no level file at {LEVEL_PATH}, building default");
        build_default_level(world);
        save_current_world_as_level(world);
    }

    link_path_from_world(game_world, world);
    link_grid_from_world(game_world, world);
}

#[cfg(target_arch = "wasm32")]
pub fn load_or_build_level(game_world: &mut TowerDefenseResources, world: &mut World) {
    build_default_level(world);
    link_path_from_world(game_world, world);
    link_grid_from_world(game_world, world);
}

#[cfg(not(target_arch = "wasm32"))]
fn save_current_world_as_level(world: &World) {
    use nightshade::ecs::scene::commands::{save_scene, world_to_scene};
    use std::path::Path;

    let target = Path::new(LEVEL_PATH);
    if let Some(parent) = target.parent()
        && !parent.as_os_str().is_empty()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        tracing::error!("failed to create {}: {error}", parent.display());
        return;
    }
    let mut scene = world_to_scene(world, "TowerDefense");
    if let Err(error) = save_scene(&mut scene, target) {
        tracing::error!("failed to save {LEVEL_PATH}: {error}");
    } else {
        tracing::info!("wrote level to {LEVEL_PATH}");
    }
}

fn default_waypoints() -> Vec<Vec3> {
    vec![
        nalgebra_glm::vec3(-6.0, 0.0, 0.0),
        nalgebra_glm::vec3(-3.0, 0.0, 0.0),
        nalgebra_glm::vec3(-3.0, 0.0, -4.0),
        nalgebra_glm::vec3(3.0, 0.0, -4.0),
        nalgebra_glm::vec3(3.0, 0.0, 2.0),
        nalgebra_glm::vec3(-1.0, 0.0, 2.0),
        nalgebra_glm::vec3(-1.0, 0.0, 5.0),
        nalgebra_glm::vec3(6.0, 0.0, 5.0),
    ]
}

fn build_default_level(world: &mut World) {
    spawn_sun(world);

    let waypoints = default_waypoints();
    let path_cells = compute_path_cells(&waypoints);
    let start = waypoints[0];
    let end = waypoints[waypoints.len() - 1];

    for (index, position) in waypoints.iter().enumerate() {
        spawn_waypoint(world, index as u32, *position);
    }

    for x in -GRID_HALF_EXTENT..=GRID_HALF_EXTENT {
        for z in -GRID_HALF_EXTENT..=GRID_HALF_EXTENT {
            if covers_marker(x, z, start) || covers_marker(x, z, end) {
                continue;
            }
            let is_path = path_cells.contains(&(x, z));
            spawn_tile(world, x, z, is_path);
        }
    }

    spawn_marker(
        world,
        START_MARKER_TAG,
        "StartMarker",
        start,
        nalgebra_glm::vec3(1.5, 1.0, 1.5),
        [1.0, 0.5, 0.0, 1.0],
        [0.5, 0.25, 0.0],
    );
    spawn_marker(
        world,
        END_MARKER_TAG,
        "EndMarker",
        end + nalgebra_glm::vec3(0.0, 0.25, 0.0),
        nalgebra_glm::vec3(2.0, 1.5, 2.0),
        [0.2, 0.2, 0.8, 1.0],
        [0.1, 0.1, 0.4],
    );
}

fn covers_marker(grid_x: i32, grid_z: i32, marker: Vec3) -> bool {
    let pos = nalgebra_glm::vec3(grid_x as f32, 0.0, grid_z as f32);
    (pos.x - marker.x).abs() <= 1.0 && (pos.z - marker.z).abs() <= 1.0
}

fn compute_path_cells(waypoints: &[Vec3]) -> std::collections::HashSet<(i32, i32)> {
    use std::collections::HashSet;
    let mut cells: HashSet<(i32, i32)> = HashSet::new();
    for window in waypoints.windows(2) {
        let start = window[0];
        let end = window[1];
        let steps = 20;
        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let pos = start + (end - start) * t;
            cells.insert((pos.x.round() as i32, pos.z.round() as i32));
        }
    }
    cells
}

fn spawn_waypoint(world: &mut World, index: u32, position: Vec3) {
    let entity = spawn_entities(world, NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM, 1)[0];
    world.set(entity, Name(format!("PathWaypoint_{index}")));
    world.set(
        entity,
        LocalTransform {
            translation: position,
            ..Default::default()
        },
    );
    world.set(entity, GlobalTransform::default());
    push_tag(world, entity, WAYPOINT_TAG);
}

fn spawn_tile(world: &mut World, x: i32, z: i32, is_path: bool) {
    let color = if is_path {
        PATH_TILE_COLOR
    } else {
        [0.1, 0.3, 0.1, 1.0]
    };
    let entity = spawn_mesh(
        world,
        "Cube",
        nalgebra_glm::vec3(x as f32, -0.5, z as f32),
        nalgebra_glm::vec3(0.9, 0.1, 0.9),
    );
    world.set(entity, Name(format!("Tile_{x}_{z}")));
    set_material(
        world,
        entity,
        format!("Tile_{x}_{z}_{}", entity.id),
        Material {
            base_color: color,
            ..Default::default()
        },
    );
    push_tag(world, entity, TILE_TAG);
}

fn spawn_marker(
    world: &mut World,
    tag: &str,
    name: &str,
    position: Vec3,
    scale: Vec3,
    base_color: [f32; 4],
    emissive: [f32; 3],
) {
    let entity = spawn_mesh(world, "Cube", position, scale);
    world.set(entity, Name(name.to_string()));
    set_material(
        world,
        entity,
        format!("{name}_{}", entity.id),
        Material {
            base_color,
            emissive_factor: emissive,
            ..Default::default()
        },
    );
    push_tag(world, entity, tag);
}

fn push_tag(world: &mut World, entity: Entity, tag: &str) {
    world
        .expect_resource_mut::<EntityRegistry>()
        .tags
        .entry(entity)
        .or_default()
        .push(tag.to_string());
}

fn link_path_from_world(game_world: &mut TowerDefenseResources, world: &World) {
    let mut waypoints: Vec<(u32, Vec3)> = world
        .expect_resource::<EntityRegistry>()
        .tags
        .iter()
        .filter_map(|(entity, tags)| {
            if !tags.iter().any(|tag| tag == WAYPOINT_TAG) {
                return None;
            }
            let name = world.get::<nightshade::ecs::primitives::Name>(*entity)?;
            let index = name.0.strip_prefix("PathWaypoint_")?.parse::<u32>().ok()?;
            let transform =
                world.get::<nightshade::ecs::transform::components::LocalTransform>(*entity)?;
            Some((index, transform.translation))
        })
        .collect();
    waypoints.sort_by_key(|(index, _)| *index);
    game_world.path = waypoints
        .into_iter()
        .map(|(_, position)| position)
        .collect();
}

fn link_grid_from_world(game_world: &mut TowerDefenseResources, world: &mut World) {
    game_world.grid_tiles.clear();
    game_world.tile_original_colors.clear();

    let tile_entries: Vec<(Entity, i32, i32, bool, Vec4)> = world
        .expect_resource::<EntityRegistry>()
        .tags
        .iter()
        .filter_map(|(entity, tags)| {
            if !tags.iter().any(|tag| tag == TILE_TAG) {
                return None;
            }
            let transform =
                world.get::<nightshade::ecs::transform::components::LocalTransform>(*entity)?;
            let x = transform.translation.x.round() as i32;
            let z = transform.translation.z.round() as i32;
            let color = tile_color(world, *entity);
            let is_path = (color.x - PATH_TILE_COLOR[0]).abs() < 1e-3
                && (color.y - PATH_TILE_COLOR[1]).abs() < 1e-3
                && (color.z - PATH_TILE_COLOR[2]).abs() < 1e-3;
            Some((*entity, x, z, is_path, color))
        })
        .collect();

    for (entity, x, z, is_path, color) in tile_entries {
        game_world.grid_tiles.insert((x, z), entity);
        game_world.tile_original_colors.insert((x, z), color);

        world.set(
            entity,
            GridCell {
                x,
                z,
                occupied: is_path,
            },
        );
    }
}

fn tile_color(world: &World, entity: Entity) -> Vec4 {
    let fallback = nalgebra_glm::vec4(0.1, 0.3, 0.1, 1.0);
    let Some(material_ref) =
        world.get::<nightshade::ecs::material::components::MaterialRef>(entity)
    else {
        return fallback;
    };
    let Some(material) = nightshade::render::generational_registry::registry_entry_by_name(
        &world
            .expect_resource::<AssetState>()
            .material_registry
            .registry,
        &material_ref.name,
    ) else {
        return fallback;
    };
    let color = material.base_color;
    nalgebra_glm::vec4(color[0], color[1], color[2], color[3])
}
