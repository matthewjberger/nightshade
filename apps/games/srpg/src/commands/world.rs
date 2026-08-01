use nightshade_graphics::commands::builtins::set_mesh_material;
pub fn build(terrain: &mut Terrain, place: Place, plan: &Plan) {
    terrain.columns = theme::TERRAIN_COLUMNS;
    terrain.rows = theme::TERRAIN_ROWS;
    terrain.origin = theme::TERRAIN_ORIGIN;
    terrain.heights = (0..terrain.rows)
        .flat_map(|row| (0..terrain.columns).map(move |column| (column, row)))
        .map(|(column, row)| match place {
            Place::Field => height_of(column, row, plan),
            Place::Camp => camp_height_of(column, row),
        })
        .collect();
    terrain.features = vec![Feature::Open; terrain.heights.len()];
    terrain.roads = (0..terrain.rows)
        .flat_map(|row| (0..terrain.columns).map(move |column| (column, row)))
        .map(|(column, row)| match place {
            Place::Field => crate::queries::field::mark_of(plan, column, row) == b'=',
            Place::Camp => camp_is_road(terrain, column, row),
        })
        .collect();
}

fn height_of(column: u32, row: u32, plan: &Plan) -> u32 {
    let mut height = match crate::queries::field::ground_of(plan, column, row) {
        b'~' => theme::MIN_HEIGHT,
        mark => u32::from(mark.saturating_sub(b'0')),
    };
    if is_land_spawn(plan, column, row) || is_site(plan, column, row) {
        height = height.max(2);
    }
    if is_water_spawn(plan, column, row) {
        height = theme::MIN_HEIGHT;
    }
    height.clamp(theme::MIN_HEIGHT, theme::MAX_HEIGHT)
}

use crate::data::{Feature, Place, Plan};
use crate::queries::field::{
    camp_is_road, is_bank, is_land_spawn, is_road, is_site, is_water_spawn, neighbors,
};
use crate::resources::Terrain;
use crate::tags::Scenery;
use crate::theme;
use nalgebra_glm::{Quat, Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::{spawn_instanced_mesh_with_material, spawn_mesh_at};
use nightshade_render::components::mesh::InstanceTransform;

fn camp_height_of(column: u32, row: u32) -> u32 {
    if row >= theme::CAMP_WATER_ROW {
        return theme::MIN_HEIGHT;
    }
    if row == theme::CAMP_WATER_ROW - 1 {
        return 2;
    }
    let mut height = 3;
    if row <= theme::CAMP_TERRACE_ROW {
        height = 5;
    } else if row == theme::CAMP_TERRACE_ROW + 1 {
        height = 4;
    }
    if (theme::CAMP_LOOKOUT_ROWS.0..=theme::CAMP_LOOKOUT_ROWS.1).contains(&row) {
        if column <= 1 {
            height = 5;
        } else if column == 2 {
            height = 4;
        }
    }
    if (theme::CAMP_BLUFF_ROWS.0..=theme::CAMP_BLUFF_ROWS.1).contains(&row) && column >= 16 {
        height = 4;
    }
    height
}

fn camp_is_shore(terrain: &Terrain, column: u32, row: u32) -> bool {
    !crate::queries::terrain::is_water(terrain, column, row)
        && crate::queries::terrain::height(terrain, column, row) <= 2
}

pub fn raze(
    commands: &mut Commands,
    scenery: &mut Query<'_, (&nightshade_scene::prelude::LocalTransform,), WithTag<Scenery>>,
) {
    scenery.for_each_mut(|entity, _| commands.despawn_recursive(entity));
}

pub fn spawn_blocks(commands: &mut Commands, terrain: &Terrain, place: Place) {
    let mut grass = Vec::new();
    let mut sand = Vec::new();
    let mut stone = Vec::new();
    let mut dirt = Vec::new();
    let cap = theme::CAP_THICKNESS;

    for row in 0..terrain.rows {
        for column in 0..terrain.columns {
            let height = crate::queries::terrain::height(terrain, column, row);
            let center = crate::queries::terrain::center(terrain, column, row);
            let top = height as f32;
            let cap_instance = InstanceTransform::new(
                Vec3::new(center.x, top - cap / 2.0, center.y),
                Quat::identity(),
                Vec3::new(1.0, cap, 1.0),
            );
            let shore = match place {
                Place::Field => height == theme::MIN_HEIGHT || is_bank(terrain, column, row),
                Place::Camp => height == theme::MIN_HEIGHT || camp_is_shore(terrain, column, row),
            };
            let paved = match place {
                Place::Field => is_road(terrain, column, row),
                Place::Camp => camp_is_road(terrain, column, row),
            };
            if shore {
                sand.push(cap_instance);
            } else if paved {
                match place {
                    Place::Field => stone.push(cap_instance),
                    Place::Camp => sand.push(cap_instance),
                }
            } else {
                grass.push(cap_instance);
            }

            let lowest_neighbor = neighbors(terrain, column, row)
                .into_iter()
                .map(|neighbor| {
                    neighbor.map_or(0, |(c, r)| crate::queries::terrain::height(terrain, c, r))
                })
                .min()
                .unwrap_or(0);
            for level in 0..height {
                if lowest_neighbor > level {
                    continue;
                }
                let (y, thickness) = if level + 1 == height {
                    (level as f32 + (1.0 - cap) / 2.0, 1.0 - cap)
                } else {
                    (level as f32 + 0.5, 1.0)
                };
                dirt.push(InstanceTransform::new(
                    Vec3::new(center.x, y, center.y),
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
            (dirt, theme::DIRT_MATERIAL),
        ] {
            let material_id =
                nightshade_render::queries::material_registry::material_registry_resolve(
                    resources
                        .res::<nightshade_render::resources::material_registry::MaterialRegistry>(),
                    material,
                )
                .unwrap_or_default();
            let entity = spawn_instanced_mesh_with_material(
                storage,
                resources,
                "Cube",
                instances,
                material_id,
            );
            storage.add_tag_type::<Scenery>(entity);
        }
    });
}

pub fn spawn_sea(commands: &mut Commands) {
    let sea = spawn_mesh_at(
        commands,
        "Plane",
        Vec3::new(0.0, theme::SEA_LEVEL, 0.0),
        Vec3::new(theme::SEA_SIZE, 1.0, theme::SEA_SIZE),
    );
    set_mesh_material(commands, sea, theme::SEA_MATERIAL);
    commands.add_tag_type::<Scenery>(sea);
}

pub fn spawn_water(commands: &mut Commands, terrain: &Terrain) {
    let center = terrain.origin + Vec2::new(terrain.columns as f32, terrain.rows as f32) / 2.0;
    let water = spawn_mesh_at(
        commands,
        "Cube",
        Vec3::new(center.x, theme::WATER_LEVEL, center.y),
        Vec3::new(terrain.columns as f32, 0.05, terrain.rows as f32),
    );
    set_mesh_material(commands, water, theme::WATER_MATERIAL);
    commands.add_tag_type::<Scenery>(water);
}
