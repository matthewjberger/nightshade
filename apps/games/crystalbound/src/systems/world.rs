use crate::commands::roll_below;
use crate::components::{
    Facing, Health, Home, Hurt, Phase, Rise, Speed, SwingRest, TouchRest, Used,
};
use crate::contract::{MonsterKind, Place, StationKind};
use crate::data;
use crate::queries::world_of;
use crate::resources::{Field, Portal, Rig};
use crate::tags::FieldPiece;
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

const OVERWORLD_ROAMERS: u32 = 7;

pub type Walkers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut SwingRest,
        &'item mut TouchRest,
        &'item mut LocalTransform,
    ),
>;

pub fn build_the_place(
    mut field: ResMut<Field>,
    mut rig: ResMut<Rig>,
    mut commands: Commands,
    mut pieces: Query<(&LocalTransform,), WithTag<FieldPiece>>,
    mut walkers: Walkers<'_, '_>,
) {
    let Some((place, arrive)) = field.pending.take() else {
        return;
    };

    let mut stale = Vec::new();
    pieces.for_each_mut(|entity, _| stale.push(entity));
    commands.despawn_batch(stale);

    let map = map_of(place);
    let rows = map.len() as u32;
    let columns = map[0].len() as u32;

    field.place = place;
    field.rows = rows;
    field.columns = columns;
    field.blocked = vec![false; (rows * columns) as usize];
    field.portals.clear();
    field.armed = false;
    field.walked = 0.0;
    field.next_encounter = theme::ENCOUNTER_DISTANCE;

    let origin = field.origin();
    let sheet = if place == Place::Dungeon {
        rig.sheets.dungeon
    } else {
        rig.sheets.town
    };

    let mut ground = Tilemap::new(theme::ATLAS, columns, rows, theme::TILE_SIZE)
        .with_sheet(sheet, theme::SHEET_COLUMNS, theme::SHEET_ROWS)
        .with_layer(theme::LAYER_GROUND);
    let mut decor = Tilemap::new(theme::ATLAS, columns, rows, theme::TILE_SIZE)
        .with_sheet(sheet, theme::SHEET_COLUMNS, theme::SHEET_ROWS)
        .with_layer(theme::LAYER_DECOR);
    let mut water = Tilemap::new(theme::ATLAS, columns, rows, theme::TILE_SIZE)
        .with_sheet(rig.sheets.water, 1, 1)
        .with_layer(theme::LAYER_WATER);
    if place == Place::Dungeon {
        ground = ground.with_color(theme::VAULT_TINT);
        decor = decor.with_color(theme::VAULT_TINT);
    }

    let mut start = Vec2::zeros();
    let mut stations = Vec::new();

    for (line, text) in map.iter().enumerate() {
        let row = rows - 1 - line as u32;
        for (column, mark) in text.chars().enumerate() {
            let mut canvas = PlaceCanvas {
                field: &mut field,
                ground: &mut ground,
                decor: &mut decor,
                water: &mut water,
                stations: &mut stations,
                start: &mut start,
            };
            paint_tile(&mut canvas, place, mark, column as u32, row);
        }
    }

    let corner = Vec3::new(origin.x, origin.y, 0.0);
    for map in [ground, water, decor] {
        let entity = spawn_tilemap(&mut commands, map, corner);
        commands.add_tag_type::<FieldPiece>(entity);
    }

    for (kind, column, row) in stations {
        spawn_station(&mut commands, &rig, &field, kind, column, row);
    }

    let start_spot = world_of(&field, start.x as u32, start.y as u32);
    if place == Place::Overworld {
        scatter_roamers(&mut commands, &mut rig, &field, start_spot);
    }

    place_the_walker(
        &mut commands,
        &mut rig,
        &mut walkers,
        arrive.unwrap_or(start_spot),
    );
}

struct PlaceCanvas<'a> {
    field: &'a mut Field,
    ground: &'a mut Tilemap,
    decor: &'a mut Tilemap,
    water: &'a mut Tilemap,
    stations: &'a mut Vec<(StationKind, u32, u32)>,
    start: &'a mut Vec2,
}

