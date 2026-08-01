use crate::commands::block;
use crate::commands::effects::flame;
use crate::commands::interiors::furnish;
use crate::commands::models::{place_model, plant_model};
use crate::commands::random::*;
use crate::data::*;
use crate::data::{FIRE, FROST, HOLY, ICE, LIFE, SOOT, VIOLET};
use crate::queries::*;
use crate::resources::Palette;
use crate::tags::Lamp;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_random::prelude::*;
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
use nightshade_render::components::culling::CastsShadow;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::InstanceTransform;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::asset_id::MaterialId;
use nightshade_render::data::bounding_volume::{BoundingVolume, OrientedBoundingBox};
use nightshade_render::data::geometry::{Mesh, Vertex};
use nightshade_render::data::material::Material;
use nightshade_render::data::material::solid_material;
use nightshade_render::resources::mesh_cache::MeshCache;
use nightshade_render::resources::mesh_state::MeshRenderState;
use nightshade_scene::prelude::{ChildOf, GlobalTransform, LocalTransform, Name, Visibility};

const TERRAIN_MESH: &str = "thornmere_terrain";
const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;
const PI: f32 = std::f32::consts::PI;
const TAU: f32 = std::f32::consts::TAU;

fn glowing(color: [f32; 4], strength: f32) -> Material {
    Material {
        base_color: color,
        emissive_factor: [
            color[0] * strength,
            color[1] * strength,
            color[2] * strength,
        ],
        roughness: 0.6,
        metallic: 0.0,
        ..Default::default()
    }
}

pub fn register_palette(registry: &mut MaterialRegistry) -> Palette {
    let mut register = |name: &str, material: Material| insert_material(registry, name, material);
    Palette {
        grass: register("thornmere_grass", solid_material(GRASS_COLOR, 0.95, 0.0)),
        tuft: register("thornmere_tuft", solid_material(TUFT_COLOR, 0.95, 0.0)),
        water: register(
            "thornmere_water",
            Material {
                base_color: WATER_COLOR,
                roughness: 0.15,
                metallic: 0.2,
                ..Default::default()
            },
        ),
        fire: register(FIRE, glowing(FIRE_COLOR, 4.0)),
        frost: register(FROST, glowing(FROST_COLOR, 3.0)),
        lantern: register("thornmere_lantern", glowing(LANTERN_COLOR, 3.0)),
        holy: register(HOLY, glowing(HOLY_COLOR, 5.0)),
        life: register(LIFE, glowing(LIFE_COLOR, 4.0)),
        ice: register(
            ICE,
            Material {
                base_color: ICE_COLOR,
                emissive_factor: [0.25, 0.4, 0.6],
                roughness: 0.2,
                metallic: 0.1,
                ..Default::default()
            },
        ),
        soot: register(SOOT, solid_material(SOOT_COLOR, 1.0, 0.0)),
        violet: register(VIOLET, glowing(VIOLET_COLOR, 5.0)),
        stone: register("thornmere_stone", solid_material(STONE_COLOR, 0.9, 0.0)),
        cloud: register("thornmere_cloud", solid_material(CLOUD_COLOR, 1.0, 0.0)),
        feather: register("thornmere_feather", solid_material(FEATHER_COLOR, 0.9, 0.0)),
        flowers: FLOWER_COLORS
            .iter()
            .enumerate()
            .map(|(index, color)| {
                register(
                    &format!("thornmere_flower_{index}"),
                    solid_material(*color, 0.8, 0.0),
                )
            })
            .collect(),
        mobs: SPAWNS
            .iter()
            .enumerate()
            .map(|(index, spawn)| {
                register(
                    &format!("thornmere_mob_{index}"),
                    solid_material(spawn.color, 0.85, 0.0),
                )
            })
            .collect(),
        pickups: PICKUPS
            .iter()
            .enumerate()
            .map(|(index, pickup)| {
                register(
                    &format!("thornmere_pickup_{index}"),
                    glowing(pickup.color, 0.8),
                )
            })
            .collect(),
    }
}

pub fn prop(
    commands: &mut Commands,
    mesh: &str,
    position: Vec3,
    scale: Vec3,
    material: MaterialId,
) -> Entity {
    let entity = spawn_mesh_at(commands, mesh, position, scale);
    commands.set(entity, MaterialRef::new(material));
    entity
}

