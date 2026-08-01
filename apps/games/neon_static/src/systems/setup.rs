use crate::commands::{load_floor_plan, pack_sheets};
use crate::components::{Angle, Cooldown, Life, Speed, Spin, Velocity};
use crate::data;
use crate::resources::{Floor, NeonStaticResources, Sheets};
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_sprite::prelude::*;

pub fn initialize(
    mut resources: ResMut<NeonStaticResources>,
    mut settings: ResMut<RenderSettings>,
    mut atlases: ResMut<SpriteAtlases>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
) {
    let resources = &mut *resources;
    settings.atmosphere = Atmosphere::None;
    settings.clear_color = [0.03, 0.01, 0.06, 1.0];
    spawn_sun_without_shadows(&mut commands);

    resources.sheets = pack_sheets(&mut atlases);
    load_floor_plan(&mut resources.floor);

    lay_the_floor(&resources.sheets, &resources.floor, &mut commands);

    let camera = nightshade_camera::commands::scene::spawn_flat_camera(
        &mut commands,
        "Floor Camera".to_string(),
        Vec3::new(resources.floor.start.x, resources.floor.start.y, 20.0),
        9.0,
        0.1,
        100.0,
    );
    resources.camera = Some(camera);
    resources.camera_anchor = resources.floor.start;
    resources.camera_position = resources.floor.start;
    active_camera.0 = Some(camera);

    spawn_actors(resources, &mut commands);
}

pub fn spawn_actors(resources: &mut NeonStaticResources, commands: &mut Commands) {
    let player = spawn_player(&resources.sheets, resources.floor.start, commands);
    resources.player = Some(player);
    resources.player_position = resources.floor.start;
    resources.round.push(player);

    let spawns = resources.floor.spawns.clone();
    for (index, spot) in spawns.into_iter().enumerate() {
        let enemy = spawn_enemy(&resources.sheets, index, spot, commands);
        resources.round.push(enemy);
    }
}

fn lay_the_floor(sheets: &Sheets, floor: &Floor, commands: &mut Commands) {
    let mut ground = Tilemap::new(data::ATLAS, floor.columns, floor.rows, data::TILE_SIZE)
        .with_sheet(
            sheets.tiles,
            data::TILE_SHEET_COLUMNS,
            data::TILE_SHEET_ROWS,
        )
        .with_layer(data::LAYER_FLOOR)
        .with_color(data::FLOOR_TINT);
    let mut walls = Tilemap::new(data::ATLAS, floor.columns, floor.rows, data::TILE_SIZE)
        .with_sheet(
            sheets.tiles,
            data::TILE_SHEET_COLUMNS,
            data::TILE_SHEET_ROWS,
        )
        .with_layer(data::LAYER_WALL)
        .with_color(data::WALL_TINT);

    for row in 0..floor.rows {
        for column in 0..floor.columns {
            let mixed = (column * 7 + row * 13) % 11 == 0;
            let ground_tile = if mixed {
                data::FLOOR_TILE_MARKED
            } else {
                data::FLOOR_TILE
            };
            ground.set_tile(column, row, Some(TileData::new(ground_tile)));
            if floor.solid[(row * floor.columns + column) as usize] {
                let wall_tile = if (column + row) % 2 == 0 {
                    data::WALL_TILE
                } else {
                    data::WALL_TILE_ALTERNATE
                };
                walls.set_tile(column, row, Some(TileData::new(wall_tile)));
            }
        }
    }

    spawn_tilemap(commands, ground, Vec3::zeros());
    spawn_tilemap(commands, walls, Vec3::zeros());
}

fn spawn_player(sheets: &Sheets, at: Vec2, commands: &mut Commands) -> Entity {
    let entity = spawn_sprite(
        commands,
        character_sprite(sheets, &data::PLAYER_POSE),
        Vec3::new(at.x, at.y, 0.0),
    );
    commands.set(entity, Angle(0.0));
    commands.set(entity, Speed(data::PLAYER_SPEED));
    commands.set(entity, Cooldown(0.0));
    entity
}

fn spawn_enemy(sheets: &Sheets, index: usize, at: Vec2, commands: &mut Commands) -> Entity {
    let pose = &data::ENEMY_POSES[index % data::ENEMY_POSES.len()];
    let entity = spawn_sprite(
        commands,
        character_sprite(sheets, pose),
        Vec3::new(at.x, at.y, 0.0),
    );
    commands.set(entity, Angle(0.0));
    commands.set(entity, Speed(data::ENEMY_SPEED));
    entity
}

pub fn spawn_bullet(sheets: &Sheets, at: Vec2, direction: Vec2, commands: &mut Commands) -> Entity {
    let sprite = Sprite::new(data::ATLAS, sheets.tile(data::WHITE_TILE))
        .with_size(data::BULLET_SIZE.x, data::BULLET_SIZE.y)
        .with_color(data::BULLET_TINT)
        .with_layer(data::LAYER_BULLET);
    let entity = spawn_sprite(commands, sprite, Vec3::new(at.x, at.y, 0.0));
    commands.set(entity, Velocity(direction * data::BULLET_SPEED));
    commands.set(entity, Life(data::BULLET_LIFE));
    turn(commands, entity, at, direction.y.atan2(direction.x));
    entity
}

pub fn spawn_shell(
    sheets: &Sheets,
    at: Vec2,
    velocity: Vec2,
    spin: f32,
    commands: &mut Commands,
) -> Entity {
    let sprite = Sprite::new(data::ATLAS, sheets.tile(data::SHARD_TILE))
        .with_size(data::SHELL_SIZE.x, data::SHELL_SIZE.y)
        .with_color(data::SHELL_TINT)
        .with_layer(data::LAYER_BLOOD);
    let entity = spawn_sprite(commands, sprite, Vec3::new(at.x, at.y, 0.0));
    commands.set(entity, Velocity(velocity));
    commands.set(entity, Spin(spin));
    commands.set(entity, Angle(0.0));
    commands.set(entity, Life(data::SHELL_LIFE));
    entity
}

pub fn spawn_blood(sheets: &Sheets, at: Vec2, turn_by: f32, commands: &mut Commands) -> Entity {
    let sprite = Sprite::new(data::ATLAS, sheets.tile(data::BLOOD_TILE))
        .with_size(data::BLOOD_SIZE.x, data::BLOOD_SIZE.y)
        .with_color(data::BLOOD_TINT)
        .with_layer(data::LAYER_BLOOD);
    let entity = spawn_sprite(commands, sprite, Vec3::new(at.x, at.y, 0.0));
    turn(commands, entity, at, turn_by);
    entity
}

fn turn(commands: &mut Commands, entity: Entity, at: Vec2, angle: f32) {
    commands.set(
        entity,
        LocalTransform {
            translation: Vec3::new(at.x, at.y, 0.0),
            rotation: nalgebra_glm::quat_angle_axis(angle, &Vec3::z()),
            ..Default::default()
        },
    );
}

fn character_sprite(sheets: &Sheets, pose: &data::CharacterPose) -> Sprite {
    Sprite::new(data::ATLAS, sheets.pose(pose))
        .with_size(
            pose.width / data::TEXELS_PER_TILE,
            data::CHARACTER_HEIGHT / data::TEXELS_PER_TILE,
        )
        .with_anchor(Vec2::new(data::CHARACTER_BODY_CENTER / pose.width, 0.5))
        .with_layer(data::LAYER_BODY)
}
