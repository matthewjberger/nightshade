use crate::components::{Pulse, Room, Shade, Strength};
use crate::data::Structure;
use crate::data::{
    Build, CAMP_LANTERNS, CAMP_PROPS, CAMP_ROOMS, CAMP_TENTS, Feature, Place, Plan, built,
    team_tile,
};
use crate::queries::field::{camp_is_road, is_road, near_spawn};
use crate::resources::{Sheet, Terrain};
use crate::tags::{Scenery, Upright};
use crate::theme;
use fastrand::Rng;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::ChildOf;
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_graphics::commands::builtins::{spawn_instanced_mesh_with_material, spawn_mesh_at};
use nightshade_graphics::commands::light::spawn_light_entity;
use nightshade_render::components::light::{Light, LightType};
use nightshade_render::components::mesh::InstanceTransform;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn build(
    commands: &mut Commands,
    terrain: &mut Terrain,
    sheet: &Sheet,
    place: Place,
    plan: &Plan,
) {
    match place {
        Place::Field => {
            spawn_sites(commands, terrain, sheet, plan);
            spawn_the_props(commands, terrain, sheet, plan);
            spawn_lanterns(commands, terrain, place, plan, &theme::LANTERN_SPOTS);
        }
        Place::Camp => {
            spawn_rooms(commands, terrain);
            spawn_props(commands, terrain, sheet);
            spawn_campfire(commands, terrain);
            spawn_lanterns(commands, terrain, place, plan, &CAMP_LANTERNS);
        }
    }
}

fn spawn_rooms(commands: &mut Commands, terrain: &mut Terrain) {
    let mut parts = Vec::new();
    for room in &CAMP_ROOMS {
        let build = room.kind.build();
        let base = crate::queries::terrain::top(terrain, room.column, room.row);
        raise_the_building(&mut parts, base, &build);
        claim_the_ground(terrain, room.column, room.row, &build, Feature::Obstacle);

        let marker = commands.spawn((
            Room { kind: room.kind },
            LocalTransform {
                translation: base,
                ..Default::default()
            },
            GlobalTransform::default(),
        ));
        commands.add_tag_type::<Scenery>(marker);
        commands.set(marker, Visibility { visible: false });
    }
    pitch_the_tents(terrain, &mut parts);
    raise_the_parts(commands, parts);
}

fn spawn_sites(commands: &mut Commands, terrain: &mut Terrain, sheet: &Sheet, plan: &Plan) {
    let mut parts = Vec::new();
    for site in plan.sites {
        if crate::queries::terrain::is_water(terrain, site.column, site.row) {
            continue;
        }
        let build = built(site.kind);
        let base = crate::queries::terrain::top(terrain, site.column, site.row);
        raise_the_building(&mut parts, base, &build);
        claim_the_ground(terrain, site.column, site.row, &build, Feature::Structure);

        let flag = spawn_standing(
            commands,
            sheet,
            crate::queries::sheet::standing(sheet, team_tile(theme::FLAG_TILE, site.owner)),
            base + Vec3::new(0.0, build.height as f32 + theme::FLAG_LIFT, 0.0),
            Vec2::new(0.4, 0.3),
            NEUTRAL_SHADOW,
            0.0,
            theme::FLAG_SCALE,
        );
        commands.set(
            flag,
            SpriteAnimator::from_regions(
                [
                    crate::queries::sheet::standing(sheet, team_tile(theme::FLAG_TILE, site.owner))
                        .0,
                    crate::queries::sheet::standing(
                        sheet,
                        team_tile(theme::FLAG_WAVE_TILE, site.owner),
                    )
                    .0,
                ],
                theme::FLAG_FRAME_SECONDS,
            ),
        );
        commands.add_tag_type::<Scenery>(flag);

        let marker = commands.spawn((
            LocalTransform {
                translation: base,
                ..Default::default()
            },
            GlobalTransform::default(),
        ));
        crate::commands::set_structure(
            commands,
            marker,
            &Structure {
                kind: site.kind,
                owner: site.owner,
                column: site.column,
                row: site.row,
                flag: Some(flag),
            },
        );
        commands.add_tag_type::<Scenery>(marker);
        commands.set(marker, Visibility { visible: false });
    }
    raise_the_parts(commands, parts);
}

