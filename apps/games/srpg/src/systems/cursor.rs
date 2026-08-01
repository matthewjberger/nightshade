use crate::commands::audio::play;
use crate::commands::cursor::spawn_flat;
use crate::commands::show_danger;
use crate::components::{Column, CursorPart, Row};
use crate::data::{Ask, CursorPartKind, Team};
use crate::queries::{
    Units, cursor_free, cursor_shown, danger_tiles, direction, facing_toward, roster,
    tile_under_ray,
};
use crate::resources::{
    Battle, Battlefield, CameraRig, Cursor, Highlights, Phase, Screen, Sheet, Terrain,
};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_input::Actions;
use nightshade_picking::screen_ray;
use nightshade_platform::contract::input::MouseState;
use nightshade_platform::prelude::*;
use nightshade_render::resources::viewport::Viewport;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;
use nightshade_state::resources::State;

pub fn read_the_input(
    time: Res<Time>,
    battle: Res<Battle>,
    actions: Res<Actions<Ask>>,
    input: Res<Input>,
    camera_rig: Res<CameraRig>,
    terrain: Res<Terrain>,
    window: Res<Window>,
    viewport: Res<Viewport>,
    active_camera: Res<ActiveCamera>,
    mut cursor: ResMut<Cursor>,
    state_now: Res<State<Screen>>,
    storage: &mut nightshade::ecs::Storage,
) {
    let screen = state_now.current;
    let delta_time = time.delta_time;
    let phase = battle.phase;
    let free = screen == Screen::Battle && cursor_free(phase);

    let direction = direction(&actions);
    let confirm = actions.just_pressed(Ask::Confirm);
    let cancel = actions.just_pressed(Ask::Cancel)
        || input.mouse.state.contains(MouseState::RIGHT_JUST_PRESSED);
    let mouse_moved = input.mouse.state.contains(MouseState::MOVED);
    let click = input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED);
    let mouse_position = input.mouse.position;

    let (forward, right) = {
        let rig = &*camera_rig;
        (
            crate::queries::rig::snapped_forward(rig),
            crate::queries::rig::snapped_right(rig),
        )
    };
    let (columns, rows) = { (terrain.columns as i32, terrain.rows as i32) };

    let mouse_tile = if free && (mouse_moved || click) {
        let ray = screen_ray(
            &window,
            &viewport,
            &active_camera,
            storage.query_ref(),
            mouse_position,
        );
        ray.and_then(|ray| tile_under_ray(&terrain, &ray))
    } else {
        None
    };

    cursor.pulse += delta_time;
    cursor.confirm = false;
    cursor.cancel = false;
    cursor.step = (0, 0);
    if screen != Screen::Battle {
        cursor.held = None;
        return;
    }
    cursor.confirm = confirm;
    cursor.cancel = cancel;

    if direction == (0, 0) {
        cursor.held = None;
    } else {
        let step = if cursor.held != Some(direction) {
            cursor.held = Some(direction);
            cursor.repeat = theme::CURSOR_REPEAT_DELAY;
            true
        } else {
            cursor.repeat -= delta_time;
            if cursor.repeat <= 0.0 {
                cursor.repeat = theme::CURSOR_REPEAT;
                true
            } else {
                false
            }
        };
        if step {
            cursor.step = direction;
            if free {
                let column_delta = right.0 * direction.0 + forward.0 * direction.1;
                let row_delta = right.1 * direction.0 + forward.1 * direction.1;
                cursor.column = (cursor.column as i32 + column_delta).clamp(0, columns - 1) as u32;
                cursor.row = (cursor.row as i32 + row_delta).clamp(0, rows - 1) as u32;
            }
        }
    }

    if let Some((column, row)) = mouse_tile {
        cursor.hovered = Some((column, row));
        cursor.column = column;
        cursor.row = row;
        if click {
            cursor.confirm = true;
        }
    }
}

pub fn toggle_the_bars(actions: Res<Actions<Ask>>, mut battlefield: ResMut<Battlefield>) {
    if !actions.just_pressed(Ask::Bars) {
        return;
    }
    battlefield.bars_shown = !battlefield.bars_shown;
}

pub fn toggle_the_danger(
    actions: Res<Actions<Ask>>,
    mut battle: ResMut<Battle>,
    terrain: Res<Terrain>,
    mut highlights: ResMut<Highlights>,
    mut units: Units<'_, '_>,
) {
    if !actions.just_pressed(Ask::Danger) {
        return;
    }
    let shown = {
        battle.danger_shown = !battle.danger_shown;
        battle.danger_shown
    };
    let tiles = if shown {
        let roster = roster(&mut units);
        danger_tiles(&terrain, &roster, Team::Player)
    } else {
        Vec::new()
    };
    show_danger(&mut highlights, tiles);
}

