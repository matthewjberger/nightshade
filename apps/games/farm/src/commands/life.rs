use crate::commands::world::{occupy, spawn_block};
use crate::components::{
    Arrive, Bend, Bush, Hits, Home, House, Inside, Layer, Lit, Milkable, Npc, Phase, Post, Reach,
    Rest, Roll, Shadow, Speed, Stride, Stuck, Stump, Target, Upright, Wood,
};
use crate::data::{Person, TreeKind};
use crate::queries::sheet::{dungeon_standing, standing, standing_stack};
use crate::queries::terrain::{
    center, column_of, in_region, is_occupied, is_path, is_rock, is_walkable, nearest_walkable,
    on_ground,
};
use crate::resources::{Sheet, Terrain};
use crate::tags::Rocks;
use crate::theme;
use fastrand::Rng;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::ChildOf;
use nightshade::prelude::*;
use nightshade_scene::contract::primitives::Visibility;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn spawn_standing(
    commands: &mut Commands,
    sheet: &Sheet,
    picture: (SpriteRegion, f32),
    position: Vec3,
    shadow: Vec2,
) -> (Entity, Entity) {
    let (region, height) = picture;
    let sprite = Sprite::new(theme::ATLAS, region)
        .with_size(1.0, height)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::LAYER_STANDING);
    let entity = spawn_sprite(commands, sprite, position);
    commands.set(entity, Upright::default());
    commands.set(
        entity,
        Lit {
            base: [1.0, 1.0, 1.0, 1.0],
        },
    );
    let blob = spawn_shadow(commands, sheet, entity, Vec2::new(0.0, 0.0), shadow);
    (entity, blob)
}