pub fn spawn_terrain(mesh_cache: &mut MeshCache, commands: &mut Commands, palette: &Palette) {
    let half = ZONE_SIZE / 2.0;
    let rows = TERRAIN_SUBDIVISIONS;
    let step = ZONE_SIZE / rows as f32;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut lowest = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut highest = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

    for row in 0..=rows {
        for column in 0..=rows {
            let x = -half + column as f32 * step;
            let z = -half + row as f32 * step;
            let position = Vec3::new(x, terrain_height(x, z), z);
            lowest = lowest.inf(&position);
            highest = highest.sup(&position);
            vertices.push(Vertex::with_tex_coords(
                position,
                terrain_normal(x, z),
                [column as f32 / rows as f32, 1.0 - row as f32 / rows as f32],
            ));
        }
    }

    let vertices_per_row = rows + 1;
    for row in 0..rows {
        for column in 0..rows {
            let top_left = row * vertices_per_row + column;
            let top_right = top_left + 1;
            let bottom_left = top_left + vertices_per_row;
            let bottom_right = bottom_left + 1;
            indices.push(top_left);
            indices.push(bottom_right);
            indices.push(top_right);
            indices.push(bottom_right);
            indices.push(top_left);
            indices.push(bottom_left);
        }
    }

    let bounds = BoundingVolume::new(
        OrientedBoundingBox::from_aabb(lowest, highest),
        (highest - lowest).norm() * 0.5,
    );
    let mesh = Mesh {
        bounding_volume: Some(bounds),
        ..Mesh::new(vertices, indices)
    };
    mesh_cache_insert(mesh_cache, TERRAIN_MESH.to_string(), mesh);

    let entity = commands.spawn((
        Name("Terrain".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        RenderMesh::new(TERRAIN_MESH),
        MaterialRef::new(palette.grass),
        bounds,
        CastsShadow,
        Visibility { visible: true },
    ));
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), entity);
    });
}

pub fn spawn_water(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    let lake = &PLACES[LAKE];
    let size = lake.radius * 2.3;
    prop(
        commands,
        "Plane",
        Vec3::new(lake.center.x, WATER_LEVEL, lake.center.y),
        Vec3::new(size, 1.0, size),
        palette.water,
    );
    let city = (CITY_MIN + CITY_MAX) / 2.0;
    let reach = MOAT_BANK + MOAT_WIDTH + 4.0;
    prop(
        commands,
        "Plane",
        Vec3::new(city.x, WATER_LEVEL, city.y),
        Vec3::new(
            CITY_MAX.x - CITY_MIN.x + reach * 2.0,
            1.0,
            CITY_MAX.y - CITY_MIN.y + reach * 2.0,
        ),
        palette.water,
    );
    for _ in 0..18 {
        let spot = random_place_spot(random, LAKE, 0.55);
        if terrain_height(spot.x, spot.y) > WATER_LEVEL - 0.6 {
            continue;
        }
        let pad = if random.bool() {
            "lily_large"
        } else {
            "lily_small"
        };
        place_model(
            commands,
            pad,
            Vec3::new(spot.x, WATER_LEVEL + 0.02, spot.y),
            random.f32() * TAU,
            2.6,
        );
    }
    let shore = lake.center + Vec2::new(lake.radius * 0.95, 4.0);
    plant_model(commands, "canoe", shore, 0.6, 2.6);
    plant_model(
        commands,
        "canoe_paddle",
        shore + Vec2::new(1.2, 0.8),
        1.3,
        2.6,
    );
    let wheel = lake.center + Vec2::new(0.0, lake.radius * 0.9);
    place_model(
        commands,
        "watermill",
        Vec3::new(wheel.x, WATER_LEVEL + 1.2, wheel.y),
        FRAC_PI_2,
        2.2,
    );
}

pub fn spawn_roads(commands: &mut Commands) {
    for road in ROADS {
        for pair in road.windows(2) {
            let span = pair[1] - pair[0];
            let length = span.magnitude();
            let direction = span / length;
            let yaw = direction.x.atan2(direction.y);
            let mut along = 0.0;
            while along <= length {
                let spot = pair[0] + direction * along;
                let position = Vec3::new(spot.x, terrain_height(spot.x, spot.y) + 0.06, spot.y);
                place_model(commands, "road", position, yaw, ROAD_WIDTH);
                along += ROAD_TILE_STEP;
            }
        }
    }
}

