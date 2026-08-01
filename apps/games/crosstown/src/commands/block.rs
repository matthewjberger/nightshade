use crate::components::Piece;
use crate::data;
use crate::data::Look;
use crate::queries::noise;
use nalgebra_glm::{Quat, Vec3};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::{
    spawn_instanced_mesh_with_material, spawn_mesh_with_material,
};
use nightshade_graphics::commands::light::spawn_light_entity;
use nightshade_render::components::light::{Light, LightType};
use nightshade_render::components::mesh::InstanceTransform;

pub struct Batch {
    pub ground: Vec<InstanceTransform>,
    pub paint: Vec<InstanceTransform>,
    pub walk: Vec<InstanceTransform>,
    pub wall: Vec<InstanceTransform>,
    pub dark_pane: Vec<InstanceTransform>,
    pub lit_pane: Vec<InstanceTransform>,
    pub metal: Vec<InstanceTransform>,
    pub wood: Vec<InstanceTransform>,
    pub rust: Vec<InstanceTransform>,
}

impl Batch {
    fn new() -> Self {
        Self {
            ground: Vec::new(),
            paint: Vec::new(),
            walk: Vec::new(),
            wall: Vec::new(),
            dark_pane: Vec::new(),
            lit_pane: Vec::new(),
            metal: Vec::new(),
            wood: Vec::new(),
            rust: Vec::new(),
        }
    }
}

pub fn build(commands: &mut Commands, look: Look, end: f32, stage: u32) {
    let mut batch = Batch::new();
    let start = data::STREET_START;
    let ground = match look {
        Look::Street => data::ASPHALT,
        Look::Alley => data::ASPHALT,
        Look::Subway => data::TILE_WALL,
        Look::Rooftop => data::GRAVEL,
        Look::Yard => data::CONCRETE,
    };
    let walk = match look {
        Look::Subway => data::CONCRETE,
        Look::Rooftop => data::GRAVEL,
        _ => data::CONCRETE,
    };
    let wall = match look {
        Look::Subway => data::TILE_WALL,
        Look::Yard => data::RUST,
        _ => data::BRICK,
    };

    lay_ground(&mut batch, start, end, look);
    match look {
        Look::Street => street(&mut batch, commands, start, end, stage),
        Look::Alley => alley(&mut batch, commands, start, end, stage),
        Look::Subway => subway(&mut batch, commands, start, end, stage),
        Look::Rooftop => rooftop(&mut batch, commands, start, end, stage),
        Look::Yard => yard(&mut batch, commands, start, end, stage),
    }

    commands.queue(move |storage, resources| {
        for (instances, material) in [
            (batch.ground, ground),
            (batch.paint, data::PAINT),
            (batch.walk, walk),
            (batch.wall, wall),
            (batch.dark_pane, data::WINDOW_DARK),
            (batch.lit_pane, data::WINDOW_LIT),
            (batch.metal, data::METAL),
            (batch.wood, data::WOOD),
            (batch.rust, data::RUST),
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
            let entity = spawn_instanced_mesh_with_material(
                storage,
                resources,
                "Cube",
                instances,
                material_id,
            );
            storage.set(entity, Piece(stage));
        }
    });
}

fn lay_ground(batch: &mut Batch, start: f32, end: f32, look: Look) {
    let roof_edge = data::ROAD_HALF + data::ROOF_LEDGE;
    let mut x = start;
    while x < end {
        let (near, far) = if look == Look::Rooftop {
            (-roof_edge, roof_edge)
        } else {
            (-data::ROAD_HALF, data::ROAD_HALF)
        };
        let mut z = near;
        while z < far {
            batch
                .ground
                .push(slab(x + 0.5, z + 0.5, 0.0, 1.0, 1.0, 0.3));
            z += 1.0;
        }
        if look != Look::Rooftop {
            for side in [-1.0, 1.0] {
                let edge = side * (data::ROAD_HALF + 0.25);
                batch
                    .walk
                    .push(slab(x + 0.5, edge, data::CURB_HEIGHT, 1.0, 0.5, 0.34));
                let middle = side * (data::ROAD_HALF + 0.5 + data::WALK_WIDTH * 0.5);
                batch.walk.push(slab(
                    x + 0.5,
                    middle,
                    data::CURB_HEIGHT,
                    1.0,
                    data::WALK_WIDTH,
                    0.34,
                ));
            }
        }
        x += 1.0;
    }
    let depth = if look == Look::Rooftop {
        roof_edge * 2.0 + 1.0
    } else {
        (data::ROAD_HALF + 0.5 + data::WALK_WIDTH) * 2.0 + 1.0
    };
    batch.wall.push(InstanceTransform::new(
        Vec3::new((start + end) * 0.5, -data::UNDERSIDE * 0.5 - 0.3, 0.0),
        Quat::identity(),
        Vec3::new(end - start + 2.0, data::UNDERSIDE, depth),
    ));
}

