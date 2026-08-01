use crate::commands::audio::cue;
use crate::commands::chat::system_line;
use crate::commands::creatures::{spawn_player, spawn_stage_figure};
use crate::commands::items::stow;
use crate::commands::random::random_name;
use crate::commands::save::restore;
use crate::components::{Experience, Kills, Level};
use crate::data::items::STARTER_KIT;
use crate::data::save::{SAVE_SLOT, Save};
use crate::data::{Ask, CALLINGS, MENU_HEIGHT, MENU_LEFT, MENU_WIDTH, SOUND_PLAIN, SOUND_SLOT};
use crate::queries::{item_index, sheet};
use crate::resources::{Creation, Cues, Icons, Panels, Screen, ThornmereResources};
use crate::systems::{hud, panels, worldmap};
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::prelude::ThirdPersonCamera;
use nightshade_input::Actions;
use nightshade_render::data::config::RendererState;
use nightshade_save::prelude::save_read;
use nightshade_scene::prelude::LocalTransform;
use nightshade_state::prelude::NextState;
use nightshade_text::prelude::TextState;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::*;

const TITLE_SIZE: f32 = 24.0;

pub fn build(
    tree: &mut UiTreeBuilder,
    style: &MenuStyle,
    menus: &mut Menus<Screen>,
    creation: &mut Creation,
    name: &str,
) {
    let kept = save_read::<Save>(SAVE_SLOT);
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(MENU_LEFT, 50.0)) + Ab(vec2(0.0, -MENU_HEIGHT / 2.0)),
            Ab(vec2(MENU_WIDTH, MENU_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_rect(style.corner_radius, style.border_width, style.border)
        .color_raw::<UiBase>(style.background)
        .flow(FlowDirection::Vertical, style.padding, style.spacing)
        .with_visible(style.visible)
        .entity();
    let first = sheet(creation.class);
    tree.in_parent(panel, |tree| {
        add_menu_heading(tree, style, "Thornmere");
        add_menu_row(
            tree,
            style,
            "Choose a calling, take a name, and enter the world.",
        );
        for (index, class) in CALLINGS.iter().enumerate() {
            creation.class_buttons[index] = tree.add_button(class.name);
        }
        creation.title = add_menu_text(tree, &first.title, TITLE_SIZE, style.heading_color);
        creation.blurb = add_menu_text(tree, &first.blurb, style.row_size, style.row_color);
        for (line, text) in creation.stats.iter_mut().zip(first.stats.iter()) {
            *line = add_menu_text(tree, text, style.row_size, style.row_color);
        }
        for (line, text) in creation.abilities.iter_mut().zip(first.abilities.iter()) {
            *line = add_menu_text(tree, text, style.row_size, style.heading_color);
        }
        creation.name_input = tree.add_text_input_with_value("Name", name);
        creation.roll = tree.add_button("Another name");
        creation.enter = tree.add_button("Enter the World");
        if let Some(kept) = &kept {
            creation.resume_button =
                tree.add_button(&format!("Return as {} of level {}", kept.name, kept.level));
        }
    });
    creation.panel = panel;
    creation.resume = kept;
    menu_show_on(menus, Screen::Creation, panel);
}

pub fn choose(
    actions: Res<Actions<Ask>>,
    frame: Res<RetainedUiFrame>,
    mut resources: ResMut<ThornmereResources>,
    mut creation: ResMut<Creation>,
    mut cues: ResMut<Cues>,
    mut next: ResMut<NextState<Screen>>,
    mut commands: Commands,
    mut inputs: Query<(&UiTextInputData,)>,
    mut cameras: Query<(&mut ThirdPersonCamera,)>,
) {
    let clicks: Vec<Entity> = ui_button_clicks(&frame).collect();
    let creation = &mut *creation;
    let resources = &mut *resources;

    if let Some(picked) = creation
        .class_buttons
        .iter()
        .position(|button| clicks.contains(button))
        && picked != creation.class
    {
        creation.class = picked;
        if let Some(old) = creation.figure {
            commands.despawn_recursive(old);
        }
        let figure = spawn_stage_figure(&mut commands, picked);
        creation.figure = Some(figure);
        if let Some(camera) = creation.camera {
            cameras.for_entity(camera, |(camera,)| camera.follow_target = Some(figure));
        }
        let text = sheet(picked);
        ui_set_text(&mut commands, creation.title, &text.title);
        ui_set_text(&mut commands, creation.blurb, &text.blurb);
        for (line, value) in creation.stats.iter().zip(text.stats.iter()) {
            ui_set_text(&mut commands, *line, value);
        }
        for (line, value) in creation.abilities.iter().zip(text.abilities.iter()) {
            ui_set_text(&mut commands, *line, value);
        }
        cue(&mut cues, SOUND_SLOT, SOUND_PLAIN);
    }

    if clicks.contains(&creation.roll) {
        let name = random_name(&mut resources.random);
        ui_text_input_set_value(&mut commands, creation.name_input, &name);
        cue(&mut cues, SOUND_SLOT, SOUND_PLAIN);
    }

    let returning = creation.resume.is_some() && clicks.contains(&creation.resume_button);
    if returning {
        let Some(kept) = creation.resume.clone() else {
            return;
        };
        resources.player_name = kept.name.clone();
        resources.class = kept.calling.min(CALLINGS.len() - 1);
        creation.class = resources.class;
        next.pending = Some(Screen::World);
        return;
    }
    let submitted = ui_events(&frame).iter().any(|event| {
        matches!(event, UiEvent::TextInputSubmitted { entity, .. } if *entity == creation.name_input)
    });
    let entered =
        clicks.contains(&creation.enter) || actions.just_pressed(Ask::Confirm) || submitted;
    if !entered {
        return;
    }
    creation.resume = None;
    let mut name = String::new();
    inputs.for_entity(creation.name_input, |(data,)| {
        name = data.text.trim().to_string();
    });
    if name.is_empty() {
        name = random_name(&mut resources.random);
    }
    resources.player_name = name;
    resources.class = creation.class;
    next.pending = Some(Screen::World);
}

pub fn enter_world(
    mut resources: ResMut<ThornmereResources>,
    mut icons: ResMut<Icons>,
    mut creation: ResMut<Creation>,
    mut active_camera: ResMut<ActiveCamera>,
    mut renderer_state: ResMut<RendererState>,
    mut text_state: ResMut<TextState>,
    mut commands: Commands,
) {
    let resources = &mut *resources;
    if let Some(figure) = creation.figure.take() {
        commands.despawn_recursive(figure);
    }
    if let Some(camera) = creation.camera.take() {
        commands.despawn_recursive(camera);
    }
    let class = &CALLINGS[resources.class];
    icons.frame_layer = icons.frame_layers.get(class.frame).copied().flatten();
    icons.tint = class.accent;
    let (player, camera) = spawn_player(resources, &mut text_state, &mut commands);
    active_camera.0 = Some(camera);
    resources.player = Some(player);
    resources.camera = Some(camera);
    if let Some(kept) = creation.resume.take() {
        restore(resources, &kept);
        renderer_state.day_night.hour = kept.hour;
        let spot = Vec3::new(kept.spot[0], kept.spot[1], kept.spot[2]);
        let level = kept.level.max(1);
        commands.set(player, Level(level));
        commands.set(player, Experience(kept.experience));
        commands.set(player, Kills(kept.kills));
        commands.queue(move |storage, _| {
            if let Some(transform) = storage.get_mut::<LocalTransform>(player) {
                transform.translation = spot;
            }
        });
        system_line(
            &mut resources.chat,
            format!(
                "Welcome back to Thornmere, {} the {}.",
                resources.player_name, class.name
            ),
        );
        return;
    }
    for (item, count) in STARTER_KIT {
        if let Some(index) = item_index(item) {
            stow(&mut resources.bag, index, count);
        }
    }
    system_line(
        &mut resources.chat,
        format!(
            "Welcome to Thornmere, {} the {}.",
            resources.player_name, class.name
        ),
    );
}

pub fn open_hud(
    mut resources: ResMut<ThornmereResources>,
    mut panels: ResMut<Panels>,
    icons: Res<Icons>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());
    let mut handles = hud::build(&mut tree, &icons, &CALLINGS[resources.class]);
    worldmap::build(&mut tree, &icons, &mut handles);
    panels::build(&mut tree, &icons, &mut panels.handles);
    panels.built = true;
    resources.hud = handles;
    tree.finish();
}
