use crate::components::{CardSlot, LineSlot, ReplySlot, SummarySlot};
use crate::data::{CARD_SLOTS, LINE_SLOTS, REPLY_SLOTS, SUMMARY_SLOTS};
use crate::queries::glass_label;
use crate::resources::{Arguments, NightwireResources, UiHandles};
use crate::theme::*;
use nightshade::prelude::*;
use nightshade_camera::prelude::{ActiveCamera, spawn_ortho_camera};
use nightshade_graphics::prelude::RenderSettings;
use nightshade_ui::components::layout_types::{FlowAlignment, FlowDirection};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiRuntime;

pub fn initialize(
    mut settings: ResMut<RenderSettings>,
    mut runtime: ResMut<RetainedUiRuntime>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
) {
    settings.render_world_to_swapchain = false;
    settings.clear_color = CLEAR_COLOR;
    runtime.background_color = None;
    active_camera.0 = Some(spawn_ortho_camera(&mut commands, vec2(0.0, 0.0)));
}

pub fn build_screens(
    mut resources: ResMut<NightwireResources>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut handles = UiHandles::default();
    tree.in_parent(root, |tree| {
        build_shift(tree, &mut handles);
        build_title(tree, &mut handles);
        build_over(tree, &mut handles);
    });
    tree.finish();

    resources.ui = handles;
    resources.rows_dirty = true;
    resources.controls_dirty = true;
}

pub fn apply_arguments(
    arguments: Res<Arguments>,
    mut resources: ResMut<NightwireResources>,
    mut commands: Commands,
) {
    if arguments.no_glass {
        resources.glass_on = false;
        resources.glass_amount = 0.0;
        let button = resources.ui.glass_button;
        ui_set_text(&mut commands, button, glass_label(false));
    }
    if arguments.shift {
        crate::commands::begin_shift(&mut resources);
    }
}

fn build_shift(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let shift = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_visible(false)
        .entity();
    handles.shift = shift;

    tree.in_parent(shift, |tree| {
        let bar = tree
            .add_node()
            .window(
                Ab(vec2(MARGIN, MARGIN)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-MARGIN * 2.0, BAR_HEIGHT)),
                Anchor::TopLeft,
            )
            .with_rect(6.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BACKGROUND)
            .entity();

        tree.in_parent(bar, |tree| {
            handles.clock = tree
                .add_node()
                .window(Ab(vec2(14.0, 10.0)), Ab(vec2(420.0, 22.0)), Anchor::TopLeft)
                .with_text("", LABEL_SIZE)
                .text_left()
                .color_raw::<UiBase>(TEXT_DIM)
                .without_pointer_events()
                .entity();

            handles.caller = tree
                .add_node()
                .window(
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(-14.0, 10.0)),
                    Ab(vec2(460.0, 22.0)),
                    Anchor::TopRight,
                )
                .with_text("", LABEL_SIZE)
                .text_right()
                .color_raw::<UiBase>(TEXT_WARM)
                .without_pointer_events()
                .entity();
        });

        let transcript = tree
            .add_node()
            .window(
                Ab(vec2(MARGIN, TRANSCRIPT_TOP)),
                Rl(vec2(TRANSCRIPT_WIDTH, 100.0))
                    + Ab(vec2(
                        -MARGIN,
                        -(TRANSCRIPT_TOP + REPLY_HEIGHT + MARGIN + 12.0),
                    )),
                Anchor::TopLeft,
            )
            .with_rect(6.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BACKGROUND)
            .flow_with_alignment(
                FlowDirection::Vertical,
                14.0,
                2.0,
                FlowAlignment::End,
                FlowAlignment::Start,
            )
            .with_clip()
            .entity();

        tree.in_parent(transcript, |tree| {
            for index in 0..LINE_SLOTS {
                let row = tree
                    .add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, ROW_HEIGHT)))
                    .with_text("", LINE_SIZE)
                    .text_left()
                    .color_raw::<UiBase>(TEXT_BODY)
                    .without_pointer_events()
                    .entity();
                tree.commands().set(row, LineSlot { index });
            }
        });

        let replies = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(MARGIN, -MARGIN)),
                Rl(vec2(TRANSCRIPT_WIDTH, 0.0)) + Ab(vec2(-MARGIN, REPLY_HEIGHT)),
                Anchor::BottomLeft,
            )
            .with_rect(6.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BACKGROUND)
            .flow(FlowDirection::Vertical, 14.0, 6.0)
            .entity();

        tree.in_parent(replies, |tree| {
            for index in 0..REPLY_SLOTS {
                let button = tree.add_button("");
                tree.commands().set(button, ReplySlot { index });
            }
            handles.send_button = tree.add_button("Ring the deputy");
            handles.hold_button = tree.add_button("Keep him at the station");
            handles.advance_button = tree.add_button("Take the next call");
        });

        let notebook = tree
            .add_node()
            .window(
                Rl(vec2(NOTEBOOK_LEFT, 0.0)) + Ab(vec2(0.0, TRANSCRIPT_TOP)),
                Rl(vec2(100.0 - NOTEBOOK_LEFT, 100.0))
                    + Ab(vec2(-MARGIN, -(TRANSCRIPT_TOP + MARGIN))),
                Anchor::TopLeft,
            )
            .with_rect(6.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BACKGROUND)
            .flow(FlowDirection::Vertical, 14.0, 6.0)
            .entity();

        tree.in_parent(notebook, |tree| {
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 26.0)))
                .with_text("NOTEBOOK", LABEL_SIZE)
                .text_left()
                .color_raw::<UiBase>(TEXT_WARM)
                .without_pointer_events();

            handles.notebook_note = tree
                .add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 44.0)))
                .with_text("", LABEL_SIZE)
                .text_left()
                .color_raw::<UiBase>(TEXT_DIM)
                .without_pointer_events()
                .entity();

            for index in 0..CARD_SLOTS {
                let button = tree.add_button("");
                tree.commands().set(button, CardSlot { index });
            }

            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 30.0)))
                .flex_grow(1.0);

            handles.glass_button = tree.add_button(glass_label(true));
        });
    });
}