fn paint_tile(canvas: &mut PlaceCanvas<'_>, place: Place, mark: char, column: u32, row: u32) {
    let floor = if place == Place::Dungeon {
        theme::VAULT_FLOOR[((column + row) % 3) as usize]
    } else {
        theme::GRASS
    };
    canvas
        .ground
        .set_tile(column, row, Some(TileData::new(floor)));

    match mark {
        ',' => canvas
            .ground
            .set_tile(column, row, Some(TileData::new(theme::FLOWERS))),
        'p' => canvas
            .ground
            .set_tile(column, row, Some(TileData::new(theme::DIRT))),
        '~' => {
            canvas.water.set_tile(column, row, Some(TileData::new(0)));
            block(canvas.field, column, row);
        }
        '^' => {
            canvas
                .decor
                .set_tile(column, row, Some(TileData::new(theme::MOUNTAIN)));
            block(canvas.field, column, row);
        }
        '#' => {
            if place == Place::Dungeon {
                canvas
                    .ground
                    .set_tile(column, row, Some(TileData::new(theme::VAULT_WALL)));
            } else {
                canvas
                    .decor
                    .set_tile(column, row, Some(TileData::new(theme::FENCE)));
            }
            block(canvas.field, column, row);
        }
        'T' => {
            canvas
                .decor
                .set_tile(column, row, Some(TileData::new(theme::TREE_BASE)));
            canvas
                .decor
                .set_tile(column, row + 1, Some(TileData::new(theme::TREE_MIDDLE)));
            canvas
                .decor
                .set_tile(column, row + 2, Some(TileData::new(theme::TREE_TOP)));
            block(canvas.field, column, row);
        }
        't' => {
            stamp_house(canvas, column, row, theme::ROOF_RED);
            canvas.field.portals.push(Portal {
                tile: Vec2::new(column as f32, row as f32 - 1.0),
                to: Place::Town,
            });
        }
        'd' => stamp_cave(canvas, column, row),
        '@' => {
            canvas
                .ground
                .set_tile(column, row, Some(TileData::new(theme::DIRT)));
            *canvas.start = Vec2::new(column as f32, row as f32);
        }
        'I' => {
            stamp_house(canvas, column, row, theme::ROOF_BLUE);
            canvas.stations.push((StationKind::Inn, column, row));
        }
        'S' => {
            stamp_house(canvas, column, row, theme::ROOF_RED);
            canvas.stations.push((StationKind::Shop, column, row));
        }
        'C' => {
            canvas.stations.push((StationKind::Crystal, column, row));
            block(canvas.field, column, row);
        }
        'c' => {
            canvas.stations.push((StationKind::Chest, column, row));
            block(canvas.field, column, row);
        }
        'x' => {
            let tile = if place == Place::Dungeon {
                theme::VAULT_STAIRS
            } else {
                theme::DIRT
            };
            canvas
                .ground
                .set_tile(column, row, Some(TileData::new(tile)));
            canvas.field.portals.push(Portal {
                tile: Vec2::new(column as f32, row as f32),
                to: Place::Overworld,
            });
            *canvas.start = Vec2::new(column as f32, row as f32);
        }
        _ => {}
    }
}

fn stamp_house(canvas: &mut PlaceCanvas<'_>, column: u32, row: u32, roof: [u32; 3]) {
    let columns = [column.saturating_sub(1), column, column + 1];
    let walls = [theme::WALL_PLAIN, theme::WALL_DOOR, theme::WALL_PLAIN];
    let upper = [theme::WALL_WINDOW, theme::WALL_PLAIN, theme::WALL_WINDOW];
    for (index, spot) in columns.iter().enumerate() {
        canvas
            .decor
            .set_tile(*spot, row, Some(TileData::new(walls[index])));
        canvas
            .decor
            .set_tile(*spot, row + 1, Some(TileData::new(upper[index])));
        canvas
            .decor
            .set_tile(*spot, row + 2, Some(TileData::new(roof[index])));
        block(canvas.field, *spot, row);
        block(canvas.field, *spot, row + 1);
        block(canvas.field, *spot, row + 2);
    }
}

fn stamp_cave(canvas: &mut PlaceCanvas<'_>, column: u32, row: u32) {
    let arch = [
        (column, row, theme::CAVE_LEFT),
        (column + 1, row, theme::CAVE_RIGHT),
        (column, row + 1, theme::CAVE_TOP_LEFT),
        (column + 1, row + 1, theme::CAVE_TOP_RIGHT),
    ];
    for (spot, line, tile) in arch {
        canvas.decor.set_tile(spot, line, Some(TileData::new(tile)));
        block(canvas.field, spot, line);
    }
    canvas.field.portals.push(Portal {
        tile: Vec2::new(column as f32, row as f32 - 1.0),
        to: Place::Dungeon,
    });
}

