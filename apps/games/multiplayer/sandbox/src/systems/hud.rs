use crate::resources::{Role, Screen, Session, UiHandles, Wire};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiInteraction;
use nightshade_ui_menu::prelude::*;

const HUD_WIDTH: f32 = 300.0;
const HUD_HEIGHT: f32 = 322.0;
const MENU_WIDTH: f32 = 420.0;
const MENU_HEIGHT: f32 = 320.0;
const ROW_HEIGHT: f32 = 6.0;

pub fn menu_style() -> MenuStyle {
    MenuStyle {
        background: theme::PANEL_BACKGROUND,
        border: theme::PANEL_BORDER,
        corner_radius: 8.0,
        border_width: 1.0,
        padding: 12.0,
        spacing: 8.0,
        visible: true,
        ..Default::default()
    }
}

fn row(tree: &mut UiTreeBuilder, text: &str, size: f32) -> Entity {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, size + ROW_HEIGHT)))
        .with_text(text, size)
        .color_raw::<UiBase>(theme::TEXT_PRIMARY)
        .entity()
}

fn dim_row(tree: &mut UiTreeBuilder, text: &str, size: f32) -> Entity {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, size + ROW_HEIGHT)))
        .with_text(text, size)
        .color_raw::<UiBase>(theme::TEXT_DIM)
        .entity()
}

pub fn build(
    wire: Res<Wire>,
    mut handles: ResMut<UiHandles>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let role = match wire.role {
        Role::Host => "hosting",
        Role::Guest => "joining",
    };
    let built = {
        let mut tree = UiTreeBuilder::new(build.context());
        let root = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .entity();

        let built = tree.in_parent(root, |tree| {
            let mut made = UiHandles::default();

            let title = add_menu_panel(tree, &menu_style(), MENU_WIDTH, MENU_HEIGHT);
            made.title_root = Some(title);
            tree.in_parent(title, |tree| {
                row(tree, theme::TITLE, theme::HEADING_TEXT_SIZE);
                dim_row(
                    tree,
                    "WASD to walk, space to jump, C to crouch, mouse to look.",
                    theme::PANEL_TEXT_SIZE,
                );
                dim_row(
                    tree,
                    "Escape pauses. Enter opens the chat line.",
                    theme::PANEL_TEXT_SIZE,
                );
                dim_row(tree, role, theme::PANEL_TEXT_SIZE);
                made.title_play = Some(tree.add_button("Play"));
                made.title_quit = Some(tree.add_button("Quit"));
            });

            let style = menu_style();
            let hud = tree
                .add_node()
                .window(
                    Rl(vec2(0.0, 0.0)) + Ab(vec2(16.0, 16.0)),
                    Ab(vec2(HUD_WIDTH, HUD_HEIGHT)),
                    Anchor::TopLeft,
                )
                .with_rect(style.corner_radius, style.border_width, style.border)
                .color_raw::<UiBase>(style.background)
                .flow(FlowDirection::Vertical, style.padding, 2.0)
                .entity();
            made.hud_root = Some(hud);
            tree.in_parent(hud, |tree| {
                made.hud_status = Some(dim_row(tree, "", theme::PANEL_TEXT_SIZE));
                made.hud_note = Some(dim_row(tree, "", theme::PANEL_TEXT_SIZE));
                made.hud_roster = (0..theme::ROSTER_LINES)
                    .map(|_| row(tree, "", theme::PANEL_TEXT_SIZE))
                    .collect();
                made.hud_chat = (0..theme::CHAT_LINES)
                    .map(|_| dim_row(tree, "", theme::PANEL_TEXT_SIZE))
                    .collect();
                made.hud_input = Some(tree.add_text_input("Say something and press enter"));
            });

            let pause = add_menu_panel(tree, &menu_style(), MENU_WIDTH, MENU_HEIGHT);
            made.pause_root = Some(pause);
            tree.in_parent(pause, |tree| {
                row(tree, "Paused", theme::HEADING_TEXT_SIZE);
                dim_row(tree, "Q closes the app.", theme::PANEL_TEXT_SIZE);
                made.pause_predict = Some(tree.add_checkbox("Predict my own moves", true));
                made.pause_interpolate = Some(tree.add_checkbox("Smooth the other players", true));
                made.pause_resume = Some(tree.add_button("Resume"));
                made.pause_quit = Some(tree.add_button("Quit"));
            });

            made
        });

        tree.finish();
        built
    };
    *handles = built;
}