pub fn spawn_shadow(
    commands: &mut Commands,
    sheet: &Sheet,
    parent: Entity,
    offset: Vec2,
    size: Vec2,
) -> Entity {
    let sprite = Sprite::new(theme::ATLAS, sheet.shadow)
        .with_size(size.x, size.y)
        .with_color([0.0, 0.0, 0.0, theme::SHADOW_ALPHA])
        .with_layer(theme::LAYER_SHADOW);
    let shadow = spawn_sprite(commands, sprite, Vec3::new(0.0, 0.0, 0.0));
    commands.set(
        shadow,
        LocalTransform {
            translation: Vec3::new(
                offset.x,
                theme::SHADOW_LIFT,
                offset.y + size.y * theme::SHADOW_FORWARD,
            ),
            rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(shadow, ChildOf(parent));
    shadow
}

pub fn set_hidden(commands: &mut Commands, entity: Entity, shadow: Option<Entity>, hidden: bool) {
    commands.set(entity, Visibility { visible: !hidden });
    if let Some(shadow) = shadow {
        commands.set(shadow, Visibility { visible: !hidden });
    }
}

pub fn let_sway(commands: &mut Commands, entity: Entity, position: Vec3, strength: f32) {
    commands.set(entity, Phase(position.x * 1.7 + position.z * 2.3));
    commands.set(entity, Bend(strength));
}

fn plantable(terrain: &Terrain, column: u32, row: u32) -> bool {
    let middle = center(terrain, column, row);
    is_walkable(terrain, column, row)
        && !is_occupied(terrain, column, row)
        && !is_path(terrain, column, row)
        && !is_rock(column, row)
        && !in_region(middle, theme::VILLAGE_MIN, theme::VILLAGE_MAX)
        && !in_region(middle, theme::PEN_MIN, theme::PEN_MAX)
        && !in_region(
            middle,
            theme::PLOT_ORIGIN - Vec2::new(1.5, 1.5),
            theme::PLOT_ORIGIN
                + Vec2::new(
                    theme::PLOT_COLUMNS as f32 + 1.5,
                    theme::PLOT_ROWS as f32 + 1.5,
                ),
        )
        && (middle - theme::WELL).magnitude() > 2.5
        && (middle - theme::CAMPFIRE).magnitude() > 2.0
        && Person::ALL
            .iter()
            .all(|person| (middle - person.post()).magnitude() > 1.5)
}

pub fn spawn_tree(
    commands: &mut Commands,
    sheet: &Sheet,
    position: Vec3,
    kind: TreeKind,
) -> Entity {
    let (picture, shadow, hits, wood, sway) = match kind {
        TreeKind::Pine => (
            standing_stack(sheet, theme::PINE_TOP, theme::PINE_BOTTOM),
            Vec2::new(1.4, 0.8),
            theme::TREE_HITS,
            theme::WOOD_PER_TREE,
            1.0,
        ),
        TreeKind::Dead => (
            standing_stack(sheet, theme::DEAD_TREE_TOP, theme::DEAD_TREE_BOTTOM),
            Vec2::new(1.0, 0.6),
            theme::TREE_HITS,
            theme::WOOD_PER_TREE,
            0.5,
        ),
        TreeKind::Small => (
            standing(sheet, theme::SMALL_PINE),
            Vec2::new(1.0, 0.6),
            theme::SMALL_TREE_HITS,
            theme::WOOD_PER_SMALL_TREE,
            0.8,
        ),
    };
    let (tree, _) = spawn_standing(commands, sheet, picture, position, shadow);
    commands.set(tree, Hits(hits));
    commands.set(tree, Wood(wood));
    let_sway(commands, tree, position, sway);
    tree
}

fn plant_trees(
    commands: &mut Commands,
    terrain: &mut Terrain,
    sheet: &Sheet,
    rng: &mut Rng,
    count: u32,
    min: Vec2,
    max: Vec2,
) {
    let mut planted = 0;
    let mut tries = 0;
    while planted < count && tries < count * 20 {
        tries += 1;
        let x = min.x + rng.f32() * (max.x - min.x);
        let z = min.y + rng.f32() * (max.y - min.y);
        let Some((column, row)) = column_of(terrain, x, z) else {
            continue;
        };
        if !plantable(terrain, column, row) {
            continue;
        }
        let middle = center(terrain, column, row);
        if (middle - theme::FARMER_START).magnitude() < 3.0 {
            continue;
        }
        let kind = match planted % 5 {
            0..=2 => TreeKind::Pine,
            3 => TreeKind::Dead,
            _ => TreeKind::Small,
        };
        spawn_tree(
            commands,
            sheet,
            on_ground(terrain, middle.x, middle.y),
            kind,
        );
        occupy(terrain, column, row);
        planted += 1;
    }
}

pub fn spawn_trees(commands: &mut Commands, terrain: &mut Terrain, sheet: &Sheet) {
    let mut rng = Rng::with_seed((19u32) as u64);
    let whole_min = terrain.origin;
    let whole_max = terrain.origin + Vec2::new(terrain.columns as f32, terrain.rows as f32);
    plant_trees(commands, terrain, sheet, &mut rng, 60, whole_min, whole_max);
    plant_trees(
        commands,
        terrain,
        sheet,
        &mut rng,
        theme::FOREST_TREES,
        theme::FOREST_MIN,
        theme::FOREST_MAX,
    );
}

pub fn spawn_rocks(commands: &mut Commands, sheet: &Sheet, position: Vec3) -> Entity {
    let (rocks, _) = spawn_standing(
        commands,
        sheet,
        standing(sheet, theme::ROCK_PILE),
        position,
        Vec2::new(0.8, 0.5),
    );
    commands.add_tag_type::<Rocks>(rocks);
    rocks
}

pub fn quarry_spots(terrain: &mut Terrain) -> Vec<Vec2> {
    let mut rng = Rng::with_seed((977u32) as u64);
    let mut spots = Vec::new();
    let mut tries = 0;
    while spots.len() < theme::QUARRY_ROCKS as usize && tries < 600 {
        tries += 1;
        let x = theme::QUARRY_MIN.x
            + 0.5
            + rng.f32() * (theme::QUARRY_MAX.x - theme::QUARRY_MIN.x - 1.0);
        let z = theme::QUARRY_MIN.y
            + 0.5
            + rng.f32() * (theme::QUARRY_MAX.y - theme::QUARRY_MIN.y - 1.0);
        let Some((column, row)) = column_of(terrain, x, z) else {
            continue;
        };
        if is_occupied(terrain, column, row) {
            continue;
        }
        occupy(terrain, column, row);
        spots.push(center(terrain, column, row));
    }
    spots
}

pub fn spawn_bushes(commands: &mut Commands, terrain: &mut Terrain, sheet: &Sheet) {
    let mut rng = Rng::with_seed((313u32) as u64);
    let mut planted = 0;
    let mut tries = 0;
    while planted < theme::MEADOW_BUSHES && tries < 600 {
        tries += 1;
        let x = theme::MEADOW_MIN.x + rng.f32() * (theme::MEADOW_MAX.x - theme::MEADOW_MIN.x);
        let z = theme::MEADOW_MIN.y + rng.f32() * (theme::MEADOW_MAX.y - theme::MEADOW_MIN.y);
        let Some((column, row)) = column_of(terrain, x, z) else {
            continue;
        };
        if !plantable(terrain, column, row) {
            continue;
        }
        let middle = center(terrain, column, row);
        let position = on_ground(terrain, middle.x, middle.y);
        let (bush, _) = spawn_standing(
            commands,
            sheet,
            standing(sheet, theme::BERRY_BUSH),
            position,
            Vec2::new(0.9, 0.6),
        );
        commands.set(bush, Bush { picked_day: None });
        let_sway(commands, bush, position, 0.6);
        occupy(terrain, column, row);
        planted += 1;
    }
}

pub fn spawn_village_props(commands: &mut Commands, terrain: &mut Terrain, sheet: &Sheet) {
    for (spot, tile) in theme::VILLAGE_PROPS {
        let Some((column, row)) = column_of(terrain, spot.x, spot.y) else {
            continue;
        };
        if is_occupied(terrain, column, row) || !is_walkable(terrain, column, row) {
            continue;
        }
        let position = on_ground(terrain, spot.x, spot.y);
        let (prop, _) = spawn_standing(
            commands,
            sheet,
            standing(sheet, tile),
            position,
            Vec2::new(0.7, 0.45),
        );
        if tile == theme::SUNFLOWER || tile == theme::BUSH {
            let_sway(commands, prop, position, 0.8);
        }
        occupy(terrain, column, row);
    }
}

pub fn spawn_props(commands: &mut Commands, terrain: &mut Terrain, sheet: &Sheet) {
    let mut rng = Rng::with_seed((7u32) as u64);
    for index in 0..220 {
        let x = terrain.origin.x + rng.f32() * terrain.columns as f32;
        let z = terrain.origin.y + rng.f32() * terrain.rows as f32;
        let Some((column, row)) = column_of(terrain, x, z) else {
            continue;
        };
        if !plantable(terrain, column, row) {
            continue;
        }
        let (tile, shadow, rocks, sway) = match index % 9 {
            0 => (theme::SUNFLOWER, Vec2::new(0.5, 0.35), false, 1.0),
            1 => (theme::ROCKS, Vec2::new(0.8, 0.5), true, 0.0),
            2 => (theme::BUSH, Vec2::new(0.9, 0.6), false, 0.6),
            3 => (theme::BERRY_BUSH, Vec2::new(0.9, 0.6), false, 0.6),
            4 => (theme::ROCK_PILE, Vec2::new(0.8, 0.5), true, 0.0),
            _ => (
                theme::TUFTS[index % theme::TUFTS.len()],
                Vec2::new(0.5, 0.3),
                false,
                1.2,
            ),
        };
        let middle = center(terrain, column, row);
        let position = on_ground(terrain, middle.x, middle.y);
        let (prop, _) = spawn_standing(commands, sheet, standing(sheet, tile), position, shadow);
        if rocks {
            commands.add_tag_type::<Rocks>(prop);
            occupy(terrain, column, row);
        } else {
            let_sway(commands, prop, position, sway);
        }
        if tile == theme::BERRY_BUSH {
            commands.set(prop, Bush { picked_day: None });
            occupy(terrain, column, row);
        }
    }

    let barn = crate::data::HouseKind::Barn.house();
    let front = barn.origin.y + barn.depth as f32 + 0.5;
    for (tile, x) in [
        (theme::HAY, barn.origin.x - 0.6),
        (theme::HAY_BALE, barn.origin.x + barn.width as f32 + 0.5),
        (theme::BARREL, barn.origin.x + barn.width as f32 + 1.3),
        (theme::CRATE, barn.origin.x + 4.5),
    ] {
        spawn_standing(
            commands,
            sheet,
            standing(sheet, tile),
            on_ground(terrain, x, front),
            Vec2::new(0.9, 0.6),
        );
    }
    let bakery = crate::data::HouseKind::Bakery.house();
    for (tile, x) in [
        (theme::SACK, bakery.origin.x - 0.5),
        (theme::BASKET, bakery.origin.x + bakery.width as f32 + 0.5),
    ] {
        spawn_standing(
            commands,
            sheet,
            standing(sheet, tile),
            on_ground(terrain, x, bakery.origin.y + bakery.depth as f32 + 0.5),
            Vec2::new(0.7, 0.5),
        );
    }
}

pub fn spawn_animals(commands: &mut Commands, terrain: &Terrain, sheet: &Sheet) {
    let grazers = [
        (theme::COW, 6.0, -6.0, true),
        (theme::COW, 8.5, -7.5, true),
        (theme::SHEEP, 5.0, -4.5, false),
        (theme::SHEEP, 9.0, -4.5, false),
        (theme::SHEEP, -10.0, 14.0, false),
        (theme::SHEEP, -6.0, 17.0, false),
    ];
    for (index, (tile, x, z, cow)) in grazers.into_iter().enumerate() {
        let spot = nearest_walkable(terrain, Vec2::new(x, z));
        let (grazer, _) = spawn_standing(
            commands,
            sheet,
            standing(sheet, tile),
            on_ground(terrain, spot.x, spot.y),
            Vec2::new(0.9, 0.6),
        );
        commands.set(grazer, Target(spot));
        commands.set(grazer, Home(spot));
        commands.set(grazer, Reach(2.5));
        commands.set(grazer, Speed(0.8));
        commands.set(grazer, Rest(1.0 + index as f32));
        commands.set(grazer, Roll(Rng::with_seed((101 + index as u32) as u64)));
        if cow {
            commands.set(grazer, Milkable { milked_day: None });
        }
    }

    let chickens = [
        (-1.0, -4.0),
        (0.5, -3.0),
        (-2.5, -5.0),
        (12.0, -6.5),
        (1.5, -1.5),
    ];
    for (index, (x, z)) in chickens.into_iter().enumerate() {
        let spot = nearest_walkable(terrain, Vec2::new(x, z));
        let (chicken, _) = spawn_standing(
            commands,
            sheet,
            standing(sheet, theme::CHICKEN),
            on_ground(terrain, spot.x, spot.y),
            Vec2::new(0.6, 0.4),
        );
        commands.set(chicken, Target(spot));
        commands.set(chicken, Home(spot));
        commands.set(chicken, Reach(3.0));
        commands.set(chicken, Speed(1.6));
        commands.set(chicken, Rest(0.5 + index as f32 * 0.7));
        commands.set(chicken, Roll(Rng::with_seed((301 + index as u32) as u64)));
        commands.set(
            chicken,
            Layer {
                until_next: theme::EGG_INTERVAL * (0.3 + index as f32 * 0.25),
            },
        );
    }
}

pub fn spawn_people(commands: &mut Commands, terrain: &Terrain, sheet: &Sheet) {
    for (index, person) in Person::ALL.into_iter().enumerate() {
        let post = nearest_walkable(terrain, person.post());
        let (region, height) = dungeon_standing(sheet, person.tile());
        let size = person.size();
        let sprite = Sprite::new(theme::ATLAS, region)
            .with_size(size, height * size)
            .with_anchor(Vec2::new(0.5, 0.0))
            .with_color(person.tint())
            .with_layer(theme::LAYER_STANDING);
        let entity = spawn_sprite(commands, sprite, on_ground(terrain, post.x, post.y));
        commands.set(entity, Upright::default());
        commands.set(
            entity,
            Lit {
                base: person.tint(),
            },
        );
        let shadow = spawn_shadow(
            commands,
            sheet,
            entity,
            Vec2::new(0.0, 0.0),
            Vec2::new(0.8 * size, 0.55 * size),
        );
        commands.set(entity, Npc { person });
        commands.set(entity, Target(post));
        commands.set(entity, Home(post));
        commands.set(entity, Reach(person.reach()));
        commands.set(entity, Speed(person.speed()));
        commands.set(entity, Rest(2.0 + index as f32 * 0.5));
        commands.set(
            entity,
            Roll(Rng::with_seed((501 + index as u32 * 13) as u64)),
        );
        commands.set(
            entity,
            House(
                person
                    .home()
                    .map(|house| nearest_walkable(terrain, house.door_spot())),
            ),
        );
        commands.set(entity, Post(post));
        commands.set(entity, Arrive(person.reach()));
        commands.set(entity, Inside(false));
        commands.set(entity, Stuck(0.0));
        commands.set(entity, Shadow(Some(shadow)));
    }
}

pub fn spawn_farmer(commands: &mut Commands, terrain: &Terrain, sheet: &Sheet) -> Entity {
    let start = nearest_walkable(terrain, theme::FARMER_START);
    let (farmer, _) = spawn_standing(
        commands,
        sheet,
        standing(sheet, theme::FARMER),
        on_ground(terrain, start.x, start.y),
        Vec2::new(0.9, 0.5),
    );
    commands.set(farmer, Speed(theme::FARMER_SPEED));
    commands.set(farmer, Stride(0.0));
    commands.set(farmer, Rest(0.0));
    farmer
}

pub fn spawn_hollow_door(commands: &mut Commands, terrain: &mut Terrain, sheet: &Sheet) {
    let spot = theme::HOLLOW_DOOR;
    let position = on_ground(terrain, spot.x, spot.y);
    let (region, height) = crate::queries::sheet::dungeon_wide(sheet, theme::DOORWAY_LEFT);
    let sprite = Sprite::new(theme::ATLAS, region)
        .with_size(2.0, height * 1.2)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::LAYER_STANDING);
    let door = spawn_sprite(commands, sprite, position);
    for side in [-theme::HOLLOW_FRAME_REACH, theme::HOLLOW_FRAME_REACH] {
        for level in 0..theme::HOLLOW_FRAME_LEVELS {
            spawn_block(
                commands,
                theme::DUNGEON_WALL_SURFACES[1].0,
                position + Vec3::new(side, level as f32 + 0.5, 0.0),
                Vec3::new(1.0, 1.0, 1.0),
            );
        }
    }
    for cell in 0..theme::HOLLOW_FRAME_WIDTH {
        spawn_block(
            commands,
            theme::DUNGEON_CAP_SURFACE.0,
            position
                + Vec3::new(
                    cell as f32 + 0.5 - theme::HOLLOW_FRAME_WIDTH as f32 / 2.0,
                    theme::HOLLOW_FRAME_HEIGHT + 0.5,
                    0.0,
                ),
            Vec3::new(1.0, 1.0, 1.0),
        );
    }
    spawn_block(
        commands,
        theme::DUNGEON_ROCK_MATERIAL,
        position + Vec3::new(0.0, theme::HOLLOW_FRAME_HEIGHT / 2.0, -0.45),
        Vec3::new(
            theme::HOLLOW_FRAME_REACH * 2.0,
            theme::HOLLOW_FRAME_HEIGHT,
            0.3,
        ),
    );
    commands.set(
        door,
        Lit {
            base: [1.0, 1.0, 1.0, 1.0],
        },
    );
    for side in [-0.9, 0.9] {
        let (torch_region, torch_height) =
            crate::queries::sheet::dungeon_standing(sheet, theme::TORCH_TILE);
        let torch_sprite = Sprite::new(theme::ATLAS, torch_region)
            .with_size(0.7, torch_height * 0.7)
            .with_anchor(Vec2::new(0.5, 0.0))
            .with_layer(theme::LAYER_STANDING);
        let torch = spawn_sprite(commands, torch_sprite, position + Vec3::new(side, 0.5, 0.0));
        commands.set(
            torch,
            Lit {
                base: [1.6, 1.6, 1.6, 1.0],
            },
        );
        let light = nightshade_graphics::commands::light::spawn_light_entity(
            commands,
            position + Vec3::new(side, 1.3, 0.5),
            "Hollow torch",
        );
        commands.set(
            light,
            nightshade_render::components::light::Light {
                light_type: nightshade_render::components::light::LightType::Point,
                color: theme::TORCH_COLOR,
                intensity: theme::TORCH_STRENGTH,
                range: theme::TORCH_RANGE,
                ..Default::default()
            },
        );
        commands.set(light, Phase(side));
        commands.set(light, crate::components::Strength(theme::TORCH_STRENGTH));
        commands.set(light, crate::components::NightOnly(false));
    }
    if let Some((column, row)) = column_of(terrain, spot.x, spot.y) {
        occupy(terrain, column, row);
    }
}

pub fn spawn_stump(commands: &mut Commands, sheet: &Sheet, position: Vec3, day: u32) {
    let (stump, _) = spawn_standing(
        commands,
        sheet,
        standing(sheet, theme::STUMP),
        position,
        Vec2::new(0.6, 0.4),
    );
    commands.set(stump, Stump { since_day: day });
}
