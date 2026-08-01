use crate::commands::board as commands_board;
use crate::commands::{effects, purse};
use crate::components::{Backing, Bar, Health, Post, Shade};
use crate::data::{self, Settings};
use crate::queries;
use crate::resources::Bulwark;
use crate::tags::{Gate, Ground};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::prelude::{Camera, OrthographicCamera, Projection};
use nightshade_config::contract::Config;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::{LocalTransform, Visibility};
use nightshade_sprite::prelude::Sprite;
use nightshade_ui::prelude::ui_button_clicks;
use nightshade_ui::resources::RetainedUiFrame;
use nightshade_waves::commands::{rewind, start, stop};
use nightshade_waves::{WavePlan, WaveProgress};

pub fn hold_the_zoom(
    config: Res<Config<Settings>>,
    bulwark: Res<Bulwark>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(camera) = bulwark.camera else {
        return;
    };
    let wanted = config.number(Settings::Zoom);
    let Some(component) = storage.get_mut::<Camera>(camera) else {
        return;
    };
    let Projection::Orthographic(current) = component.projection else {
        return;
    };
    if (current.y_mag - wanted).abs() < 0.001 {
        return;
    }
    component.projection = Projection::Orthographic(OrthographicCamera {
        x_mag: None,
        y_mag: wanted,
        z_near: theme::CAMERA_NEAR,
        z_far: theme::CAMERA_FAR,
    });
}

pub fn rock_the_camera(
    time: Res<Time>,
    mut bulwark: ResMut<Bulwark>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(camera) = bulwark.camera else {
        return;
    };
    if bulwark.shake <= 0.0 {
        return;
    }
    bulwark.shake = (bulwark.shake - time.delta_time).max(0.0);
    let share = bulwark.shake / theme::SHAKE_SECONDS;
    let swing = (bulwark.shake * theme::SHAKE_RATE).sin() * theme::SHAKE_REACH * share;
    if let Some(transform) = storage.get_mut::<LocalTransform>(camera) {
        transform.translation = theme::CAMERA_AT + Vec3::new(swing, swing * 0.4, 0.0);
    }
}

pub fn mend_the_keep(
    frame: Res<RetainedUiFrame>,
    mut bulwark: ResMut<Bulwark>,
    mut commands: Commands,
) {
    let button = bulwark.hud.mend;
    let asked = ui_button_clicks(&frame).any(|entity| entity == button);
    if !asked || !queries::mendable(&bulwark) {
        return;
    }
    purse::pay_for_the_mend(&mut bulwark);
    let at = queries::cell_center(queries::keep_cell(bulwark.board));
    effects::tower_rises(&mut commands, Vec3::new(at.x, at.y, 0.0));
}

pub fn pulse_the_gate(time: Res<Time>, storage: &mut nightshade::ecs::Storage) {
    let uptime = time.uptime_milliseconds as f32 * 0.001;
    let swing = (uptime * theme::GATE_RATE).sin() * 0.5 + 0.5;
    let alpha = theme::GATE_LOW + (theme::GATE_HIGH - theme::GATE_LOW) * swing;
    let gates: Vec<Entity> = storage.query_tag_type::<Gate>().collect();
    for gate in gates {
        if let Some(sprite) = storage.get_mut::<Sprite>(gate) {
            sprite.color = [
                theme::GATE_COLOR[0],
                theme::GATE_COLOR[1],
                theme::GATE_COLOR[2],
                alpha,
            ];
        }
    }
}

pub fn sweep_the_crystals(
    mut progress: ResMut<WaveProgress>,
    mut commands: Commands,
    walkers: Query<(&Health, &Bar, &Backing, &Shade)>,
) {
    walkers.for_each(|entity, (_, bar, backing, shade)| {
        commands.despawn(bar.0);
        commands.despawn(backing.0);
        commands.despawn(shade.0);
        commands.despawn(entity);
    });
    progress.alive = 0;
}

pub fn clear_the_board(
    config: Res<Config<Settings>>,
    mut bulwark: ResMut<Bulwark>,
    mut plan: ResMut<WavePlan>,
    mut progress: ResMut<WaveProgress>,
) {
    bulwark.taken = vec![None; (data::COLUMNS * data::ROWS) as usize];
    bulwark.selected = None;
    bulwark.coin = queries::opening_coin(config.choice(Settings::Difficulty));
    bulwark.lives = theme::STARTING_LIVES;
    bulwark.kills = 0;
    bulwark.mends = 0;
    bulwark.leaked = 0;
    bulwark.shake = 0.0;
    let rest = config.number(Settings::Rest);
    plan.0 = queries::wave_plan(rest);
    rewind(&mut progress);
    start(&mut progress);
    progress.resting = true;
    progress.rest = rest * theme::FIRST_REST_SHARE;
    bulwark.last_banner.clear();
    bulwark.last_send.clear();
    bulwark.last_mend.clear();
    bulwark.last_note.clear();
    bulwark.last_hotbar = None;
    bulwark.last_inspect = false;
    for line in &mut bulwark.last_chips {
        line.clear();
    }
}

pub fn pull_the_towers(mut commands: Commands, towers: Query<(&Post,)>) {
    towers.for_each(|entity, _| {
        commands.despawn(entity);
    });
}

pub fn rest_the_board(
    mut bulwark: ResMut<Bulwark>,
    mut progress: ResMut<WaveProgress>,
    storage: &mut nightshade::ecs::Storage,
) {
    stop(&mut progress);
    bulwark.selected = None;
    bulwark.hovered = None;
    bulwark.shake = 0.0;
    if let Some(keep) = bulwark.keep
        && let Some(sprite) = storage.get_mut::<Sprite>(keep)
    {
        sprite.color = theme::SHARD_COLOR;
    }
    if let Some(camera) = bulwark.camera
        && let Some(transform) = storage.get_mut::<LocalTransform>(camera)
    {
        transform.translation = theme::CAMERA_AT;
    }
    for helper in [bulwark.marker, bulwark.ring, bulwark.ghost]
        .into_iter()
        .flatten()
    {
        if let Some(visibility) = storage.get_mut::<Visibility>(helper) {
            visibility.visible = false;
        }
    }
}

pub fn lay_the_chosen(
    mut bulwark: ResMut<Bulwark>,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    if bulwark.laid == Some(bulwark.board) {
        return;
    }
    let laid: Vec<Entity> = storage.query_tag_type::<Ground>().collect();
    for entity in laid {
        commands.despawn(entity);
    }
    let art = bulwark.art;
    let board = bulwark.board;
    bulwark.keep = Some(commands_board::lay_the_board(&mut commands, &art, board));
    bulwark.laid = Some(board);
}

pub fn march_the_board(mut progress: ResMut<WaveProgress>) {
    start(&mut progress);
}
