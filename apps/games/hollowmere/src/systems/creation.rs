use crate::commands::audio::cue;
use crate::commands::chat::system_line;
use crate::commands::creatures::{HeroStart, spawn_player, spawn_stage_camera, spawn_stage_figure};
use crate::commands::random::random_name;
use crate::data::{Ask, CALLING_COUNT, CALLINGS, SOUND_SLOT};
use crate::queries::calling_sheet;
use crate::resources::{Creation, Cues, Hero, Icons, Journal, Player, Save, Screen};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::prelude::ThirdPersonCamera;
use nightshade_input::Actions;
use nightshade_state::prelude::NextState;
use nightshade_text::prelude::TextState;
use nightshade_ui::prelude::*;

pub fn open(
    player: Res<Player>,
    mut creation: ResMut<Creation>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
) {
    if let Some(figure) = creation.figure.take() {
        commands.despawn_recursive(figure);
    }
    if let Some(camera) = creation.camera.take() {
        commands.despawn_recursive(camera);
    }
    let figure = spawn_stage_figure(&mut commands, creation.class, player.position);
    let camera = spawn_stage_camera(&mut commands, figure);
    active_camera.0 = Some(camera);
    creation.figure = Some(figure);
    creation.camera = Some(camera);
    creation.shown_class = None;
}

pub fn choose(
    actions: Res<Actions<Ask>>,
    frame: Res<RetainedUiFrame>,
    player: Res<Player>,
    mut hero: ResMut<Hero>,
    mut creation: ResMut<Creation>,
    mut cues: ResMut<Cues>,
    mut next: ResMut<NextState<Screen>>,
    mut commands: Commands,
    mut inputs: Query<(&UiTextInputData,)>,
    mut cameras: Query<(&mut ThirdPersonCamera,)>,
) {
    let clicks: Vec<Entity> = ui_button_clicks(&frame).collect();
    let creation = &mut *creation;

    if let Some(picked) = creation
        .class_buttons
        .iter()
        .take(CALLING_COUNT)
        .position(|button| clicks.contains(button))
        && picked != creation.class
    {
        creation.class = picked;
        if let Some(old) = creation.figure {
            commands.despawn_recursive(old);
        }
        let figure = spawn_stage_figure(&mut commands, picked, player.position);
        creation.figure = Some(figure);
        if let Some(camera) = creation.camera {
            cameras.for_entity(camera, |(camera,)| camera.follow_target = Some(figure));
        }
        cue(&mut cues, SOUND_SLOT, theme::SOUND_PLAIN);
    }

    if creation.shown_class != Some(creation.class) {
        creation.shown_class = Some(creation.class);
        let text = calling_sheet(creation.class);
        ui_set_text(&mut commands, creation.title, &text.title);
        ui_set_text(&mut commands, creation.blurb, &text.blurb);
        for (line, value) in creation.stats.iter().zip(text.stats.iter()) {
            ui_set_text(&mut commands, *line, value);
        }
        for (line, value) in creation.abilities.iter().zip(text.abilities.iter()) {
            ui_set_text(&mut commands, *line, value);
        }
    }

    if clicks.contains(&creation.roll) {
        let name = random_name(&mut hero.random);
        ui_text_input_set_value(&mut commands, creation.name_input, &name);
        cue(&mut cues, SOUND_SLOT, theme::SOUND_PLAIN);
    }

    let submitted = ui_events(&frame).iter().any(|event| {
        matches!(event, UiEvent::TextInputSubmitted { entity, .. } if *entity == creation.name_input)
    });
    let entered =
        clicks.contains(&creation.enter) || actions.just_pressed(Ask::Confirm) || submitted;
    if !entered {
        return;
    }
    let mut name = String::new();
    inputs.for_entity(creation.name_input, |(data,)| {
        name = data.text.trim().to_string();
    });
    if name.is_empty() {
        name = random_name(&mut hero.random);
    }
    hero.name = name;
    hero.class = creation.class;
    hero.named = true;
    next.pending = Some(Screen::Playing);
}

pub fn enter_world(
    player: Res<Player>,
    save: Res<Save>,
    mut hero: ResMut<Hero>,
    mut journal: ResMut<Journal>,
    mut icons: ResMut<Icons>,
    mut creation: ResMut<Creation>,
    mut active_camera: ResMut<ActiveCamera>,
    mut text_state: ResMut<TextState>,
    mut commands: Commands,
) {
    if let Some(figure) = creation.figure.take() {
        commands.despawn_recursive(figure);
    }
    if let Some(camera) = creation.camera.take() {
        commands.despawn_recursive(camera);
    }
    if let Some(camera) = hero.camera {
        active_camera.0 = Some(camera);
        return;
    }
    let class = &CALLINGS[hero.class];
    icons.tint = class.accent;
    let kept = save.hero.as_ref().filter(|kept| kept.name == hero.name);
    let start = HeroStart {
        position: player.position,
        yaw: player.yaw,
        level: kept.map(|kept| kept.level).unwrap_or(1),
        experience: kept.map(|kept| kept.experience).unwrap_or(0),
        kills: kept.map(|kept| kept.kills).unwrap_or(0),
        health: kept.map(|kept| kept.health),
        mana: kept.map(|kept| kept.mana),
    };
    let (body, camera) = spawn_player(&hero, start, &mut text_state, &mut commands);
    active_camera.0 = Some(camera);
    hero.player = Some(body);
    hero.camera = Some(camera);
    system_line(
        &mut journal.chat,
        format!("Welcome to Hollowmere, {} the {}.", hero.name, class.name),
    );
}
