use crate::data::{self, Ground};
use crate::queries::cylinder;
use crate::queries::town::{center, ground, height, index, neighbors, shaped};
use crate::resources::Town;
use nalgebra_glm::Vec3;
use nightshade::ecs::{ChildOf, ResourceMap, Storage};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::spawn_instanced_mesh_with_material;
use nightshade_render::components::mesh::InstanceTransform;

pub fn spawn_ring(
    storage: &mut Storage,
    resources: &mut ResourceMap,
    turntable: Entity,
    instances: Vec<InstanceTransform>,
    material: &str,
) {
    if instances.is_empty() {
        return;
    }
    let material_id = nightshade_render::queries::material_registry::material_registry_resolve(
        resources.res::<nightshade_render::resources::material_registry::MaterialRegistry>(),
        material,
    )
    .unwrap_or_default();
    let entity =
        spawn_instanced_mesh_with_material(storage, resources, "Cube", instances, material_id);
    storage.set(entity, ChildOf(turntable));
}

pub fn build(town: &mut Town) {
    town.columns = data::ACROSS;
    town.rows = data::AROUND;
    let cells: Vec<(u32, u32)> = (0..town.rows)
        .flat_map(|row| (0..town.columns).map(move |column| (column, row)))
        .collect();
    let shaped: Vec<(u32, Ground)> = cells
        .iter()
        .map(|(column, row)| shaped(*column, *row))
        .collect();
    town.heights = shaped.iter().map(|(height, _)| *height).collect();
    town.ground = shaped.iter().map(|(_, ground)| *ground).collect();
    town.blocked = vec![false; cells.len()];
}

pub fn block(town: &mut Town, column: u32, row: u32) {
    let at = index(town, column, row);
    town.blocked[at] = true;
}