pub fn turned(local: Vec2, yaw: f32) -> Vec2 {
    Vec2::new(
        local.x * yaw.cos() + local.y * yaw.sin(),
        -local.x * yaw.sin() + local.y * yaw.cos(),
    )
}

pub fn house(
    commands: &mut Commands,
    palette: &Palette,
    random: &mut Random,
    spot: Vec2,
    yaw: f32,
    cells: usize,
    floors: usize,
    wood: bool,
    roof: &str,
    interior: Interior,
) {
    let size = HOUSE_CELL;
    let base = ground(spot);
    let half = (cells as f32 - 1.0) / 2.0;
    let door_cell = cells / 2;
    let tile = if wood {
        "holiday-floor-wood"
    } else {
        "retro-floor"
    };
    for floor in 0..floors {
        let upper = floor > 0;
        let prefix = if wood || upper { "wall-wood" } else { "wall" };
        let height = base.y + floor as f32 * size;
        for column in 0..cells {
            for row in 0..cells {
                let local = Vec2::new((column as f32 - half) * size, (row as f32 - half) * size);
                let center = spot + turned(local, yaw);
                place_model(
                    commands,
                    tile,
                    Vec3::new(center.x, height + 0.02, center.y),
                    yaw,
                    size,
                );
                if !upper {
                    let reach = size / 2.0;
                    let corner = |dx: f32, dy: f32| spot + turned(local + Vec2::new(dx, dy), yaw);
                    if column == 0 {
                        block(commands, corner(-reach, -reach), corner(-reach, reach));
                    }
                    if column + 1 == cells {
                        block(commands, corner(reach, -reach), corner(reach, reach));
                    }
                    if row == 0 {
                        block(commands, corner(-reach, -reach), corner(reach, -reach));
                    }
                    if row + 1 == cells && column != door_cell {
                        block(commands, corner(-reach, reach), corner(reach, reach));
                    }
                }
                let mut sides = Vec::new();
                if column == 0 {
                    sides.push((yaw + PI, false));
                }
                if column + 1 == cells {
                    sides.push((yaw, false));
                }
                if row == 0 {
                    sides.push((yaw + FRAC_PI_2, false));
                }
                if row + 1 == cells {
                    sides.push((yaw - FRAC_PI_2, column == door_cell && !upper));
                }
                for (side_yaw, front) in sides {
                    let piece = if front {
                        format!("{prefix}-doorway-square-wide")
                    } else if random.f32() < 0.45 {
                        if upper && !wood && random.bool() {
                            format!("{prefix}-window-glass")
                        } else {
                            format!("{prefix}-window-shutters")
                        }
                    } else {
                        prefix.to_string()
                    };
                    place_model(
                        commands,
                        &piece,
                        Vec3::new(center.x, height, center.y),
                        side_yaw,
                        size,
                    );
                }
            }
        }
    }
    furnish(
        commands, palette, random, spot, yaw, cells, base.y, interior,
    );
    let width = cells as f32 * size;
    place_model(
        commands,
        roof,
        Vec3::new(spot.x, base.y + floors as f32 * size - 0.05, spot.y),
        yaw,
        width * 0.95,
    );
    if random.f32() < 0.7 {
        let corner = spot + turned(Vec2::new(width * 0.3, -width * 0.3), yaw);
        place_model(
            commands,
            "chimney",
            Vec3::new(
                corner.x,
                base.y + floors as f32 * size + width * 0.12,
                corner.y,
            ),
            yaw,
            size,
        );
    }
}

pub fn lantern(commands: &mut Commands, palette: &Palette, spot: Vec2, yaw: f32) {
    let foot = ground(spot);
    place_model(commands, "lantern", foot, yaw, 3.2);
    let glow = prop(
        commands,
        "Sphere",
        foot + Vec3::new(0.0, 4.2, 0.0),
        Vec3::new(0.3, 0.3, 0.3),
        palette.lantern,
    );
    commands.add_tag_type::<Lamp>(glow);
}

