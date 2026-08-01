use crate::commands::effects::{spawn_fire, spawn_forge_sparks, spawn_smoke};
use crate::commands::world::occupy;
use crate::components::{NightOnly, Phase, Strength};
use crate::data::{House, HouseKind, WallStyle};
use crate::queries::terrain::{column_of, height_at, index, is_walkable};
use crate::resources::{Door, Terrain};
use crate::theme;
use nalgebra_glm::{Quat, Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::spawn_instanced_mesh_with_material;
use nightshade_graphics::commands::light::spawn_light_entity;
use nightshade_render::components::light::{Light, LightType};
use nightshade_render::components::mesh::InstanceTransform;
use std::collections::HashMap;

type Blocks = HashMap<&'static str, Vec<InstanceTransform>>;

fn block(blocks: &mut Blocks, material: &'static str, position: Vec3, scale: Vec3) {
    blocks
        .entry(material)
        .or_default()
        .push(InstanceTransform::new(position, Quat::identity(), scale));
}

fn turned_block(
    blocks: &mut Blocks,
    material: &'static str,
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
) {
    blocks
        .entry(material)
        .or_default()
        .push(InstanceTransform::new(position, rotation, scale));
}

fn add_house(blocks: &mut Blocks, house: &House, base: f32) -> Vec3 {
    let wall = house.wall.material();
    let width = house.width as f32;
    let depth = house.depth as f32;
    let middle_x = house.origin.x + width / 2.0;
    let middle_z = house.origin.y + depth / 2.0;
    let window_level = house.height - 1;
    for level in 0..house.height {
        for cell_z in 0..house.depth {
            for cell_x in 0..house.width {
                let edge = cell_x == 0
                    || cell_x + 1 == house.width
                    || cell_z == 0
                    || cell_z + 1 == house.depth;
                if !edge {
                    continue;
                }
                let front = cell_z + 1 == house.depth;
                let corner = (cell_x == 0 || cell_x + 1 == house.width)
                    && (cell_z == 0 || cell_z + 1 == house.depth);
                let in_door = cell_x >= house.door && cell_x < house.door + house.door_width;
                let material = if front && level == 0 && in_door {
                    theme::DOOR_MATERIAL
                } else if front && level == 0 && house.kind == HouseKind::Forge && cell_x == 3 {
                    theme::FORGE_MATERIAL
                } else if front && level == window_level && house.windows.contains(&cell_x) {
                    theme::WINDOW_MATERIAL
                } else if corner && house.wall == WallStyle::Plaster {
                    theme::TIMBER_MATERIAL
                } else {
                    wall
                };
                block(
                    blocks,
                    material,
                    Vec3::new(
                        house.origin.x + cell_x as f32 + 0.5,
                        base + level as f32 + 0.5,
                        house.origin.y + cell_z as f32 + 0.5,
                    ),
                    Vec3::new(1.0, 1.0, 1.0),
                );
            }
        }
    }

    let eave_y = base + house.height as f32 + 0.1;
    let half = depth / 2.0 + 0.4;
    let slab_length = half * std::f32::consts::SQRT_2;
    let roof = house.roof.material();
    for side in [-1.0f32, 1.0] {
        turned_block(
            blocks,
            roof,
            Vec3::new(middle_x, eave_y + half / 2.0, middle_z + side * half / 2.0),
            nalgebra_glm::quat_angle_axis(side * std::f32::consts::FRAC_PI_4, &Vec3::x()),
            Vec3::new(width + 0.8, 0.2, slab_length),
        );
    }
    let mut step = 0;
    loop {
        let extent = depth / 2.0 - 0.5 * step as f32;
        if extent <= 0.0 {
            break;
        }
        block(
            blocks,
            wall,
            Vec3::new(
                middle_x,
                base + house.height as f32 + 0.5 * step as f32 + 0.25,
                middle_z,
            ),
            Vec3::new(width, 0.5, extent * 2.0),
        );
        step += 1;
    }
    let ridge_y = eave_y + half;
    block(
        blocks,
        theme::TIMBER_MATERIAL,
        Vec3::new(middle_x, ridge_y - 0.05, middle_z),
        Vec3::new(width + 0.9, 0.22, 0.36),
    );
    let chimney_top = Vec3::new(house.origin.x + width - 0.8, ridge_y + 0.5, middle_z - 0.5);
    if house.chimney {
        block(
            blocks,
            theme::STONE_WALL_MATERIAL,
            chimney_top - Vec3::new(0.0, 0.6, 0.0),
            Vec3::new(0.5, 1.2, 0.5),
        );
    }
    chimney_top
}

pub fn spawn_houses(commands: &mut Commands, terrain: &Terrain) -> Vec<Door> {
    let mut blocks = Blocks::new();
    let mut doors = Vec::new();
    for kind in HouseKind::ALL {
        let house = kind.house();
        let base = height_at(terrain, house.origin.x + 0.5, house.origin.y + 0.5)
            .unwrap_or(theme::VILLAGE_HEIGHT) as f32;
        let chimney_top = add_house(&mut blocks, &house, base);
        if house.chimney {
            spawn_smoke(commands, chimney_top + Vec3::new(0.0, 0.1, 0.0));
        }
        if kind == HouseKind::Forge {
            let furnace = Vec3::new(
                house.origin.x + 3.5,
                base,
                house.origin.y + house.depth as f32,
            );
            spawn_forge_sparks(commands, furnace + Vec3::new(0.0, 0.6, 0.15));
            let light = spawn_light_entity(commands, furnace + theme::FORGE_LIGHT_OFFSET, "Forge");
            commands.set(
                light,
                Light {
                    light_type: LightType::Point,
                    color: theme::FORGE_LIGHT_COLOR,
                    intensity: theme::FORGE_STRENGTH,
                    range: 6.0,
                    ..Default::default()
                },
            );
            commands.set(light, Phase(2.0));
            commands.set(light, Strength(theme::FORGE_STRENGTH));
            commands.set(light, NightOnly(false));
        }
        doors.push(Door {
            kind,
            spot: kind.door_spot(),
        });
    }
    spawn_blocks(commands, blocks);
    doors
}

fn spawn_blocks(commands: &mut Commands, blocks: Blocks) {
    commands.queue(move |storage, resources| {
        for (material, instances) in blocks {
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

pub fn spawn_pen_fence(commands: &mut Commands, terrain: &mut Terrain) {
    let mut blocks = Blocks::new();
    let y = theme::PEN_HEIGHT as f32;
    let min = theme::PEN_MIN;
    let max = theme::PEN_MAX;
    let gate_rows = [min.y + 2.0, min.y + 3.0];
    let post = |blocks: &mut Blocks, x: f32, z: f32| {
        block(
            blocks,
            theme::TIMBER_MATERIAL,
            Vec3::new(x, y + 0.45, z),
            Vec3::new(0.12, 0.9, 0.12),
        );
    };
    let rail = |blocks: &mut Blocks, from: Vec2, to: Vec2| {
        let middle = (from + to) / 2.0;
        let along_x = (to.x - from.x).abs() > 0.01;
        for lift in [0.4, 0.75] {
            block(
                blocks,
                theme::TIMBER_MATERIAL,
                Vec3::new(middle.x, y + lift, middle.y),
                if along_x {
                    Vec3::new(1.0, 0.06, 0.06)
                } else {
                    Vec3::new(0.06, 0.06, 1.0)
                },
            );
        }
    };
    let columns = (max.x - min.x) as u32;
    let rows = (max.y - min.y) as u32;
    for step in 0..=columns {
        let x = min.x + step as f32;
        post(&mut blocks, x, min.y);
        post(&mut blocks, x, max.y);
        if step < columns {
            rail(&mut blocks, Vec2::new(x, min.y), Vec2::new(x + 1.0, min.y));
            rail(&mut blocks, Vec2::new(x, max.y), Vec2::new(x + 1.0, max.y));
        }
    }
    for step in 0..=rows {
        let z = min.y + step as f32;
        post(&mut blocks, max.x, z);
        let gate = gate_rows.iter().any(|row| (z - row).abs() < 0.01) && step < rows;
        if !gate_rows
            .iter()
            .any(|row| (z - row).abs() < 0.01 || (z - row - 1.0).abs() < 0.01)
        {
            post(&mut blocks, min.x, z);
        }
        if step < rows {
            rail(&mut blocks, Vec2::new(max.x, z), Vec2::new(max.x, z + 1.0));
            if !gate {
                rail(&mut blocks, Vec2::new(min.x, z), Vec2::new(min.x, z + 1.0));
            }
        }
    }
    spawn_blocks(commands, blocks);
    for row in 0..rows {
        for column in 0..columns {
            let x = min.x + column as f32 + 0.5;
            let z = min.y + row as f32 + 0.5;
            let ring = column == 0 || column + 1 == columns || row == 0 || row + 1 == rows;
            let gate = column == 0
                && gate_rows
                    .iter()
                    .any(|gate_row| (z - 0.5 - gate_row).abs() < 0.01);
            if ring
                && !gate
                && let Some((c, r)) = column_of(terrain, x, z)
            {
                let at = index(terrain, c, r);
                terrain.walkable[at] = false;
            }
        }
    }
}

pub fn spawn_well(commands: &mut Commands, terrain: &mut Terrain) {
    let spot = theme::WELL;
    let y = height_at(terrain, spot.x, spot.y).unwrap_or(theme::MIN_HEIGHT) as f32;
    let mut blocks = Blocks::new();
    for (offset, scale) in [
        (Vec3::new(0.0, 0.25, -0.5), Vec3::new(1.2, 0.5, 0.2)),
        (Vec3::new(0.0, 0.25, 0.5), Vec3::new(1.2, 0.5, 0.2)),
        (Vec3::new(-0.5, 0.25, 0.0), Vec3::new(0.2, 0.5, 1.2)),
        (Vec3::new(0.5, 0.25, 0.0), Vec3::new(0.2, 0.5, 1.2)),
    ] {
        block(
            &mut blocks,
            theme::STONE_WALL_MATERIAL,
            Vec3::new(spot.x, y, spot.y) + offset,
            scale,
        );
    }
    for side in [-0.55, 0.55] {
        block(
            &mut blocks,
            theme::TIMBER_MATERIAL,
            Vec3::new(spot.x + side, y + 0.9, spot.y),
            Vec3::new(0.1, 1.4, 0.1),
        );
    }
    block(
        &mut blocks,
        theme::TIMBER_MATERIAL,
        Vec3::new(spot.x, y + 1.35, spot.y),
        Vec3::new(1.3, 0.08, 0.08),
    );
    turned_block(
        &mut blocks,
        theme::ROOF_DARK_MATERIAL,
        Vec3::new(spot.x, y + 1.7, spot.y - 0.3),
        nalgebra_glm::quat_angle_axis(-0.6, &Vec3::x()),
        Vec3::new(1.6, 0.08, 0.7),
    );
    turned_block(
        &mut blocks,
        theme::ROOF_DARK_MATERIAL,
        Vec3::new(spot.x, y + 1.7, spot.y + 0.3),
        nalgebra_glm::quat_angle_axis(0.6, &Vec3::x()),
        Vec3::new(1.6, 0.08, 0.7),
    );
    spawn_blocks(commands, blocks);
    if let Some((column, row)) = column_of(terrain, spot.x, spot.y) {
        occupy(terrain, column, row);
        let at = index(terrain, column, row);
        terrain.walkable[at] = false;
    }
}

pub fn spawn_anvil(commands: &mut Commands, terrain: &mut Terrain) {
    let spot = theme::ANVIL;
    let y = height_at(terrain, spot.x, spot.y).unwrap_or(theme::VILLAGE_HEIGHT) as f32;
    let mut blocks = Blocks::new();
    block(
        &mut blocks,
        theme::TIMBER_MATERIAL,
        Vec3::new(spot.x, y + 0.15, spot.y),
        Vec3::new(0.45, 0.3, 0.45),
    );
    block(
        &mut blocks,
        theme::IRON_MATERIAL,
        Vec3::new(spot.x, y + 0.42, spot.y),
        Vec3::new(0.6, 0.24, 0.26),
    );
    spawn_blocks(commands, blocks);
    if let Some((column, row)) = column_of(terrain, spot.x, spot.y) {
        occupy(terrain, column, row);
    }
}

pub fn spawn_wood_pile(commands: &mut Commands, terrain: &mut Terrain, spot: Vec2) {
    let y = height_at(terrain, spot.x, spot.y).unwrap_or(theme::VILLAGE_HEIGHT) as f32;
    let mut blocks = Blocks::new();
    for (index, (dx, dy)) in [(-0.22, 0.12), (0.22, 0.12), (0.0, 0.36)]
        .into_iter()
        .enumerate()
    {
        turned_block(
            &mut blocks,
            theme::PLANK_MATERIAL,
            Vec3::new(spot.x + dx, y + dy, spot.y),
            nalgebra_glm::quat_angle_axis(index as f32 * 0.1, &Vec3::x()),
            Vec3::new(0.22, 0.22, 0.9),
        );
    }
    spawn_blocks(commands, blocks);
    if let Some((column, row)) = column_of(terrain, spot.x, spot.y) {
        occupy(terrain, column, row);
    }
}

pub fn spawn_campfire(commands: &mut Commands, terrain: &mut Terrain) {
    let spot = theme::CAMPFIRE;
    let y = height_at(terrain, spot.x, spot.y).unwrap_or(theme::MIN_HEIGHT) as f32;
    let mut blocks = Blocks::new();
    for step in 0..3 {
        let angle = step as f32 * std::f32::consts::FRAC_PI_3;
        turned_block(
            &mut blocks,
            theme::TIMBER_MATERIAL,
            Vec3::new(spot.x, y + 0.08, spot.y),
            nalgebra_glm::quat_angle_axis(angle, &Vec3::y()),
            Vec3::new(0.7, 0.12, 0.12),
        );
    }
    for step in 0..7 {
        let angle = step as f32 * std::f32::consts::TAU / 7.0;
        block(
            &mut blocks,
            theme::STONE_WALL_MATERIAL,
            Vec3::new(
                spot.x + angle.cos() * 0.5,
                y + 0.09,
                spot.y + angle.sin() * 0.5,
            ),
            Vec3::new(0.18, 0.18, 0.18),
        );
    }
    spawn_blocks(commands, blocks);
    let flame = Vec3::new(spot.x, y + 0.2, spot.y);
    spawn_fire(commands, flame);
    let light = spawn_light_entity(commands, flame + Vec3::new(0.0, 0.6, 0.0), "Campfire");
    commands.set(
        light,
        Light {
            light_type: LightType::Point,
            color: theme::CAMPFIRE_COLOR,
            intensity: theme::CAMPFIRE_STRENGTH,
            range: 7.0,
            ..Default::default()
        },
    );
    commands.set(light, Phase(0.7));
    commands.set(light, Strength(theme::CAMPFIRE_STRENGTH));
    commands.set(light, NightOnly(false));
    if let Some((column, row)) = column_of(terrain, spot.x, spot.y) {
        occupy(terrain, column, row);
        if is_walkable(terrain, column, row) {
            let at = index(terrain, column, row);
            terrain.walkable[at] = false;
        }
    }
}