pub fn spawn_blocks(commands: &mut Commands, town: &Town, turntable: Entity) {
    let mut caps: Vec<(&'static str, Vec<InstanceTransform>)> = Vec::new();
    let mut sides: Vec<InstanceTransform> = Vec::new();
    let mut beds: Vec<InstanceTransform> = Vec::new();
    let cap = data::CAP_THICKNESS;

    for row in 0..town.rows {
        for column in 0..town.columns {
            let top = height(town, column, row);
            let kind = ground(town, column, row);
            let middle = center(column, row);
            let material = kind.material();
            let cap_instance = InstanceTransform::new(
                cylinder::surface(middle.x, middle.y, top as f32 - cap / 2.0),
                cylinder::tilt(middle.y),
                Vec3::new(1.0, cap, cylinder::arc_at(top as f32)),
            );
            match caps.iter_mut().find(|(name, _)| *name == material) {
                Some((_, list)) => list.push(cap_instance),
                None => caps.push((material, vec![cap_instance])),
            }
            if kind == Ground::Plank {
                continue;
            }

            let lowest = neighbors(town, column, row)
                .into_iter()
                .map(|neighbor| neighbor.map_or(0, |(c, r)| height(town, c, r)))
                .min()
                .unwrap_or(0);
            for level in 0..top {
                if lowest > level {
                    continue;
                }
                let (lift, thickness) = if level + 1 == top {
                    (level as f32 + (1.0 - cap) / 2.0, 1.0 - cap)
                } else {
                    (level as f32 + 0.5, 1.0)
                };
                let instance = InstanceTransform::new(
                    cylinder::surface(middle.x, middle.y, lift),
                    cylinder::tilt(middle.y),
                    Vec3::new(1.0, thickness, cylinder::arc_at(level as f32 + 1.0)),
                );
                match kind {
                    Ground::Sand | Ground::Water => beds.push(instance),
                    _ => sides.push(instance),
                }
            }
        }
    }

    commands.queue(move |storage, resources| {
        for (material, instances) in caps {
            spawn_ring(storage, resources, turntable, instances, material);
        }
        for (instances, material) in [(sides, data::DIRT_MATERIAL), (beds, data::SAND_MATERIAL)] {
            spawn_ring(storage, resources, turntable, instances, material);
        }
    });
}

pub fn spawn_water(commands: &mut Commands, town: &Town, turntable: Entity) {
    let mut water = Vec::new();
    let deep = data::WATER_DEPTH;
    for row in 0..town.rows {
        for column in 0..town.columns {
            if ground(town, column, row) != Ground::Water {
                continue;
            }
            let middle = center(column, row);
            water.push(InstanceTransform::new(
                cylinder::surface(middle.x, middle.y, data::SEA_LEVEL - deep / 2.0),
                cylinder::tilt(middle.y),
                Vec3::new(1.03, deep, cylinder::arc_at(data::SEA_LEVEL) * 1.03),
            ));
        }
    }
    commands.queue(move |storage, resources| {
        spawn_ring(storage, resources, turntable, water, data::WATER_MATERIAL);
    });
}

pub fn spawn_sea_apron(commands: &mut Commands, town: &Town, turntable: Entity) {
    let mut water = Vec::new();
    let mut beds = Vec::new();
    let deep = data::WATER_DEPTH;
    let columns = town.columns as f32;
    let mut out = 0.0;
    for (strip, width) in data::SEA_STRIPS.into_iter().enumerate() {
        let middle = out + width / 2.0;
        let lift = data::SEA_LEVEL - middle * middle / (2.0 * data::RADIUS);
        for row in 0..town.rows {
            let wrap = row as f32 + 0.5;
            for across in [-middle, columns + middle] {
                water.push(InstanceTransform::new(
                    cylinder::surface(across, wrap, lift - deep / 2.0),
                    cylinder::tilt(wrap),
                    Vec3::new(width * 1.02, deep, cylinder::arc_at(lift) * 1.02),
                ));
                if strip == 0 {
                    beds.push(InstanceTransform::new(
                        cylinder::surface(across, wrap, 0.5),
                        cylinder::tilt(wrap),
                        Vec3::new(width, 1.0, cylinder::arc_at(1.0)),
                    ));
                }
            }
        }
        out += width;
    }
    commands.queue(move |storage, resources| {
        spawn_ring(storage, resources, turntable, water, data::WATER_MATERIAL);
        spawn_ring(storage, resources, turntable, beds, data::SAND_MATERIAL);
    });
}

pub fn spawn_decks(commands: &mut Commands, town: &Town, turntable: Entity) {
    let mut posts = Vec::new();
    let mut rails = Vec::new();
    let deck = data::LAND_HEIGHT as f32;
    for row in 0..town.rows {
        for column in 0..town.columns {
            if ground(town, column, row) != Ground::Plank {
                continue;
            }
            let middle = center(column, row);
            for side in [-0.35, 0.35] {
                posts.push(InstanceTransform::new(
                    cylinder::surface(middle.x + side, middle.y, deck - 0.75),
                    cylinder::tilt(middle.y),
                    Vec3::new(0.14, 1.4, 0.14),
                ));
            }
            let on_bridge = (data::RIVER_START..data::RIVER_END).contains(&row);
            if !on_bridge {
                continue;
            }
            for side in [-0.44, 0.44] {
                rails.push(InstanceTransform::new(
                    cylinder::surface(middle.x + side, middle.y, deck + 0.4),
                    cylinder::tilt(middle.y),
                    Vec3::new(0.12, 0.8, 0.12),
                ));
                rails.push(InstanceTransform::new(
                    cylinder::surface(middle.x + side, middle.y, deck + 0.74),
                    cylinder::tilt(middle.y),
                    Vec3::new(0.08, 0.08, cylinder::arc_at(deck + 0.74)),
                ));
            }
        }
    }
    commands.queue(move |storage, resources| {
        spawn_ring(storage, resources, turntable, posts, data::TIMBER_MATERIAL);
        spawn_ring(storage, resources, turntable, rails, data::TIMBER_MATERIAL);
    });
}
