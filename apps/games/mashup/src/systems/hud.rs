use crate::contract::WorldSide;
use crate::resources::{HudHandles, MashupResources, Phase};
use nightshade::prelude::*;
use nightshade_ui::prelude::*;

const PANEL_BACKGROUND: Vec4 = Vec4::new(0.04, 0.05, 0.07, 0.72);
const PANEL_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.10);
const TEXT_COLOR: Vec4 = Vec4::new(0.86, 0.88, 0.94, 1.0);
const CROSSHAIR_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.7);

const PANEL_WIDTH: f32 = 300.0;
const PANEL_HEIGHT: f32 = 44.0;
const MARGIN: f32 = 16.0;

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
            .entity();

        tree.in_parent(panel, |tree| {
            handles.readout = tree
                .add_node()
                .window(
                    Ab(vec2(12.0, 7.0)),
                    Ab(vec2(PANEL_WIDTH - 24.0, 30.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 18.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_COLOR)
                .entity();
        });

        handles.crosshair = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(6.0, 6.0)), Anchor::Center)
            .with_rect(3.0, 0.0, CROSSHAIR_COLOR)
            .color_raw::<UiBase>(CROSSHAIR_COLOR)
            .without_pointer_events()
            .with_visible(false)
            .entity();
    });

    handles
}

pub fn show(
    mut resources: ResMut<MashupResources>,
    mut dirty: ResMut<nightshade_ui::resources::RetainedUiDirty>,
    mut commands: Commands,
) {
    let readout = resources.hud.readout;
    if readout == Entity::default() {
        return;
    }

    let note = match (resources.here, resources.crossing.phase) {
        (_, Phase::Diving) => "stepping through".to_string(),
        (_, Phase::Settling) => "arriving".to_string(),
        (WorldSide::Meadow, _) => "the meadow".to_string(),
        (WorldSide::Quarry, _) => "the quarry".to_string(),
    };
    if note == resources.last_note {
        return;
    }
    resources.last_note = note.clone();
    ui_set_text(&mut commands, readout, &note);

    let crosshair = resources.hud.crosshair;
    if crosshair != Entity::default() {
        let wanted = resources.here == WorldSide::Quarry && resources.crossing.phase == Phase::Idle;
        ui_set_visible(&mut dirty, &mut commands, crosshair, wanted);
    }
}
