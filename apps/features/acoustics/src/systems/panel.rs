use crate::resources::{AcousticsDemo, UiHandles};
use nightshade::prelude::*;
use nightshade_acoustics::contract::{AcousticsHeard, AcousticsRoom, AcousticsSettings};
use nightshade_acoustics::queries::acoustics_path_of;
use nightshade_ui::prelude::*;

const PANEL_WIDTH: f32 = 560.0;
const PANEL_HEIGHT: f32 = 168.0;
const ROW_HEIGHT: f32 = 30.0;
const TEXT_SIZE: f32 = 21.0;
const MARGIN: f32 = 16.0;
const PADDING: f32 = 12.0;
const BACKGROUND: Vec4 = Vec4::new(0.05, 0.05, 0.08, 0.78);
const BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.12);
const TEXT: Vec4 = Vec4::new(0.82, 0.85, 0.92, 1.0);

pub fn build(build: &mut nightshade_ui::commands::builder::UiBuild) -> UiHandles {
    let mut handles = UiHandles::default();
    let mut tree = UiTreeBuilder::new(build.context());

    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    tree.in_parent(root, |tree| {
        let panel = tree
            .add_node()
            .window(
                Ab(vec2(MARGIN, MARGIN)),
                Ab(vec2(PANEL_WIDTH, PANEL_HEIGHT)),
                Anchor::TopLeft,
            )
            .with_rect(8.0, 1.0, BORDER)
            .color_raw::<UiBase>(BACKGROUND)
            .entity();

        tree.in_parent(panel, |tree| {
            handles.door_row = row(tree, 0);
            handles.path_row = row(tree, 1);
            handles.room_row = row(tree, 2);
            handles.opening_row = row(tree, 3);
        });

        handles.panel = panel;
    });

    tree.finish();
    handles
}

fn row(tree: &mut UiTreeBuilder, index: u32) -> Entity {
    tree.add_node()
        .window(
            Ab(vec2(PADDING, PADDING + ROW_HEIGHT * index as f32)),
            Ab(vec2(PANEL_WIDTH - PADDING * 2.0, ROW_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_text("", TEXT_SIZE)
        .text_left()
        .color_raw::<UiBase>(TEXT)
        .entity()
}

pub fn refresh(
    demo: Res<AcousticsDemo>,
    heard: Res<AcousticsHeard>,
    room: Res<AcousticsRoom>,
    settings: Res<AcousticsSettings>,
    mut commands: Commands,
) {
    if !demo.built {
        return;
    }
    let handles = demo.handles;

    let door = if demo.shut { "shut" } else { "open" };
    let dots = if demo.showing { "on" } else { "off" };
    let flow = if demo.flowing { "on" } else { "off" };
    let view = if demo.chasing { "third" } else { "first" };
    ui_set_text(
        &mut commands,
        handles.door_row,
        &format!("E door {door}   V echoes {dots}   R rays {flow}   C view {view}"),
    );

    match acoustics_path_of(&heard, demo.source) {
        Some(path) => ui_set_text(
            &mut commands,
            handles.path_row,
            &format!(
                "low {:.2}   high {:.2}   cut {:.0} Hz   {:.1} m",
                path.gain_low,
                path.gain_high,
                path.cutoff(settings.low_band, settings.high_band),
                path.travel
            ),
        ),
        None => ui_set_text(&mut commands, handles.path_row, "waiting for the rays"),
    }

    ui_set_text(
        &mut commands,
        handles.room_row,
        &format!(
            "room  back {:.2}   decay {:.2} s   soak {:.2}   outside {:.2}",
            room.heard.loudness, room.heard.decay, room.heard.absorbed, room.heard.outdoors
        ),
    );

    ui_set_text(
        &mut commands,
        handles.opening_row,
        &format!(
            "opening  {:.2} {:.2} {:.2}",
            room.heard.opening.x, room.heard.opening.y, room.heard.opening.z
        ),
    );
}
