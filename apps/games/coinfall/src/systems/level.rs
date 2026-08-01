use crate::commands::reset_run;
use crate::components::{Alive, Direction, FacingLeft, Home, Parallax, Phase, Speed, Velocity};
use crate::data::{self, LevelSpec};
use crate::queries;
use crate::resources::{CoinfallResources, Sheets};
use crate::tags::{Flag, LevelPiece};
use crate::theme;
use nightshade::prelude::*;
use nightshade_pickup::{Collector, Middle, Pickup, PickupCount, Taken};
use nightshade_platformer::{Airborne, Coyote, JumpAsked};
use nightshade_scene::prelude::LocalTransform;
use nightshade_sprite::prelude::*;
use nightshade_tiles::prelude::read;

pub type Pieces<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item LocalTransform,), WithTag<LevelPiece>>;

pub fn load(
    mut resources: ResMut<CoinfallResources>,
    mut commands: Commands,
    mut pieces: Pieces<'_, '_>,
) {
    if resources.run.wanted_level.take().is_none() {
        return;
    }

    let mut old = Vec::new();
    pieces.for_each_mut(|entity, _| old.push(entity));
    for entity in old {
        commands.despawn_recursive(entity);
    }

    let level = queries::current_level(&resources.run);
    let sheets = resources.sheets;
    resources.solids = read(level.rows);

    spawn_sky(&sheets, level, &mut commands);
    let player = spawn_terrain(level, &sheets, &mut commands);
    resources.player = Some(player);
    reset_run(&mut resources.run);
}

fn spawn_terrain(level: &'static LevelSpec, sheets: &Sheets, commands: &mut Commands) -> Entity {
    let columns = queries::level_columns(level);
    let rows = queries::level_rows(level);
    let mut tiles = Tilemap::new(theme::ATLAS, columns as u32, rows as u32, theme::TILE_SIZE)
        .with_sheet(sheets.tiles, theme::TILE_COLUMNS, theme::TILE_ROWS)
        .with_layer(theme::LAYER_TILES)
        .with_color(level.palette.tint);
    let mut player = Entity::default();

    for row in 0..rows {
        for column in 0..columns {
            let mark = queries::level_cell(level, column, row);
            match mark {
                data::GROUND => {
                    let picture = queries::ground_picture(level, column, row);
                    tiles.set_tile(column as u32, row as u32, Some(TileData::new(picture)));
                }
                data::BLOCK => {
                    tiles.set_tile(column as u32, row as u32, Some(TileData::new(theme::CRATE)));
                }
                data::COIN => spawn_coin(sheets, commands, column, row),
                data::WALKER => spawn_walker(sheets, level, commands, column, row),
                data::START => player = spawn_player(sheets, commands, column, row),
                data::FLAG => spawn_flag(sheets, commands, column, row),
                _ => {
                    if let Some(picture) = queries::prop_picture(mark) {
                        spawn_prop(sheets, commands, picture, column, row);
                    }
                }
            }
        }
    }

    let map = spawn_tilemap(commands, tiles, Vec3::zeros());
    commands.add_tag_type::<LevelPiece>(map);
    player
}

fn spawn_sky(sheets: &Sheets, level: &'static LevelSpec, commands: &mut Commands) {
    let columns = queries::level_columns(level) as u32;
    let mut sky = Tilemap::new(
        theme::ATLAS,
        columns,
        theme::SKY_ROWS,
        Vec2::new(theme::SKY_TILE, theme::SKY_TILE),
    )
    .with_sheet(
        sheets.backgrounds,
        data::BACKGROUND_COLUMNS,
        data::BACKGROUND_ROWS,
    )
    .with_layer(theme::LAYER_SKY);

    for column in 0..columns {
        let choice = level.palette.background[(column % 2) as usize];
        for row in 0..theme::SKY_ROWS {
            let picture = if row < theme::SKY_HORIZON {
                choice + data::BACKGROUND_COLUMNS * 2
            } else if row == theme::SKY_HORIZON {
                choice + data::BACKGROUND_COLUMNS
            } else {
                choice
            };
            sky.set_tile(column, row, Some(TileData::new(picture)));
        }
    }

    let entity = spawn_tilemap(commands, sky, Vec3::new(0.0, theme::SKY_ORIGIN, 0.0));
    commands.set(entity, Parallax(theme::SKY_PARALLAX));
    commands.add_tag_type::<LevelPiece>(entity);
}