pub fn campfire(commands: &mut Commands, palette: &Palette, spot: Vec2, scale: f32) {
    let foot = ground(spot);
    place_model(commands, "campfire-pit", foot, 0.0, scale);
    let ember = prop(
        commands,
        "Sphere",
        foot + Vec3::new(0.0, 0.25, 0.0),
        Vec3::new(0.3, 0.2, 0.3),
        palette.fire,
    );
    commands.add_tag_type::<Lamp>(ember);
    flame(commands, foot + Vec3::new(0.0, 0.3, 0.0), scale * 0.3);
}

pub fn tower(commands: &mut Commands, spot: Vec2, yaw: f32, levels: usize, scale: f32) {
    let foot = ground(spot);
    place_model(commands, "tower-square-base", foot, yaw, scale);
    for level in 1..levels {
        let piece = if level % 2 == 0 {
            "tower-square-mid-windows"
        } else {
            "tower-square-mid"
        };
        place_model(
            commands,
            piece,
            foot + Vec3::new(0.0, level as f32 * scale, 0.0),
            yaw,
            scale,
        );
    }
    place_model(
        commands,
        "tower-square-top-roof",
        foot + Vec3::new(0.0, levels as f32 * scale, 0.0),
        yaw,
        scale,
    );
}

pub fn spawn_farm(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    let farm = &PLACES[FARM];
    let farmhouse = farm.center + Vec2::new(4.0, -6.0);
    house(
        commands,
        palette,
        random,
        farmhouse,
        PI,
        2,
        1,
        true,
        "roof-point",
        Interior::Home,
    );
    let corner = farm.center + Vec2::new(-9.0, -1.0);
    let width = 12.0;
    let depth = 8.0;
    let segment = 2.2;
    let along_x = (width / segment) as usize;
    let along_z = (depth / segment) as usize;
    for index in 0..along_x {
        let x = corner.x + segment * 0.5 + index as f32 * segment;
        for z in [corner.y, corner.y + depth] {
            plant_model(commands, "fence_simple", Vec2::new(x, z), 0.0, segment);
        }
    }
    for index in 0..along_z {
        let z = corner.y + segment * 0.5 + index as f32 * segment;
        for x in [corner.x, corner.x + width] {
            plant_model(
                commands,
                "fence_simple",
                Vec2::new(x, z),
                FRAC_PI_2,
                segment,
            );
        }
    }
    for row in 0..4 {
        for column in 0..6 {
            let spot = corner + Vec2::new(1.4 + column as f32 * 1.8, 1.0 + row as f32 * 1.9);
            let crop = match row {
                0 | 1 => "crops_wheatStageB",
                2 => "crops_cornStageC",
                _ => ["crop_pumpkin", "crop_carrot", "crop_melon", "crop_turnip"][column % 4],
            };
            plant_model(commands, crop, spot, between(random, -0.3, 0.3), 2.2);
        }
    }
    for index in 0..3 {
        let spot = farm.center + Vec2::new(6.0 + index as f32 * 2.2, 3.0);
        plant_model(commands, "hay-bale", spot, between(random, 0.0, TAU), 2.4);
    }
    let mill = farm.center + Vec2::new(10.0, -8.0);
    tower(commands, mill, 0.0, 3, 2.4);
    place_model(
        commands,
        "windmill",
        ground(mill) + Vec3::new(0.0, 7.8, -1.5),
        0.0,
        1.9,
    );
    plant_model(
        commands,
        "bucket",
        farmhouse + Vec2::new(3.0, 2.0),
        0.0,
        2.6,
    );
    plant_model(
        commands,
        "workbench",
        farmhouse + Vec2::new(-3.6, 1.5),
        0.5,
        2.6,
    );
    plant_model(
        commands,
        "log_stack",
        farm.center + Vec2::new(-11.0, -6.0),
        0.3,
        2.6,
    );
}

