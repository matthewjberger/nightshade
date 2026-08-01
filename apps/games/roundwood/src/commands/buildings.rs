use crate::commands::world::{block, spawn_ring};
use crate::data;
use crate::queries::cylinder;
use crate::queries::town::center;
use crate::resources::Town;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_render::components::mesh::InstanceTransform;

pub fn spawn_cottages(commands: &mut Commands, town: &mut Town, turntable: Entity) {
    let mut walls: Vec<(&'static str, Vec<InstanceTransform>)> = Vec::new();
    let mut doors = Vec::new();
    let mut windows = Vec::new();
    let mut awnings = Vec::new();
    let mut posts = Vec::new();

    for cottage in data::COTTAGES {
        let base = data::LAND_HEIGHT as f32;
        let face = if cottage.door_west {
            cottage.column
        } else {
            cottage.column + data::COTTAGE_WIDTH - 1
        };
        let middle_row = cottage.row + data::COTTAGE_DEPTH / 2;
        for step in 0..data::COTTAGE_WIDTH {
            for depth in 0..data::COTTAGE_DEPTH {
                let column = cottage.column + step;
                let row = (cottage.row + depth) % town.rows;
                block(town, column, row);
                let edge = step == 0
                    || step + 1 == data::COTTAGE_WIDTH
                    || depth == 0
                    || depth + 1 == data::COTTAGE_DEPTH;
                if !edge {
                    continue;
                }
                let spot = center(column, row);
                for level in 0..data::COTTAGE_WALL_LEVELS {
                    let lift = base + level as f32 + 0.5;
                    let instance = InstanceTransform::new(
                        cylinder::surface(spot.x, spot.y, lift),
                        cylinder::tilt(spot.y),
                        Vec3::new(1.0, 1.0, cylinder::arc_at(lift + 0.5)),
                    );
                    let on_face = column == face;
                    let doorway = on_face && row == middle_row % town.rows && level < 2;
                    let pane = on_face
                        && level == 1
                        && (row == (cottage.row) % town.rows
                            || row == (cottage.row + data::COTTAGE_DEPTH - 1) % town.rows);
                    if doorway {
                        doors.push(instance);
                    } else if pane {
                        windows.push(instance);
                    } else {
                        match walls.iter_mut().find(|(name, _)| *name == cottage.wall) {
                            Some((_, list)) => list.push(instance),
                            None => walls.push((cottage.wall, vec![instance])),
                        }
                    }
                }
            }
        }

        let peak = base + data::COTTAGE_WALL_LEVELS as f32;
        let middle_column = cottage.column as f32 + data::COTTAGE_WIDTH as f32 / 2.0;
        for course in 0..3u32 {
            let lift = peak + 0.2 + course as f32 * 0.3;
            let width = data::COTTAGE_WIDTH as f32 + 0.6 - course as f32 * 1.6;
            if width <= 0.2 {
                continue;
            }
            for depth in 0..data::COTTAGE_DEPTH {
                let row = (cottage.row + depth) % town.rows;
                let spot = center(cottage.column, row);
                let instance = InstanceTransform::new(
                    cylinder::surface(middle_column, spot.y, lift),
                    cylinder::tilt(spot.y),
                    Vec3::new(width, 0.4, cylinder::arc_at(lift) * 1.02),
                );
                match walls.iter_mut().find(|(name, _)| *name == cottage.roof) {
                    Some((_, list)) => list.push(instance),
                    None => walls.push((cottage.roof, vec![instance])),
                }
            }
        }

        if !cottage.awning {
            continue;
        }
        let reach = if cottage.door_west { -0.7 } else { 0.7 };
        for depth in 0..data::COTTAGE_DEPTH {
            let row = (cottage.row + depth) % town.rows;
            let spot = center(face, row);
            let lift = base + 2.3;
            awnings.push(InstanceTransform::new(
                cylinder::surface(spot.x + reach, spot.y, lift),
                cylinder::tilt(spot.y),
                Vec3::new(1.4, 0.16, cylinder::arc_at(lift)),
            ));
        }
        let sign = center(face, middle_row % town.rows);
        posts.push(InstanceTransform::new(
            cylinder::surface(sign.x + reach * 1.6, sign.y, base + 0.9),
            cylinder::tilt(sign.y),
            Vec3::new(0.12, 1.8, 0.12),
        ));
        posts.push(InstanceTransform::new(
            cylinder::surface(sign.x + reach * 1.6, sign.y, base + 1.7),
            cylinder::tilt(sign.y),
            Vec3::new(0.9, 0.7, 0.14),
        ));
    }

    commands.queue(move |storage, resources| {
        for (material, instances) in walls {
            spawn_ring(storage, resources, turntable, instances, material);
        }
        spawn_ring(storage, resources, turntable, doors, data::DOOR_MATERIAL);
        spawn_ring(
            storage,
            resources,
            turntable,
            windows,
            data::WINDOW_MATERIAL,
        );
        spawn_ring(
            storage,
            resources,
            turntable,
            awnings,
            data::AWNING_MATERIAL,
        );
        spawn_ring(storage, resources, turntable, posts, data::TIMBER_MATERIAL);
    });
}

pub fn spawn_lanterns(commands: &mut Commands, town: &mut Town, turntable: Entity) {
    let mut posts = Vec::new();
    let mut lamps = Vec::new();
    for spot in data::LANTERN_SPOTS {
        let base = data::LAND_HEIGHT as f32;
        posts.push(InstanceTransform::new(
            cylinder::surface(spot.x, spot.y, base + 0.7),
            cylinder::tilt(spot.y),
            Vec3::new(0.12, 1.4, 0.12),
        ));
        lamps.push(InstanceTransform::new(
            cylinder::surface(spot.x, spot.y, base + 1.52),
            cylinder::tilt(spot.y),
            Vec3::new(0.22, 0.24, 0.22),
        ));
        occupy(town, spot);
    }
    commands.queue(move |storage, resources| {
        spawn_ring(storage, resources, turntable, posts, data::TIMBER_MATERIAL);
        spawn_ring(storage, resources, turntable, lamps, data::LAMP_MATERIAL);
    });
}

pub fn spawn_board(commands: &mut Commands, town: &mut Town, turntable: Entity) {
    let spot = data::BOARD;
    let base = data::LAND_HEIGHT as f32;
    let mut timber = Vec::new();
    for side in [-0.4, 0.4] {
        timber.push(InstanceTransform::new(
            cylinder::surface(spot.x + side, spot.y, base + 0.5),
            cylinder::tilt(spot.y),
            Vec3::new(0.12, 1.0, 0.12),
        ));
    }
    timber.push(InstanceTransform::new(
        cylinder::surface(spot.x, spot.y, base + 1.1),
        cylinder::tilt(spot.y),
        Vec3::new(1.2, 0.7, 0.14),
    ));
    occupy(town, spot);
    commands.queue(move |storage, resources| {
        spawn_ring(storage, resources, turntable, timber, data::PLANK_MATERIAL);
    });
}

fn occupy(town: &mut Town, spot: Vec2) {
    let row = cylinder::wrap(spot.y).floor() as u32 % town.rows;
    let column = spot.x.floor() as u32;
    if column < town.columns {
        block(town, column, row);
    }
}