fn frontage(batch: &mut Batch, start: f32, end: f32, front: f32, lit: bool) {
    let mut block = 0u32;
    let mut at = start - 2.0;
    while at < end + 2.0 {
        let height =
            data::BUILDING_LOW + noise(block * 17 + 3) * (data::BUILDING_HIGH - data::BUILDING_LOW);
        batch.wall.push(InstanceTransform::new(
            Vec3::new(
                at + data::BUILDING_RUN * 0.5,
                height * 0.5,
                front - data::BUILDING_DEPTH * 0.5 + 0.1,
            ),
            Quat::identity(),
            Vec3::new(data::BUILDING_RUN - 0.2, height, data::BUILDING_DEPTH),
        ));
        if lit {
            let mut level = 1.4;
            let mut row = 0u32;
            while level < height - 1.2 {
                for column in 0..3u32 {
                    let pane = InstanceTransform::new(
                        Vec3::new(at + 1.0 + column as f32 * 1.5, level, front + 0.12),
                        Quat::identity(),
                        Vec3::new(0.95, 1.15, 0.12),
                    );
                    if noise(block * 131 + row * 19 + column) > 0.48 {
                        batch.lit_pane.push(pane);
                    } else {
                        batch.dark_pane.push(pane);
                    }
                }
                level += 1.9;
                row += 1;
            }
        }
        at += data::BUILDING_RUN;
        block += 1;
    }
}

fn street(batch: &mut Batch, commands: &mut Commands, start: f32, end: f32, stage: u32) {
    let mut dash = start;
    while dash < end {
        batch
            .paint
            .push(slab(dash + 0.9, 0.0, 0.02, 1.8, 0.16, 0.03));
        dash += 3.4;
    }
    let front = -(data::ROAD_HALF + 0.5 + data::WALK_WIDTH);
    frontage(batch, start, end, front, true);
    parapet(batch, start, end);
    let mut lamp = 0u32;
    let mut post = start + 2.0;
    while post < end {
        let side = if lamp.is_multiple_of(2) {
            front + 1.2
        } else {
            -front - 1.2
        };
        spawn_lamp(commands, post, side, stage);
        post += data::LAMP_SPACING * 0.5;
        lamp += 1;
    }
}

fn alley(batch: &mut Batch, commands: &mut Commands, start: f32, end: f32, stage: u32) {
    let front = -(data::ROAD_HALF + 0.6);
    frontage(batch, start, end, front, true);
    let mut post = start + 3.0;
    let mut index = 0u32;
    while post < end {
        if index.is_multiple_of(2) {
            batch
                .metal
                .push(slab(post, front + 0.5, 3.4, 1.2, 0.9, 0.16));
        }
        spawn_bulb(
            commands,
            Vec3::new(post, 3.2, front + 0.7),
            data::LAMP_COLOR,
            data::LAMP_STRENGTH * 0.8,
            stage,
        );
        post += 7.0;
        index += 1;
    }
    parapet(batch, start, end);
}

fn subway(batch: &mut Batch, commands: &mut Commands, start: f32, end: f32, stage: u32) {
    let back = -(data::ROAD_HALF + 0.5 + data::WALK_WIDTH);
    batch.wall.push(InstanceTransform::new(
        Vec3::new((start + end) * 0.5, 3.0, back - 0.8),
        Quat::identity(),
        Vec3::new(end - start, 6.0, 1.6),
    ));
    batch.wall.push(InstanceTransform::new(
        Vec3::new((start + end) * 0.5, 5.8, back + 3.2),
        Quat::identity(),
        Vec3::new(end - start, 0.5, 8.0),
    ));
    let mut pillar = start + 3.0;
    while pillar < end {
        batch.wall.push(InstanceTransform::new(
            Vec3::new(pillar, 2.8, -(data::ROAD_HALF + 1.4)),
            Quat::identity(),
            Vec3::new(0.7, 5.6, 0.7),
        ));
        spawn_strip(commands, Vec3::new(pillar + 3.0, 5.2, -1.0), stage);
        pillar += 6.0;
    }
    let rail = back + 1.1;
    let mut sleeper = start;
    while sleeper < end {
        batch.wood.push(slab(sleeper, rail, -0.5, 0.5, 2.2, 0.2));
        sleeper += 1.4;
    }
    for offset in [-0.5, 0.5] {
        batch.metal.push(slab(
            (start + end) * 0.5,
            rail + offset,
            -0.3,
            end - start,
            0.14,
            0.14,
        ));
    }
    batch.walk.push(slab(
        (start + end) * 0.5,
        data::ROAD_HALF + 0.5 + data::WALK_WIDTH,
        1.15,
        end - start,
        0.5,
        1.15,
    ));
}