pub fn spawn_barrow(commands: &mut Commands, random: &mut Random) {
    let barrow = &PLACES[BARROW];
    for _ in 0..22 {
        let spot = around(random, barrow.center, 14.5, barrow.radius * 0.95);
        let piece = [
            "gravestone-broken",
            "gravestone-round",
            "gravestone-cross",
            "debris",
            "debris-wood",
            "coffin-old",
            "rocks",
            "cross-wood",
        ][random.usize(0..8)];
        plant_model(commands, piece, spot, random.f32() * TAU, 2.2);
    }
    for _ in 0..8 {
        let spot = around(random, barrow.center, 15.0, 20.0);
        let piece = ["rocks-tall", "pillar-large", "pine-fall", "trunk-long"][random.usize(0..4)];
        plant_model(commands, piece, spot, random.f32() * TAU, 2.4);
    }
    let fence_scale = 2.0;
    for index in 0..8 {
        let angle = PI * 0.6 + index as f32 * 0.16;
        let spot = barrow.center + Vec2::new(angle.cos(), angle.sin()) * 17.0;
        let piece = if index == 4 {
            "iron-fence-border-column"
        } else {
            "iron-fence"
        };
        plant_model(commands, piece, spot, -angle + FRAC_PI_2, fence_scale);
    }
}

fn tree(commands: &mut Commands, random: &mut Random, spot: Vec2, kinds: &[&str]) {
    let kind = kinds[random.usize(0..kinds.len())];
    plant_model(
        commands,
        kind,
        spot,
        random.f32() * TAU,
        TREE_SCALE * between(random, 0.85, 1.25),
    );
}

pub fn spawn_wood(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    let kinds = [
        "tree_pineDefaultA",
        "tree_pineDefaultB",
        "tree_pineTallA",
        "tree_detailed_dark",
        "tree_oak_dark",
        "tree_default_dark",
        "tree_pineRoundA",
    ];
    let mut taken: Vec<Vec2> = Vec::new();
    let mut planted = 0;
    let mut attempts = 0;
    while planted < WOOD_TREE_COUNT && attempts < WOOD_TREE_COUNT * 12 {
        attempts += 1;
        let spot = random_place_spot(random, WOOD, 1.0);
        if road_distance(spot) < ROAD_WIDTH * 1.8
            || (spot - GIVERS[1].spot).magnitude() < 6.0
            || !far_enough(spot, &taken, WOOD_TREE_GAP)
        {
            continue;
        }
        taken.push(spot);
        planted += 1;
        tree(commands, random, spot, &kinds);
    }
    for _ in 0..30 {
        let spot = random_place_spot(random, WOOD, 0.95);
        let piece = [
            "stump_round",
            "stump_old",
            "log",
            "log_large",
            "mushroom_red",
            "mushroom_redGroup",
            "mushroom_tanGroup",
            "plant_bushDetailed",
            "rock_smallA",
            "rock_smallE",
        ][random.usize(0..10)];
        plant_model(commands, piece, spot, random.f32() * TAU, 2.5);
    }
    let hut = GIVERS[1].spot + Vec2::new(-4.5, 3.5);
    house(
        commands,
        palette,
        random,
        hut,
        2.6,
        2,
        1,
        true,
        "roof-point",
        Interior::Home,
    );
}

pub fn spawn_trees(commands: &mut Commands, random: &mut Random) {
    let kinds = [
        "tree_default",
        "tree_oak",
        "tree_detailed",
        "tree_fat",
        "tree_tall",
        "tree_simple",
        "tree_plateau",
    ];
    let mut taken: Vec<Vec2> = Vec::new();
    let mut planted = 0;
    let mut attempts = 0;
    while planted < TREE_COUNT && attempts < TREE_COUNT * 12 {
        attempts += 1;
        let spot = random_zone_spot(random);
        if near_anything_built(spot)
            || in_place(spot, WOOD, 1.1)
            || under_water(spot)
            || !far_enough(spot, &taken, TREE_GAP)
        {
            continue;
        }
        taken.push(spot);
        planted += 1;
        tree(commands, random, spot, &kinds);
    }
    let rocks = [
        "rock_largeA",
        "rock_largeB",
        "rock_largeC",
        "rock_largeD",
        "stone_largeA",
        "stone_largeB",
        "rock_tallA",
        "rock_tallC",
    ];
    let mut placed = 0;
    let mut attempts = 0;
    while placed < ROCK_COUNT && attempts < ROCK_COUNT * 10 {
        attempts += 1;
        let spot = random_zone_spot(random);
        if near_anything_built(spot) || under_water(spot) || !far_enough(spot, &taken, SCATTER_GAP)
        {
            continue;
        }
        taken.push(spot);
        placed += 1;
        plant_model(
            commands,
            rocks[random.usize(0..rocks.len())],
            spot,
            random.f32() * TAU,
            between(random, 1.8, 3.2),
        );
    }
    let bushes = [
        "plant_bush",
        "plant_bushLarge",
        "plant_bushSmall",
        "plant_bushDetailed",
        "grass_leafs",
        "flower_purpleA",
        "flower_redB",
        "flower_yellowC",
    ];
    let mut placed = 0;
    let mut attempts = 0;
    while placed < BUSH_COUNT && attempts < BUSH_COUNT * 8 {
        attempts += 1;
        let spot = random_zone_spot(random);
        if road_distance(spot) < ROAD_WIDTH
            || under_water(spot)
            || in_city(spot, 2.0)
            || !far_enough(spot, &taken, SCATTER_GAP)
        {
            continue;
        }
        taken.push(spot);
        placed += 1;
        plant_model(
            commands,
            bushes[random.usize(0..bushes.len())],
            spot,
            random.f32() * TAU,
            2.5,
        );
    }
}