fn block(field: &mut Field, column: u32, row: u32) {
    if column >= field.columns || row >= field.rows {
        return;
    }
    field.blocked[(row * field.columns + column) as usize] = true;
}

fn map_of(place: Place) -> &'static [&'static str] {
    match place {
        Place::Overworld => &data::OVERWORLD,
        Place::Town => &data::TOWN,
        Place::Dungeon => &data::VAULT,
    }
}

fn spawn_station(
    commands: &mut Commands,
    rig: &Rig,
    field: &Field,
    kind: StationKind,
    column: u32,
    row: u32,
) {
    let spot = world_of(field, column, row);
    let entity = match kind {
        StationKind::Crystal => {
            let sprite = Sprite::new(theme::ATLAS, rig.sheets.gem)
                .with_size(1.0, 1.0)
                .with_layer(theme::LAYER_ACTOR);
            let entity = spawn_sprite(commands, sprite, Vec3::new(spot.x, spot.y, 0.0));
            commands.set(entity, Home(spot));
            commands.set(entity, Phase(0.0));
            commands.set(entity, Rise(0.18));
            entity
        }
        StationKind::Chest => {
            let sprite = Sprite::new(theme::ATLAS, rig.sheets.dungeon_tile(theme::CHEST_SHUT))
                .with_size(1.0, 1.0)
                .with_layer(theme::LAYER_ACTOR);
            spawn_sprite(commands, sprite, Vec3::new(spot.x, spot.y, 0.0))
        }
        StationKind::Inn | StationKind::Shop => commands.spawn((
            Name("Doorway".to_string()),
            LocalTransform {
                translation: Vec3::new(spot.x, spot.y, 0.0),
                ..Default::default()
            },
            GlobalTransform::default(),
        )),
    };
    commands.set(entity, kind);
    commands.set(entity, Used(false));
    commands.add_tag_type::<FieldPiece>(entity);
}

fn scatter_roamers(commands: &mut Commands, rig: &mut Rig, field: &Field, start: Vec2) {
    let mut placed = 0;
    let mut tries = 0;
    while placed < OVERWORLD_ROAMERS && tries < 400 {
        tries += 1;
        let column = roll_below(rig, field.columns);
        let row = roll_below(rig, field.rows);
        if field.is_blocked(column as i32, row as i32) {
            continue;
        }
        let spot = world_of(field, column, row);
        if (spot - start).magnitude() < 6.0 {
            continue;
        }
        let pick = roll_below(rig, MonsterKind::ROAMERS.len() as u32) as usize;
        let kind = MonsterKind::ROAMERS[pick];
        let stats = data::monster_stats(kind);
        let sprite = Sprite::new(theme::ATLAS, rig.sheets.dungeon_tile(stats.tile))
            .with_size(1.0, 1.0)
            .with_layer(theme::LAYER_ACTOR);
        let entity = spawn_sprite(commands, sprite, Vec3::new(spot.x, spot.y, 0.0));
        commands.set(entity, kind);
        commands.set(entity, Health(stats.health));
        commands.set(
            entity,
            Speed(theme::ROAMER_SPEED * (0.75 + rig.rng.f32() * 0.5)),
        );
        commands.set(entity, Hurt(0.0));
        commands.set(entity, Home(spot));
        commands.add_tag_type::<FieldPiece>(entity);
        placed += 1;
    }
}

fn place_the_walker(
    commands: &mut Commands,
    rig: &mut Rig,
    walkers: &mut Walkers<'_, '_>,
    spot: Vec2,
) {
    let landing = Vec3::new(spot.x, spot.y, 0.0);
    if let Some(entity) = rig.walker {
        walkers.for_entity(entity, |(swing_rest, touch_rest, transform)| {
            transform.translation = landing;
            swing_rest.0 = 0.0;
            touch_rest.0 = theme::TOUCH_REST;
        });
        return;
    }
    let sprite = Sprite::new(theme::ATLAS, rig.sheets.dungeon_tile(theme::HERO_TILES[0]))
        .with_size(1.0, 1.0)
        .with_layer(theme::LAYER_ACTOR);
    let entity = spawn_sprite(commands, sprite, landing);
    commands.set(entity, Speed(theme::WALK_SPEED));
    commands.set(entity, Facing(Vec2::new(0.0, -1.0)));
    commands.set(entity, SwingRest(0.0));
    commands.set(entity, TouchRest(theme::TOUCH_REST));
    rig.walker = Some(entity);
}
