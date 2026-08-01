use crate::contract::Dimension;
use crate::data;
use crate::resources::{FourfoldResources, HudHandles};
use nightshade::prelude::*;
use nightshade_state::prelude::State;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

const PANEL_BACKGROUND: Vec4 = Vec4::new(0.03, 0.03, 0.05, 0.62);
const PANEL_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.08);
const TEXT_COLOR: Vec4 = Vec4::new(0.74, 0.78, 0.88, 1.0);
const PROMPT_COLOR: Vec4 = Vec4::new(0.60, 0.70, 0.82, 1.0);
const CROSSHAIR_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.45);
const CLEAR_COLOR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);

const MARGIN: f32 = 18.0;
const PANEL_WIDTH: f32 = 176.0;
const PANEL_HEIGHT: f32 = 40.0;
const BAND_WIDTH: f32 = 1060.0;
const LINE_SIZE: f32 = 22.0;
const SPEAKER_SIZE: f32 = 14.0;
const FADE_IN: f32 = 0.26;
const FADE_OUT: f32 = 0.48;

pub fn build(tree: &mut UiTreeBuilder) -> HudHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut handles = HudHandles::default();

    tree.in_parent(root, |tree| {
        let panel = tree
            .add_node()
            .window(
                Ab(vec2(MARGIN, MARGIN)),
                Ab(vec2(PANEL_WIDTH, PANEL_HEIGHT)),
                Anchor::TopLeft,
            )
            .with_rect(8.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BACKGROUND)
            .without_pointer_events()
            .entity();

        tree.in_parent(panel, |tree| {
            handles.readout = tree
                .add_node()
                .window(
                    Ab(vec2(14.0, 8.0)),
                    Ab(vec2(PANEL_WIDTH - 28.0, 26.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 17.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_COLOR)
                .without_pointer_events()
                .entity();
        });

        handles.prompt = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -30.0)),
                Ab(vec2(BAND_WIDTH, 26.0)),
                Anchor::BottomCenter,
            )
            .with_text("", 16.0)
            .text_center()
            .color_raw::<UiBase>(PROMPT_COLOR)
            .without_pointer_events()
            .entity();

        handles.speaker = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -122.0)),
                Ab(vec2(BAND_WIDTH, 20.0)),
                Anchor::BottomCenter,
            )
            .with_text("", SPEAKER_SIZE)
            .text_center()
            .color_raw::<UiBase>(CLEAR_COLOR)
            .without_pointer_events()
            .entity();

        handles.line = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -68.0)),
                Ab(vec2(BAND_WIDTH, 54.0)),
                Anchor::BottomCenter,
            )
            .with_text("", LINE_SIZE)
            .text_center()
            .with_text_outline(Vec4::new(0.0, 0.0, 0.0, 0.9), 1.4)
            .color_raw::<UiBase>(CLEAR_COLOR)
            .without_pointer_events()
            .entity();

        handles.crosshair = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(5.0, 5.0)), Anchor::Center)
            .with_rect(3.0, 0.0, CROSSHAIR_COLOR)
            .color_raw::<UiBase>(CROSSHAIR_COLOR)
            .without_pointer_events()
            .entity();

        handles.fade = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, CLEAR_COLOR)
            .color_raw::<UiBase>(CLEAR_COLOR)
            .without_pointer_events()
            .entity();
    });

    handles.root = root;
    handles
}

pub fn show_hud(
    resources: Res<FourfoldResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    reveal(&resources, &mut dirty, &mut commands, true);
}

pub fn hide_hud(
    resources: Res<FourfoldResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    reveal(&resources, &mut dirty, &mut commands, false);
}

fn reveal(
    resources: &FourfoldResources,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    visible: bool,
) {
    if resources.hud.root == Entity::default() {
        return;
    }
    ui_set_visible(dirty, commands, resources.hud.root, visible);
}

pub fn show(
    mut resources: ResMut<FourfoldResources>,
    dimension: Res<State<Dimension>>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    let handles = resources.hud;
    if handles.readout == Entity::default() {
        return;
    }

    let here = dimension.current;
    let note = here.name().to_string();
    if note != resources.last_note {
        resources.last_note = note.clone();
        ui_set_text(&mut commands, handles.readout, &note);
    }

    let prompt = prompt_for(&resources, here);
    if prompt != resources.last_prompt {
        resources.last_prompt = prompt.clone();
        ui_set_text(&mut commands, handles.prompt, &prompt);
    }

    paint_line(&mut resources, handles, &mut commands);
    paint_fade(&resources, handles, &mut commands);

    let wanted = here == Dimension::Hub;
    if wanted != resources.crosshair_shown {
        resources.crosshair_shown = wanted;
        ui_set_visible(&mut dirty, &mut commands, handles.crosshair, wanted);
    }
}

fn paint_line(resources: &mut FourfoldResources, handles: HudHandles, commands: &mut Commands) {
    let Some(index) = resources.story.showing else {
        if resources.story.painted.is_some() {
            resources.story.painted = None;
            ui_set_color::<UiBase>(commands, handles.line, CLEAR_COLOR);
            ui_set_color::<UiBase>(commands, handles.speaker, CLEAR_COLOR);
        }
        return;
    };

    let line = &data::SCRIPT[index];
    if resources.story.painted != Some(index) {
        resources.story.painted = Some(index);
        ui_set_text(commands, handles.line, line.text);
        ui_set_text(commands, handles.speaker, line.speaker);
    }

    let held = resources.story.held;
    let left = line.hold - held;
    let alpha = (held / FADE_IN).min(1.0).min((left / FADE_OUT).max(0.0));
    let tint = data::speaker_color(line.speaker);
    ui_set_color::<UiBase>(commands, handles.line, Vec4::new(0.94, 0.95, 0.98, alpha));
    ui_set_color::<UiBase>(
        commands,
        handles.speaker,
        Vec4::new(tint[0], tint[1], tint[2], alpha * 0.85),
    );
}

fn paint_fade(resources: &FourfoldResources, handles: HudHandles, commands: &mut Commands) {
    let color = data::FADE_COLOR;
    ui_set_color::<UiBase>(
        commands,
        handles.fade,
        Vec4::new(color[0], color[1], color[2], resources.fade),
    );
}

fn prompt_for(resources: &FourfoldResources, here: Dimension) -> String {
    match here {
        Dimension::Hub => {
            if resources.arch.open {
                "the floor stands open".to_string()
            } else if resources.walker.near_dials {
                "turn the dials".to_string()
            } else {
                String::new()
            }
        }
        Dimension::Glass => {
            if resources.runner.armed {
                "the door back is at the low end".to_string()
            } else {
                String::new()
            }
        }
        Dimension::Loom => "push every block onto a mark, then stand on the pale tile".to_string(),
        Dimension::Cairn => "climb to the cairn on the high ground".to_string(),
        Dimension::Hearth => "the well is past the houses".to_string(),
    }
}