pub fn spawn_reeds(commands: &mut Commands, random: &mut Random) {
    let lake = &PLACES[LAKE];
    let mut planted = 0;
    let mut attempts = 0;
    while planted < 60 && attempts < 600 {
        attempts += 1;
        let spot = around(random, lake.center, lake.radius * 0.5, lake.radius * 1.05);
        let height = terrain_height(spot.x, spot.y);
        if height < WATER_LEVEL - 0.3 || height > WATER_LEVEL + 0.8 {
            continue;
        }
        planted += 1;
        let reed = if random.bool() {
            "grass_large"
        } else {
            "grass"
        };
        place_model(
            commands,
            reed,
            Vec3::new(spot.x, height.max(WATER_LEVEL), spot.y),
            random.f32() * TAU,
            3.2,
        );
    }
}

pub fn spawn_ground_cover(commands: &mut Commands, palette: &Palette, seed: u64) {
    let tuft = palette.tuft;
    let flowers = palette.flowers.clone();
    commands.queue(move |storage, resources| {
        let mut random = Random::seeded(seed);
        register_primitive_mesh(resources, "Sphere");
        register_primitive_mesh(resources, "Cone");
        register_primitive_mesh(resources, "Cube");

        let mut tufts = Vec::new();
        let mut attempts = 0;
        while tufts.len() < GRASS_COUNT && attempts < GRASS_COUNT * 3 {
            attempts += 1;
            let spot = random_zone_spot(&mut random);
            if road_distance(spot) < ROAD_WIDTH * 0.8 || under_water(spot) || in_city(spot, 1.0) {
                continue;
            }
            let size = between(&mut random, 0.18, 0.34);
            tufts.push(InstanceTransform {
                translation: ground(spot) + Vec3::new(0.0, size * 0.6, 0.0),
                rotation: nalgebra_glm::quat_angle_axis(random.f32() * TAU, &Vec3::y()),
                scale: Vec3::new(size, size * 1.6, size),
            });
        }
        spawn_instanced_mesh_with_material(storage, resources, "Cone", tufts, tuft);

        for flower in &flowers {
            let mut blooms = Vec::new();
            let mut attempts = 0;
            while blooms.len() < FLOWER_COUNT / flowers.len() && attempts < FLOWER_COUNT {
                attempts += 1;
                let spot = random_zone_spot(&mut random);
                if road_distance(spot) < ROAD_WIDTH || under_water(spot) || in_city(spot, 1.0) {
                    continue;
                }
                blooms.push(InstanceTransform {
                    translation: ground(spot) + Vec3::new(0.0, 0.16, 0.0),
                    rotation: nalgebra_glm::quat_angle_axis(random.f32() * TAU, &Vec3::y()),
                    scale: Vec3::new(0.14, 0.32, 0.14),
                });
            }
            spawn_instanced_mesh_with_material(storage, resources, "Cube", blooms, *flower);
        }
    });
}

pub fn spawn_marker(commands: &mut Commands, palette: &Palette, parent: Entity, height: f32) {
    let glow = prop(
        commands,
        "Sphere",
        Vec3::new(0.0, height, 0.0),
        Vec3::new(0.2, 0.2, 0.2),
        palette.lantern,
    );
    commands.set(glow, ChildOf(parent));
    commands.add_tag_type::<Lamp>(glow);
}