fn rooftop(batch: &mut Batch, commands: &mut Commands, start: f32, end: f32, stage: u32) {
    let edge = data::ROAD_HALF + data::ROOF_LEDGE;
    for side in [-1.0, 1.0] {
        batch.walk.push(slab(
            (start + end) * 0.5,
            side * edge,
            0.75,
            end - start,
            0.6,
            0.75,
        ));
    }
    batch
        .walk
        .push(slab(start - 0.5, 0.0, 0.75, 0.6, edge * 2.0, 0.75));
    batch
        .walk
        .push(slab(end + 0.5, 0.0, 0.75, 0.6, edge * 2.0, 0.75));

    let mut tank = start + 5.0;
    let mut index = 0u32;
    while tank < end {
        let far = -(edge - 1.6);
        batch.rust.push(InstanceTransform::new(
            Vec3::new(tank, 1.4, far),
            Quat::identity(),
            Vec3::new(1.8, 2.8, 1.8),
        ));
        batch.metal.push(InstanceTransform::new(
            Vec3::new(tank, 0.35, far),
            Quat::identity(),
            Vec3::new(0.18, 0.7, 0.18),
        ));
        if index.is_multiple_of(2) {
            spawn_bulb(
                commands,
                Vec3::new(tank + 2.4, 2.6, 0.0),
                data::LAMP_COLOR,
                data::LAMP_STRENGTH * 0.8,
                stage,
            );
        }
        tank += 9.0;
        index += 1;
    }

    let mut roof = 0u32;
    let mut at = start - 8.0;
    while at < end + 10.0 {
        let drop = 3.0 + noise(roof * 31 + 5) * 5.0;
        let run = 7.0 + noise(roof * 17) * 5.0;
        let near = -(edge + 2.0);
        batch.wall.push(InstanceTransform::new(
            Vec3::new(at + run * 0.5, -drop - 7.0, near - 5.0),
            Quat::identity(),
            Vec3::new(run, 14.0, 10.0),
        ));
        batch.walk.push(slab(
            at + run * 0.5,
            near - 5.0,
            -drop,
            run - 0.3,
            10.0,
            0.5,
        ));
        at += run;
        roof += 1;
    }

    let mut tower = 0u32;
    let mut back = start - 12.0;
    while back < end + 16.0 {
        let height = 26.0 + noise(tower * 41 + 7) * 22.0;
        let run = 8.0 + noise(tower * 23) * 4.0;
        batch.wall.push(InstanceTransform::new(
            Vec3::new(
                back + run * 0.5,
                height * 0.5 - 10.0,
                -30.0 - noise(tower * 13) * 10.0,
            ),
            Quat::identity(),
            Vec3::new(run, height, 8.0),
        ));
        let mut level = 2.0;
        let mut row = 0u32;
        while level < height - 12.0 {
            for column in 0..2u32 {
                let pane = InstanceTransform::new(
                    Vec3::new(back + 2.0 + column as f32 * 3.0, level - 8.0, -17.6),
                    Quat::identity(),
                    Vec3::new(1.3, 1.6, 0.2),
                );
                if noise(tower * 97 + row * 11 + column) > 0.55 {
                    batch.lit_pane.push(pane);
                } else {
                    batch.dark_pane.push(pane);
                }
            }
            level += 3.0;
            row += 1;
        }
        back += run;
        tower += 1;
    }
}

fn yard(batch: &mut Batch, commands: &mut Commands, start: f32, end: f32, stage: u32) {
    let edge = data::ROAD_HALF + 0.5 + data::WALK_WIDTH;
    for side in [-1.0, 1.0] {
        let mut post = start;
        while post < end {
            batch.metal.push(InstanceTransform::new(
                Vec3::new(post, 1.6, side * edge),
                Quat::identity(),
                Vec3::new(0.16, 3.2, 0.16),
            ));
            post += 2.5;
        }
        batch.metal.push(slab(
            (start + end) * 0.5,
            side * edge,
            3.2,
            end - start,
            0.1,
            0.12,
        ));
    }
    let mut container = start + 2.0;
    let mut index = 0u32;
    while container < end {
        let far = -(edge + 2.2);
        batch.rust.push(InstanceTransform::new(
            Vec3::new(container + 3.0, 1.3, far - noise(index * 29) * 3.0),
            Quat::identity(),
            Vec3::new(6.0, 2.6, 2.6),
        ));
        if index.is_multiple_of(2) {
            spawn_flood(commands, Vec3::new(container + 4.0, 5.4, far + 1.0), stage);
        }
        container += 7.5;
        index += 1;
    }
    frontage(batch, start, end, -(edge + 7.0), false);
}