fn build_title(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(PANEL_WIDTH, TITLE_HEIGHT)),
            Anchor::Center,
        )
        .with_rect(8.0, 1.0, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BACKGROUND)
        .flow(FlowDirection::Vertical, 24.0, 10.0)
        .with_visible(false)
        .entity();
    handles.title = panel;

    tree.in_parent(panel, |tree| {
        text_row(tree, "NIGHTWIRE", TITLE_SIZE, TEXT_WARM, 52.0);
        text_row(
            tree,
            "Ashgrove Valley, 1977. You are Hale, on the night board.",
            LABEL_SIZE,
            TEXT_BODY,
            26.0,
        );
        text_row(
            tree,
            "Three calls stand between you and the morning.",
            LABEL_SIZE,
            TEXT_BODY,
            26.0,
        );
        text_row(
            tree,
            "Write down every name. A name can be raised later.",
            LABEL_SIZE,
            TEXT_DIM,
            30.0,
        );
        handles.begin_button = tree.add_button("Take the board");
        handles.quit_button = tree.add_button("Go home");
    });
}

fn build_over(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(PANEL_WIDTH, OVER_HEIGHT)),
            Anchor::Center,
        )
        .with_rect(8.0, 1.0, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BACKGROUND)
        .flow(FlowDirection::Vertical, 24.0, 8.0)
        .with_visible(false)
        .entity();
    handles.over = panel;

    tree.in_parent(panel, |tree| {
        text_row(tree, "SHIFT REPORT", HEADING_SIZE, TEXT_WARM, 40.0);
        for index in 0..SUMMARY_SLOTS {
            let row = text_row(tree, "", LABEL_SIZE, TEXT_BODY, 24.0);
            tree.commands().set(row, SummarySlot { index });
        }
        handles.again_button = tree.add_button("Work the night again");
        handles.over_title_button = tree.add_button("Back to the title");
    });
}

fn text_row(tree: &mut UiTreeBuilder, text: &str, size: f32, color: Vec4, height: f32) -> Entity {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, height)))
        .with_text(text, size)
        .text_left()
        .color_raw::<UiBase>(color)
        .without_pointer_events()
        .entity()
}