pub fn show_the_cursor(
    state: Res<State<Screen>>,
    battle: Res<Battle>,
    cursor: Res<Cursor>,
    terrain: Res<Terrain>,
    rig: Res<CameraRig>,
    mut units: Query<(&Column, &Row, &Team)>,
    parts: Query<(
        &CursorPart,
        &mut LocalTransform,
        &mut Sprite,
        &mut Visibility,
    )>,
) {
    let camp = state.current != Screen::Battle;
    let shown = !camp && cursor_shown(battle.phase);
    let stand = crate::queries::terrain::stand(&terrain, cursor.column, cursor.row);
    let flat = nalgebra_glm::quat_angle_axis(rig.yaw, &Vec3::y())
        * nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x());
    let mut foot = None;
    let mut ring_color = theme::RING_PLAYER;
    if !camp && let Some(active) = battle.active {
        units.for_entity(active, |(column, row, team)| {
            foot = Some(crate::queries::terrain::stand(&terrain, column.0, row.0));
            ring_color = match team {
                Team::Player => theme::RING_PLAYER,
                Team::Enemy => theme::RING_ENEMY,
            };
        });
    }
    let ring_shown = foot.is_some() && !matches!(battle.phase, Phase::Ticking | Phase::TurnEnd);
    let glow = 0.75 + 0.25 * (cursor.pulse * theme::CURSOR_PULSE).sin();
    let bob = (cursor.pulse * theme::CURSOR_PULSE * 0.7).sin() * theme::ARROW_BOB;

    parts.for_each(|_entity, (part, transform, sprite, visibility)| {
        let visible = match part.kind {
            CursorPartKind::Ring => ring_shown,
            _ => shown,
        };
        if visibility.visible != visible {
            visibility.visible = visible;
        }
        match part.kind {
            CursorPartKind::Bracket => {
                transform.translation = stand + Vec3::new(0.0, theme::CURSOR_LIFT, 0.0);
                transform.rotation = flat;
                sprite.color = [1.0, 1.0, 1.0, theme::CURSOR_COLOR[3] * glow];
            }
            CursorPartKind::Arrow => {
                transform.translation = stand + Vec3::new(0.0, theme::ARROW_HEIGHT + bob, 0.0);
            }
            CursorPartKind::Ring => {
                if let Some(foot) = foot {
                    transform.translation = foot + Vec3::new(0.0, theme::MARKER_LIFT + 0.012, 0.0);
                    transform.rotation = flat;
                    sprite.color = ring_color;
                }
            }
        }
    });
}

struct Layer<'a> {
    tiles: &'a [(u32, u32)],
    color: [f32; 4],
    lift: f32,
    tile: Option<u32>,
}

pub fn show_the_highlights(
    mut highlights: ResMut<Highlights>,
    terrain: Res<Terrain>,
    sheet: Res<Sheet>,
    mut commands: Commands,
) {
    if highlights.version == highlights.shown_version {
        return;
    }
    highlights.shown_version = highlights.version;
    for marker in highlights.markers.drain(..) {
        commands.despawn(marker);
    }
    let layers = [
        Layer {
            tiles: &highlights.danger_tiles,
            color: theme::DANGER_COLOR,
            lift: -0.006,
            tile: Some(theme::HAZARD_TILE),
        },
        Layer {
            tiles: &highlights.move_tiles,
            color: theme::MOVE_COLOR,
            lift: 0.0,
            tile: None,
        },
        Layer {
            tiles: &highlights.target_tiles,
            color: theme::TARGET_COLOR,
            lift: 0.0,
            tile: None,
        },
        Layer {
            tiles: &highlights.area_tiles,
            color: theme::AREA_COLOR,
            lift: 0.008,
            tile: None,
        },
    ];
    let mut markers = Vec::new();
    for Layer {
        tiles,
        color,
        lift,
        tile,
    } in layers
    {
        for (column, row) in tiles {
            let region = tile.map_or(sheet.cell, |tile| crate::queries::sheet::tile(&sheet, tile));
            let sprite = Sprite::new(theme::ATLAS, region)
                .with_size(0.94, 0.94)
                .with_color(color)
                .with_layer(theme::LAYER_GROUND);
            markers.push(spawn_flat(
                &mut commands,
                sprite,
                crate::queries::terrain::stand(&terrain, *column, *row)
                    + Vec3::new(0.0, theme::MARKER_LIFT + lift, 0.0),
            ));
        }
    }

    let path = &highlights.path_tiles;
    for (step, tile) in path.iter().enumerate().skip(1) {
        let Some(facing) = facing_toward(path[step - 1], *tile) else {
            continue;
        };
        let picture = if step + 1 == path.len() {
            facing.arrow_tip()
        } else {
            facing.arrow_shaft()
        };
        let sprite = Sprite::new(theme::ATLAS, crate::queries::sheet::tile(&sheet, picture))
            .with_size(1.0, 1.0)
            .with_color(theme::CURSOR_COLOR)
            .with_layer(theme::LAYER_GROUND);
        markers.push(spawn_flat(
            &mut commands,
            sprite,
            crate::queries::terrain::stand(&terrain, tile.0, tile.1)
                + Vec3::new(0.0, theme::MARKER_LIFT + 0.004, 0.0),
        ));
    }
    highlights.markers = markers;
}

pub fn sound_the_cursor(
    state: Res<State<Screen>>,
    battle: Res<Battle>,
    mut cursor: ResMut<Cursor>,
    mut commands: Commands,
) {
    if state.current != Screen::Battle || !cursor_shown(battle.phase) {
        cursor.sounded = None;
        return;
    }
    let tile = (cursor.column, cursor.row);
    if cursor.sounded == Some(tile) {
        return;
    }
    if cursor.sounded.is_some() {
        play(&mut commands, theme::SOUND_STEP, theme::SOUND_QUIET);
    }
    cursor.sounded = Some(tile);
}