pub fn refresh(
    wire: Res<Wire>,
    time: Res<Time>,
    session: Res<Session>,
    handles: Res<UiHandles>,
    interaction: Res<RetainedUiInteraction>,
    mut commands: Commands,
) {
    let role = match wire.role {
        Role::Host => "host",
        Role::Guest => "guest",
    };
    let status = format!(
        "{role}  peers {}  tick {}  {:.0} fps",
        session.seats.len().max(1),
        session.tick,
        time.frames_per_second,
    );
    if let Some(line) = handles.hud_status {
        ui_set_text(&mut commands, line, &status);
    }
    if let Some(line) = handles.hud_note {
        ui_set_text(&mut commands, line, &session.note);
    }

    for (index, line) in handles.hud_roster.iter().enumerate() {
        let text = match session.seats.get(index) {
            Some(seat) if seat.peer == wire.peer => format!("{} (you)", seat.name),
            Some(seat) => seat.name.clone(),
            None => String::new(),
        };
        ui_set_text(&mut commands, *line, &text);
    }

    let first = session.chat.len().saturating_sub(handles.hud_chat.len());
    for (index, line) in handles.hud_chat.iter().enumerate() {
        let text = session.chat.get(first + index).cloned().unwrap_or_default();
        ui_set_text(&mut commands, *line, &text);
    }

    let _ = interaction;
}

pub fn chat(
    frame: Res<RetainedUiFrame>,
    handles: Res<UiHandles>,
    interaction: Res<RetainedUiInteraction>,
    wire: Res<Wire>,
    mut session: ResMut<Session>,
    mut commands: Commands,
    mut fields: Query<(&mut nightshade_ui::prelude::UiTextInputData,)>,
) {
    let Some(field) = handles.hud_input else {
        return;
    };
    session.typing = interaction.focused_entity == Some(field);

    let submitted = ui_events(&frame).iter().any(
        |event| matches!(event, UiEvent::TextInputSubmitted { entity, .. } if *entity == field),
    );
    if !session.typing || !submitted {
        return;
    }

    let mut said = String::new();
    fields.for_entity(field, |(data,)| {
        said = data.text.trim().to_string();
        data.text.clear();
        data.cursor_position = 0;
    });
    if said.is_empty() {
        return;
    }
    ui_set_text(&mut commands, field, "");

    match wire.role {
        Role::Host => {
            let peer = wire.peer;
            session.chat.push(format!("you: {said}"));
            let _ = wire.to_net.send(crate::resources::Outgoing::Everyone(
                crate::data::wire::Update::Chat { peer, text: said },
            ));
        }
        Role::Guest => {
            session.chat.push(format!("you: {said}"));
            let _ = wire.to_net.send(crate::resources::Outgoing::Host(
                crate::data::wire::Request::Chat { text: said },
            ));
        }
    }
}

pub fn switches(
    handles: Res<UiHandles>,
    mut session: ResMut<Session>,
    mut boxes: Query<(&nightshade_ui::prelude::UiCheckboxData,)>,
) {
    if let Some(entity) = handles.pause_predict {
        boxes.for_entity(entity, |(data,)| session.predict = data.value);
    }
    if let Some(entity) = handles.pause_interpolate {
        boxes.for_entity(entity, |(data,)| session.interpolate = data.value);
    }
    let _ = Screen::Playing;
}