fn cell_center(column: i32, row: i32) -> Vec3 {
    Vec3::new(column as f32 + 0.5, row as f32 + 0.5, 0.0)
}

fn cell_floor(column: i32, row: i32) -> Vec3 {
    Vec3::new(column as f32 + 0.5, row as f32, 0.0)
}

fn spawn_coin(sheets: &Sheets, commands: &mut Commands, column: i32, row: i32) {
    let home = cell_center(column, row);
    let sprite = Sprite::new(theme::ATLAS, queries::tile_region(sheets, theme::COIN))
        .with_size(theme::TILE, theme::TILE)
        .with_layer(theme::LAYER_PICKUPS);
    let entity = spawn_sprite(commands, sprite, home);
    commands.set(entity, Home(home));
    commands.set(entity, Phase(column as f32 * 0.6));
    commands.set(entity, Taken(false));
    commands.set(entity, Pickup(theme::COIN_KIND));
    commands.set(entity, PickupCount(1));
    commands.add_tag_type::<LevelPiece>(entity);
}

fn spawn_prop(sheets: &Sheets, commands: &mut Commands, picture: u32, column: i32, row: i32) {
    let sprite = Sprite::new(theme::ATLAS, queries::tile_region(sheets, picture))
        .with_size(theme::TILE, theme::TILE)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::LAYER_PROPS);
    let entity = spawn_sprite(commands, sprite, cell_floor(column, row));
    commands.add_tag_type::<LevelPiece>(entity);
}

fn spawn_flag(sheets: &Sheets, commands: &mut Commands, column: i32, row: i32) {
    let home = cell_floor(column, row);
    let sprite = Sprite::new(theme::ATLAS, queries::tile_region(sheets, theme::FLAG))
        .with_size(theme::TILE, theme::TILE)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::LAYER_PICKUPS);
    let entity = spawn_sprite(commands, sprite, home);
    commands.set(entity, Home(home));
    commands.add_tag_type::<Flag>(entity);
    commands.add_tag_type::<LevelPiece>(entity);
}

fn spawn_walker(
    sheets: &Sheets,
    level: &'static LevelSpec,
    commands: &mut Commands,
    column: i32,
    row: i32,
) {
    let frames = level.palette.walker;
    let home = cell_floor(column, row);
    let sprite = Sprite::new(theme::ATLAS, queries::character_region(sheets, frames[0]))
        .with_size(theme::CHARACTER_TILES, theme::CHARACTER_TILES)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::LAYER_ACTORS);
    let entity = spawn_sprite(commands, sprite, home);
    commands.set(
        entity,
        SpriteAnimator::from_regions(
            frames
                .iter()
                .map(|frame| queries::character_region(sheets, *frame)),
            theme::WALKER_FRAME_SECONDS,
        ),
    );
    commands.set(entity, Home(home));
    commands.set(entity, Speed(theme::WALKER_SPEED));
    commands.set(entity, Direction(-1.0));
    commands.set(entity, Alive(true));
    commands.add_tag_type::<LevelPiece>(entity);
}

fn spawn_player(sheets: &Sheets, commands: &mut Commands, column: i32, row: i32) -> Entity {
    let home = cell_floor(column, row);
    let sprite = Sprite::new(
        theme::ATLAS,
        queries::character_region(sheets, theme::PLAYER_FRAMES[0]),
    )
    .with_size(theme::CHARACTER_TILES, theme::CHARACTER_TILES)
    .with_anchor(Vec2::new(0.5, 0.0))
    .with_layer(theme::LAYER_ACTORS);
    let entity = spawn_sprite(commands, sprite, home);
    let mut walk = SpriteAnimator::from_regions(
        theme::PLAYER_FRAMES
            .iter()
            .map(|frame| queries::character_region(sheets, *frame)),
        theme::WALK_FRAME_SECONDS,
    );
    walk.pause();
    commands.set(entity, walk);
    commands.set(entity, Home(home));
    commands.set(entity, Velocity::default());
    commands.set(entity, Airborne(true));
    commands.set(entity, Coyote(0.0));
    commands.set(entity, JumpAsked(0.0));
    commands.set(entity, Collector(theme::COIN_GRASP));
    commands.set(
        entity,
        Middle(Vec3::new(0.0, theme::PLAYER_HEIGHT * 0.5, 0.0)),
    );
    commands.set(entity, FacingLeft(false));
    commands.add_tag_type::<LevelPiece>(entity);
    entity
}
