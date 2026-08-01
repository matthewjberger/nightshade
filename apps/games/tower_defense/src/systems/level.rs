use crate::commands::Scene;
use crate::components::{GridCoord, Occupied};
use crate::resources::TowerDefenseResources;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;

pub const TILE_TAG: &str = "td_tile";
pub const WAYPOINT_TAG: &str = "td_waypoint";
pub const START_MARKER_TAG: &str = "td_start_marker";
pub const END_MARKER_TAG: &str = "td_end_marker";

const GRID_HALF_EXTENT: i32 = 6;

const PATH_TILE_COLOR: [f32; 4] = [0.5, 0.3, 0.1, 1.0];

pub fn build_level(scene: &mut Scene<'_, '_>) {
    build_default_level(scene);
}

pub fn link_level(game_world: &mut TowerDefenseResources, scene: &mut Scene<'_, '_>) {
    link_path_from_world(game_world, scene);
    link_grid_from_world(game_world, scene);
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

fn build_default_level(scene: &mut Scene<'_, '_>) {
    spawn_sun(scene.commands);

    let waypoints = default_waypoints();
    let path_cells = compute_path_cells(&waypoints);
    let start = waypoints[0];
    let end = waypoints[waypoints.len() - 1];

    for (index, position) in waypoints.iter().enumerate() {
        spawn_waypoint(scene, index as u32, *position);
    }

    for x in -GRID_HALF_EXTENT..=GRID_HALF_EXTENT {
        for z in -GRID_HALF_EXTENT..=GRID_HALF_EXTENT {
            if covers_marker(x, z, start) || covers_marker(x, z, end) {
                continue;
            }
            let is_path = path_cells.contains(&(x, z));
            spawn_tile(scene, x, z, is_path);
        }
    }

    spawn_marker(
        scene,
        START_MARKER_TAG,
        "StartMarker",
        start,
        nalgebra_glm::vec3(1.5, 1.0, 1.5),
        [1.0, 0.5, 0.0, 1.0],
        [0.5, 0.25, 0.0],
    );
    spawn_marker(
        scene,
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

fn spawn_waypoint(scene: &mut Scene<'_, '_>, index: u32, position: Vec3) {
    let entity = scene.commands.spawn(());
    scene
        .commands
        .set(entity, Name(format!("PathWaypoint_{index}")));
    scene.commands.set(
        entity,
        LocalTransform {
            translation: position,
            ..Default::default()
        },
    );
    scene.commands.set(entity, GlobalTransform::default());
    push_tag(scene, entity, WAYPOINT_TAG);
}

fn spawn_tile(scene: &mut Scene<'_, '_>, x: i32, z: i32, is_path: bool) {
    let color = if is_path {
        PATH_TILE_COLOR
    } else {
        [0.1, 0.3, 0.1, 1.0]
    };
    let entity = {
        spawn_mesh(
            scene.commands,
            "Cube",
            nalgebra_glm::vec3(x as f32, -0.5, z as f32),
            nalgebra_glm::vec3(0.9, 0.1, 0.9),
        )
    };
    scene.commands.set(entity, Name(format!("Tile_{x}_{z}")));
    {
        set_material(
            scene.registry,
            scene.commands,
            entity,
            format!("Tile_{x}_{z}_{}", entity.id),
            Material {
                base_color: color,
                ..Default::default()
            },
        );
    };
    push_tag(scene, entity, TILE_TAG);
}

fn spawn_marker(
    scene: &mut Scene<'_, '_>,
    tag: &str,
    name: &str,
    position: Vec3,
    scale: Vec3,
    base_color: [f32; 4],
    emissive: [f32; 3],
) {
    let entity = spawn_mesh(scene.commands, "Cube", position, scale);
    scene.commands.set(entity, Name(name.to_string()));
    {
        set_material(
            scene.registry,
            scene.commands,
            entity,
            format!("{name}_{}", entity.id),
            Material {
                base_color,
                emissive_factor: emissive,
                ..Default::default()
            },
        );
    };
    push_tag(scene, entity, tag);
}

fn push_tag(scene: &mut Scene<'_, '_>, entity: Entity, tag: &str) {
    scene
        .entity_registry
        .tags
        .entry(entity)
        .or_default()
        .push(tag.to_string());
}

fn link_path_from_world(game_world: &mut TowerDefenseResources, scene: &Scene<'_, '_>) {
    let mut waypoints: Vec<(u32, Vec3)> = scene
        .entity_registry
        .tags
        .iter()
        .filter_map(|(entity, tags)| {
            if !tags.iter().any(|tag| tag == WAYPOINT_TAG) {
                return None;
            }
            let name = scene
                .storage
                .get::<nightshade_scene::contract::primitives::Name>(*entity)?;
            let index = name.0.strip_prefix("PathWaypoint_")?.parse::<u32>().ok()?;
            let transform = scene
                .storage
                .get::<nightshade_scene::contract::LocalTransform>(*entity)?;
            Some((index, transform.translation))
        })
        .collect();
    waypoints.sort_by_key(|(index, _)| *index);
    game_world.path = waypoints
        .into_iter()
        .map(|(_, position)| position)
        .collect();
}

fn link_grid_from_world(game_world: &mut TowerDefenseResources, scene: &mut Scene<'_, '_>) {
    game_world.grid_tiles.clear();
    game_world.tile_original_colors.clear();

    let tile_entries: Vec<(Entity, i32, i32, bool, Vec4)> = scene
        .entity_registry
        .tags
        .iter()
        .filter_map(|(entity, tags)| {
            if !tags.iter().any(|tag| tag == TILE_TAG) {
                return None;
            }
            let transform = scene
                .storage
                .get::<nightshade_scene::contract::LocalTransform>(*entity)?;
            let x = transform.translation.x.round() as i32;
            let z = transform.translation.z.round() as i32;
            let color = tile_color(scene, *entity);
            let is_path = (color.x - PATH_TILE_COLOR[0]).abs() < 1e-3
                && (color.y - PATH_TILE_COLOR[1]).abs() < 1e-3
                && (color.z - PATH_TILE_COLOR[2]).abs() < 1e-3;
            Some((*entity, x, z, is_path, color))
        })
        .collect();

    for (entity, x, z, is_path, color) in tile_entries {
        game_world.grid_tiles.insert((x, z), entity);
        game_world.tile_original_colors.insert((x, z), color);

        scene.commands.set(entity, GridCoord(x, z));
        scene.commands.set(entity, Occupied(is_path));
    }
}

fn tile_color(scene: &Scene<'_, '_>, entity: Entity) -> Vec4 {
    let fallback = nalgebra_glm::vec4(0.1, 0.3, 0.1, 1.0);
    let Some(material_ref) = scene
        .storage
        .get::<nightshade_render::components::material::MaterialRef>(entity)
    else {
        return fallback;
    };
    let Some(material) = nightshade_render::data::generational_registry::registry_entry(
        &scene.registry.registry,
        material_ref.id.index,
        material_ref.id.generation,
    ) else {
        return fallback;
    };
    let color = material.base_color;
    nalgebra_glm::vec4(color[0], color[1], color[2], color[3])
}