fn raise_the_building(
    parts: &mut Vec<(&'static str, Vec<InstanceTransform>)>,
    base: Vec3,
    build: &Build,
) {
    let base = base - Vec3::new(0.0, theme::BUILD_SINK, 0.0);
    let wide = build.wide;
    let deep = build.deep;
    let middle = wide / 2;
    let window_row = theme::WINDOW_ROW.min(build.height - 1);
    for across in 0..wide {
        for level in 0..build.height {
            for back in 0..deep {
                let front = back == deep - 1;
                let material = if front && level == 0 && across == middle {
                    build.door
                } else if front && level == window_row && across != middle {
                    build.window
                } else {
                    build.wall
                };
                push(
                    parts,
                    material,
                    base + Vec3::new(
                        across as f32 - (wide as f32 - 1.0) / 2.0,
                        level as f32 + 0.5,
                        back as f32 - (deep as f32 - 1.0),
                    ),
                    Vec3::new(1.0, 1.0, 1.0),
                    0.0,
                );
            }
        }
    }
    gable(
        parts,
        build.roof,
        base + Vec3::new(0.0, build.height as f32, -(deep as f32 - 1.0) / 2.0),
        wide + 1,
        theme::ROOF_HALF_DEPTH,
        theme::ROOF_RISE,
    );
}

fn claim_the_ground(terrain: &mut Terrain, column: u32, row: u32, build: &Build, feature: Feature) {
    let half = build.wide / 2;
    for near_column in column.saturating_sub(half)..=column + half {
        for near_row in row.saturating_sub(build.deep - 1)..=row {
            if near_column < terrain.columns && near_row < terrain.rows {
                mark(terrain, near_column, near_row, feature);
            }
        }
    }
}

fn raise_the_parts(commands: &mut Commands, parts: Vec<(&'static str, Vec<InstanceTransform>)>) {
    commands.queue(move |storage, resources| {
        for (material, instances) in parts {
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

fn pitch_the_tents(terrain: &mut Terrain, parts: &mut Vec<(&'static str, Vec<InstanceTransform>)>) {
    for (column, row) in CAMP_TENTS {
        if !camp_ground(terrain, column, row) {
            continue;
        }
        gable(
            parts,
            theme::RED_ROOF_MATERIAL,
            crate::queries::terrain::top(terrain, column, row),
            theme::TENT_BLOCKS,
            theme::TENT_HALF_DEPTH,
            theme::TENT_RISE,
        );
        mark(terrain, column, row, Feature::Obstacle);
    }
}

fn push(
    parts: &mut Vec<(&'static str, Vec<InstanceTransform>)>,
    material: &'static str,
    translation: Vec3,
    scale: Vec3,
    pitch: f32,
) {
    let instance = InstanceTransform::new(
        translation,
        nalgebra_glm::quat_angle_axis(pitch, &Vec3::x()),
        scale,
    );
    match parts.iter_mut().find(|(name, _)| *name == material) {
        Some((_, instances)) => instances.push(instance),
        None => parts.push((material, vec![instance])),
    }
}

fn gable(
    parts: &mut Vec<(&'static str, Vec<InstanceTransform>)>,
    material: &'static str,
    eaves: Vec3,
    slabs: u32,
    half: f32,
    rise: f32,
) {
    let angle = rise.atan2(half);
    let slope = (half * half + rise * rise).sqrt();
    for across in 0..slabs {
        let offset = across as f32 - (slabs as f32 - 1.0) / 2.0;
        for side in [-1.0, 1.0] {
            push(
                parts,
                material,
                eaves + Vec3::new(offset, rise / 2.0, side * half / 2.0),
                Vec3::new(1.0, theme::ROOF_THICKNESS, slope * theme::ROOF_OVERLAP),
                side * angle,
            );
        }
    }
}

pub fn spawn_standing(
    commands: &mut Commands,
    sheet: &Sheet,
    picture: (SpriteRegion, f32),
    position: Vec3,
    shadow: Vec2,
    shadow_tint: [f32; 4],
    hover: f32,
    scale: f32,
) -> Entity {
    let (region, height) = picture;
    let sprite = Sprite::new(theme::ATLAS, region)
        .with_size(scale, height * scale)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::LAYER_STANDING);
    let entity = spawn_sprite(commands, sprite, position + Vec3::new(0.0, hover, 0.0));
    commands.add_tag_type::<Upright>(entity);
    spawn_shadow(commands, sheet, entity, shadow * scale, shadow_tint, hover);
    entity
}

fn spawn_shadow(
    commands: &mut Commands,
    sheet: &Sheet,
    parent: Entity,
    size: Vec2,
    tint: [f32; 4],
    hover: f32,
) {
    let sprite = Sprite::new(theme::ATLAS, sheet.shadow)
        .with_size(size.x, size.y)
        .with_color(tint)
        .with_layer(theme::LAYER_GROUND);
    let shadow = spawn_sprite(commands, sprite, Vec3::new(0.0, 0.0, 0.0));
    commands.set(
        shadow,
        LocalTransform {
            translation: Vec3::new(
                0.0,
                theme::SHADOW_LIFT - hover,
                size.y * theme::SHADOW_FORWARD,
            ),
            rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(shadow, ChildOf(parent));
    commands.set(shadow, Shade { owner: parent });
}

fn mark(terrain: &mut Terrain, column: u32, row: u32, feature: Feature) {
    let index = crate::queries::terrain::index(terrain, column, row);
    terrain.features[index] = feature;
}

fn open_ground(terrain: &Terrain, plan: &Plan, column: u32, row: u32) -> bool {
    crate::queries::terrain::is_land(terrain, column, row)
        && crate::queries::terrain::feature(terrain, column, row) == Feature::Open
        && !is_road(terrain, column, row)
        && !near_spawn(plan, column, row)
        && in_the_open(terrain, column, row)
}

fn camp_ground(terrain: &Terrain, column: u32, row: u32) -> bool {
    crate::queries::terrain::is_land(terrain, column, row)
        && crate::queries::terrain::feature(terrain, column, row) == Feature::Open
        && !camp_is_road(terrain, column, row)
        && in_the_open(terrain, column, row)
}

fn in_the_open(terrain: &Terrain, column: u32, row: u32) -> bool {
    let here = crate::queries::terrain::height(terrain, column, row);
    let steps = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    steps.iter().all(|(across, down)| {
        let near_column = column as i32 + across;
        let near_row = row as i32 + down;
        if near_column < 0
            || near_row < 0
            || near_column as u32 >= terrain.columns
            || near_row as u32 >= terrain.rows
        {
            return true;
        }
        crate::queries::terrain::height(terrain, near_column as u32, near_row as u32) <= here
    })
}

fn buildable(terrain: &Terrain, place: Place, plan: &Plan, column: u32, row: u32) -> bool {
    match place {
        Place::Field => open_ground(terrain, plan, column, row),
        Place::Camp => camp_ground(terrain, column, row),
    }
}

const NEUTRAL_SHADOW: [f32; 4] = [0.0, 0.0, 0.0, theme::SHADOW_ALPHA];

fn spawn_props(commands: &mut Commands, terrain: &mut Terrain, sheet: &Sheet) {
    for prop in &CAMP_PROPS {
        if !camp_ground(terrain, prop.column, prop.row) {
            continue;
        }
        let piece = spawn_standing(
            commands,
            sheet,
            crate::queries::sheet::town_standing(sheet, prop.tile),
            crate::queries::terrain::top(terrain, prop.column, prop.row),
            theme::PROP_SHADOW,
            NEUTRAL_SHADOW,
            0.0,
            prop.scale,
        );
        commands.add_tag_type::<Scenery>(piece);
        mark(terrain, prop.column, prop.row, Feature::Obstacle);
    }
}

fn spawn_campfire(commands: &mut Commands, terrain: &mut Terrain) {
    let (column, row) = theme::CAMPFIRE_SPOT;
    let top = crate::queries::terrain::top(terrain, column, row);
    let ring = spawn_mesh_at(
        commands,
        "Cube",
        top + Vec3::new(0.0, theme::CAMPFIRE_RING_HEIGHT / 2.0, 0.0),
        Vec3::new(0.9, theme::CAMPFIRE_RING_HEIGHT, 0.9),
    );
    set_mesh_material(commands, ring, theme::POST_MATERIAL);
    commands.add_tag_type::<Scenery>(ring);

    let flame_center = top + Vec3::new(0.0, theme::CAMPFIRE_FLAME_LIFT, 0.0);
    let flame = spawn_mesh_at(
        commands,
        "Cube",
        flame_center,
        Vec3::new(0.34, theme::CAMPFIRE_FLAME_SIZE, 0.34),
    );
    set_mesh_material(commands, flame, theme::LAMP_MATERIAL);
    commands.add_tag_type::<Scenery>(flame);

    let light = spawn_light_entity(
        commands,
        flame_center + Vec3::new(0.0, 0.4, 0.0),
        "Campfire",
    );
    commands.set(
        light,
        Light {
            light_type: LightType::Point,
            color: theme::CAMPFIRE_COLOR,
            intensity: theme::CAMPFIRE_STRENGTH,
            range: theme::CAMPFIRE_RANGE,
            ..Default::default()
        },
    );
    commands.add_tag_type::<Scenery>(light);
    commands.set(light, Pulse(theme::CAMPFIRE_PHASE));
    commands.set(light, Strength(theme::CAMPFIRE_STRENGTH));
    mark(terrain, column, row, Feature::Obstacle);
}

fn spawn_the_props(commands: &mut Commands, terrain: &mut Terrain, sheet: &Sheet, plan: &Plan) {
    let mut rng = Rng::with_seed(19_u64);
    for row in 0..terrain.rows {
        for column in 0..terrain.columns {
            let mark_kind = crate::queries::field::mark_of(plan, column, row);
            if mark_kind == b'.' || mark_kind == b'=' {
                continue;
            }
            if !open_ground(terrain, plan, column, row) {
                continue;
            }
            let (tile, shadow, feature, scale) = match mark_kind {
                b'T' => {
                    if rng.f32() < 0.3 {
                        (theme::TALL_TREE, Vec2::new(1.0, 0.7), Feature::Tree, 1.0)
                    } else {
                        (theme::TREE, Vec2::new(0.9, 0.6), Feature::Tree, 1.0)
                    }
                }
                b'B' => {
                    if rng.f32() < 0.5 {
                        (theme::BARRICADE, Vec2::new(1.0, 0.7), Feature::Bunker, 1.0)
                    } else {
                        (theme::SANDBAGS, Vec2::new(1.0, 0.7), Feature::Bunker, 1.0)
                    }
                }
                _ => match (rng.f32() * 5.0) as u32 {
                    0 => (
                        theme::OFFICE_TILE,
                        Vec2::new(1.0, 0.7),
                        Feature::Obstacle,
                        1.0,
                    ),
                    1 => (
                        theme::CRANE_TILE,
                        Vec2::new(1.0, 0.7),
                        Feature::Obstacle,
                        1.0,
                    ),
                    2 => (
                        theme::HANGAR_TILE,
                        Vec2::new(1.0, 0.7),
                        Feature::Obstacle,
                        1.0,
                    ),
                    3 => (
                        theme::AIRPORT_TILE,
                        Vec2::new(1.0, 0.7),
                        Feature::Obstacle,
                        1.0,
                    ),
                    _ => (theme::ROCK, Vec2::new(0.9, 0.6), Feature::Obstacle, 1.0),
                },
            };
            let prop = spawn_standing(
                commands,
                sheet,
                crate::queries::sheet::standing(sheet, tile),
                crate::queries::terrain::top(terrain, column, row),
                shadow,
                NEUTRAL_SHADOW,
                0.0,
                scale,
            );
            commands.add_tag_type::<Scenery>(prop);
            mark(terrain, column, row, feature);
        }
    }
}

fn spawn_lanterns(
    commands: &mut Commands,
    terrain: &mut Terrain,
    place: Place,
    plan: &Plan,
    spots: &[(u32, u32)],
) {
    for (index, (column, row)) in spots.iter().copied().enumerate() {
        if !buildable(terrain, place, plan, column, row) {
            continue;
        }
        let top = crate::queries::terrain::top(terrain, column, row);
        let post = spawn_mesh_at(
            commands,
            "Cube",
            top + Vec3::new(0.0, theme::LANTERN_POST_HEIGHT / 2.0, 0.0),
            Vec3::new(0.12, theme::LANTERN_POST_HEIGHT, 0.12),
        );
        set_mesh_material(commands, post, theme::POST_MATERIAL);
        commands.add_tag_type::<Scenery>(post);

        let lamp_center = top
            + Vec3::new(
                0.0,
                theme::LANTERN_POST_HEIGHT + theme::LANTERN_LAMP_SIZE / 2.0,
                0.0,
            );
        let lamp = spawn_mesh_at(
            commands,
            "Cube",
            lamp_center,
            Vec3::new(
                theme::LANTERN_LAMP_SIZE,
                theme::LANTERN_LAMP_SIZE,
                theme::LANTERN_LAMP_SIZE,
            ),
        );
        set_mesh_material(commands, lamp, theme::LAMP_MATERIAL);
        commands.add_tag_type::<Scenery>(lamp);

        let light = spawn_light_entity(commands, lamp_center + Vec3::new(0.0, 0.3, 0.0), "Lantern");
        commands.set(
            light,
            Light {
                light_type: LightType::Point,
                color: theme::LANTERN_COLOR,
                intensity: theme::LANTERN_STRENGTH,
                range: theme::LANTERN_RANGE,
                ..Default::default()
            },
        );
        commands.add_tag_type::<Scenery>(light);
        commands.set(light, Pulse(index as f32 * 1.7));
        commands.set(light, Strength(theme::LANTERN_STRENGTH));
        mark(terrain, column, row, Feature::Obstacle);
    }
}