fn parapet(batch: &mut Batch, start: f32, end: f32) {
    batch.walk.push(slab(
        (start + end) * 0.5,
        data::ROAD_HALF + 0.5 + data::WALK_WIDTH,
        1.15,
        end - start,
        0.4,
        1.15,
    ));
}

pub fn gate_posts(commands: &mut Commands, look: Look, gate: f32, stage: u32) -> Vec<Entity> {
    let material = match look {
        Look::Subway => data::CONCRETE,
        _ => data::METAL,
    };
    let mut parts = Vec::new();
    for side in [-1.0, 1.0] {
        let post = spawn_mesh_with_material(
            commands,
            "Cube",
            Vec3::new(gate, 1.6, side * (data::ROAD_HALF + 0.2)),
            Vec3::new(0.3, 3.2, 0.3),
            material,
        );
        commands.set(post, Piece(stage));
        parts.push(post);
    }
    for level in 0..4u32 {
        let bar = spawn_mesh_with_material(
            commands,
            "Cube",
            Vec3::new(gate, 0.45 + level as f32 * 0.75, 0.0),
            Vec3::new(0.12, 0.14, data::ROAD_HALF * 2.0),
            material,
        );
        commands.set(bar, Piece(stage));
        parts.push(bar);
    }
    parts
}

fn spawn_lamp(commands: &mut Commands, x: f32, z: f32, stage: u32) {
    let post = spawn_mesh_with_material(
        commands,
        "Cube",
        Vec3::new(x, data::CURB_HEIGHT + data::LAMP_HEIGHT * 0.5, z),
        Vec3::new(0.14, data::LAMP_HEIGHT, 0.14),
        data::METAL,
    );
    commands.set(post, Piece(stage));
    let head = Vec3::new(x, data::CURB_HEIGHT + data::LAMP_HEIGHT, z);
    let lamp = spawn_mesh_with_material(
        commands,
        "Cube",
        head,
        Vec3::new(0.36, 0.24, 0.36),
        data::LAMP_GLASS,
    );
    commands.set(lamp, Piece(stage));
    spawn_bulb(
        commands,
        head - Vec3::new(0.0, 0.2, 0.0),
        data::LAMP_COLOR,
        data::LAMP_STRENGTH,
        stage,
    );
}

fn spawn_strip(commands: &mut Commands, at: Vec3, stage: u32) {
    let tube = spawn_mesh_with_material(
        commands,
        "Cube",
        at,
        Vec3::new(1.6, 0.12, 0.3),
        data::LAMP_GLASS,
    );
    commands.set(tube, Piece(stage));
    spawn_bulb(
        commands,
        at - Vec3::new(0.0, 0.3, 0.0),
        data::STRIP_COLOR,
        data::STRIP_STRENGTH,
        stage,
    );
}

fn spawn_flood(commands: &mut Commands, at: Vec3, stage: u32) {
    let housing = spawn_mesh_with_material(
        commands,
        "Cube",
        at,
        Vec3::new(0.5, 0.4, 0.4),
        data::LAMP_GLASS,
    );
    commands.set(housing, Piece(stage));
    spawn_bulb(
        commands,
        at + Vec3::new(0.0, -0.3, 0.6),
        data::FLOOD_COLOR,
        data::FLOOD_STRENGTH,
        stage,
    );
}

fn spawn_bulb(commands: &mut Commands, at: Vec3, color: Vec3, strength: f32, stage: u32) {
    let light = spawn_light_entity(commands, at, "Lamp");
    commands.set(
        light,
        Light {
            light_type: LightType::Point,
            color,
            intensity: strength,
            range: data::LAMP_RANGE,
            ..Default::default()
        },
    );
    commands.set(light, Piece(stage));
}

fn slab(x: f32, z: f32, top: f32, width: f32, depth: f32, height: f32) -> InstanceTransform {
    InstanceTransform::new(
        Vec3::new(x, top - height * 0.5, z),
        Quat::identity(),
        Vec3::new(width, height, depth),
    )
}
