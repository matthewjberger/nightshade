use crate::queries::terrain::{
    center, height, in_hollow_mouth, in_house, index, is_bridge, is_path, is_rock, is_water,
    neighbors, shaped_height,
};
use crate::resources::Terrain;
use crate::theme;
use nalgebra_glm::{Quat, Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::{
    spawn_instanced_mesh_with_material, spawn_mesh_with_material,
};
use nightshade_render::components::mesh::InstanceTransform;

pub fn build(terrain: &mut Terrain) {
    terrain.columns = theme::TERRAIN_COLUMNS;
    terrain.rows = theme::TERRAIN_ROWS;
    terrain.origin = theme::TERRAIN_ORIGIN;
    let cells: Vec<(u32, u32)> = (0..terrain.rows)
        .flat_map(|row| (0..terrain.columns).map(move |column| (column, row)))
        .collect();
    let shaped: Vec<(u32, bool)> = cells
        .iter()
        .map(|(column, row)| shaped_height(*column, *row))
        .collect();
    terrain.heights = shaped.iter().map(|(height, _)| *height).collect();
    terrain.bridge = shaped.iter().map(|(_, bridge)| *bridge).collect();
    terrain.occupied = cells
        .iter()
        .map(|(column, row)| in_house(*column, *row) || in_hollow_mouth(*column, *row))
        .collect();
    terrain.walkable = cells
        .iter()
        .map(|(column, row)| {
            height(terrain, *column, *row) > theme::MIN_HEIGHT && !in_house(*column, *row)
        })
        .collect();
}

pub fn occupy(terrain: &mut Terrain, column: u32, row: u32) {
    let at = index(terrain, column, row);
    terrain.occupied[at] = true;
}

pub fn free(terrain: &mut Terrain, column: u32, row: u32) {
    let at = index(terrain, column, row);
    terrain.occupied[at] = false;
}

pub fn spawn_blocks(commands: &mut Commands, terrain: &Terrain) {
    let mut grass = Vec::new();
    let mut sand = Vec::new();
    let mut stone = Vec::new();
    let mut rock = Vec::new();
    let mut plank = Vec::new();
    let mut dirt = Vec::new();
    let cap = theme::CAP_THICKNESS;

    for row in 0..terrain.rows {
        for column in 0..terrain.columns {
            let column_height = height(terrain, column, row);
            let middle = center(terrain, column, row);
            let top = column_height as f32;
            let cap_instance = InstanceTransform::new(
                Vec3::new(middle.x, top - cap / 2.0, middle.y),
                Quat::identity(),
                Vec3::new(1.0, cap, 1.0),
            );
            if is_bridge(terrain, column, row) {
                plank.push(cap_instance);
                continue;
            }
            if is_water(terrain, column, row) {
                sand.push(cap_instance);
            } else if is_rock(column, row) {
                rock.push(cap_instance);
            } else if is_path(terrain, column, row) {
                stone.push(cap_instance);
            } else {
                grass.push(cap_instance);
            }

            let lowest_neighbor = neighbors(terrain, column, row)
                .into_iter()
                .map(|neighbor| neighbor.map_or(0, |(c, r)| height(terrain, c, r)))
                .min()
                .unwrap_or(0);
            for level in 0..column_height {
                if lowest_neighbor > level {
                    continue;
                }
                let (y, thickness) = if level + 1 == column_height {
                    (level as f32 + (1.0 - cap) / 2.0, 1.0 - cap)
                } else {
                    (level as f32 + 0.5, 1.0)
                };
                dirt.push(InstanceTransform::new(
                    Vec3::new(middle.x, y, middle.y),
                    Quat::identity(),
                    Vec3::new(1.0, thickness, 1.0),
                ));
            }
        }
    }

    commands.queue(move |storage, resources| {
        for (instances, material) in [
            (grass, theme::GRASS_MATERIAL),
            (sand, theme::SAND_MATERIAL),
            (stone, theme::STONE_MATERIAL),
            (rock, theme::STONE_WALL_MATERIAL),
            (plank, theme::PLANK_MATERIAL),
            (dirt, theme::DIRT_MATERIAL),
        ] {
            if instances.is_empty() {
                continue;
            }
            let material_id =
                nightshade_render::queries::material_registry::material_registry_resolve(
                    resources
                        .res::<nightshade_render::resources::material_registry::MaterialRegistry>(),
                    material,
                )
                .unwrap_or_default();
            spawn_instanced_mesh_with_material(storage, resources, "Cube", instances, material_id);
        }
    });
}

pub fn spawn_bridge_rails(commands: &mut Commands, terrain: &Terrain) {
    let mut posts = Vec::new();
    let deck = theme::BRIDGE_HEIGHT as f32;
    for row in 0..terrain.rows {
        for column in 0..terrain.columns {
            if !is_bridge(terrain, column, row) {
                continue;
            }
            let middle = center(terrain, column, row);
            for side in [-0.42, 0.42] {
                posts.push(InstanceTransform::new(
                    Vec3::new(middle.x + side, deck + 0.4, middle.y),
                    Quat::identity(),
                    Vec3::new(0.1, 0.8, 0.1),
                ));
                posts.push(InstanceTransform::new(
                    Vec3::new(middle.x + side, deck + 0.75, middle.y),
                    Quat::identity(),
                    Vec3::new(0.06, 0.06, 1.0),
                ));
            }
        }
    }
    if posts.is_empty() {
        return;
    }
    commands.queue(move |storage, resources| {
        let material_id = nightshade_render::queries::material_registry::material_registry_resolve(
            resources.res::<nightshade_render::resources::material_registry::MaterialRegistry>(),
            theme::TIMBER_MATERIAL,
        )
        .unwrap_or_default();
        spawn_instanced_mesh_with_material(storage, resources, "Cube", posts, material_id);
    });
}

pub fn spawn_block(commands: &mut Commands, material: &'static str, position: Vec3, scale: Vec3) {
    commands.queue(move |storage, resources| {
        let material_id = nightshade_render::queries::material_registry::material_registry_resolve(
            resources.res::<nightshade_render::resources::material_registry::MaterialRegistry>(),
            material,
        )
        .unwrap_or_default();
        spawn_instanced_mesh_with_material(
            storage,
            resources,
            "Cube",
            vec![InstanceTransform::new(position, Quat::identity(), scale)],
            material_id,
        );
    });
}

pub fn spawn_marked_blocks<T: 'static>(
    commands: &mut Commands,
    material: &'static str,
    blocks: Vec<InstanceTransform>,
) {
    commands.queue(move |storage, resources| {
        let material_id = nightshade_render::queries::material_registry::material_registry_resolve(
            resources.res::<nightshade_render::resources::material_registry::MaterialRegistry>(),
            material,
        )
        .unwrap_or_default();
        let entity =
            spawn_instanced_mesh_with_material(storage, resources, "Cube", blocks, material_id);
        storage.add_tag_type::<T>(entity);
    });
}

pub fn spawn_sea(commands: &mut Commands) {
    spawn_mesh_with_material(
        commands,
        "Plane",
        Vec3::new(0.0, theme::SEA_LEVEL, 0.0),
        Vec3::new(theme::SEA_SIZE, 1.0, theme::SEA_SIZE),
        theme::SEA_MATERIAL,
    );
}

pub fn spawn_water(commands: &mut Commands, terrain: &Terrain) {
    let middle = terrain.origin + Vec2::new(terrain.columns as f32, terrain.rows as f32) / 2.0;
    spawn_mesh_with_material(
        commands,
        "Cube",
        Vec3::new(middle.x, theme::WATER_LEVEL, middle.y),
        Vec3::new(terrain.columns as f32, 0.05, terrain.rows as f32),
        theme::WATER_MATERIAL,
    );
}
