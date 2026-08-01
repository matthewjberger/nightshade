use crate::resources::{Aim, UiHandles, VoxelSettings};
use crate::theme;
use nalgebra_glm::{vec2, vec4};
use nightshade::prelude::*;
use nightshade_platform::prelude::Window;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

pub fn build(mut handles: ResMut<UiHandles>, mut build: UiBuild) {
    let mut tree = UiTreeBuilder::new(build.context());
    let mut built = UiHandles::default();

    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();
    tree.in_parent(root, |tree| {
        built.hud = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_visible(false)
            .entity();
        tree.in_parent(built.hud, |tree| {
            build_crosshair(tree);
            build_readouts(tree, &mut built);
        });
        build_title(tree, &mut built);
    });

    built.built = true;
    tree.finish();
    *handles = built;
}

fn build_crosshair(tree: &mut UiTreeBuilder<'_, '_>) {
    let reach = theme::CROSSHAIR_GAP + theme::CROSSHAIR_LENGTH / 2.0;
    let arms = [
        (
            vec2(0.0, -reach),
            vec2(theme::CROSSHAIR_THICKNESS, theme::CROSSHAIR_LENGTH),
        ),
        (
            vec2(0.0, reach),
            vec2(theme::CROSSHAIR_THICKNESS, theme::CROSSHAIR_LENGTH),
        ),
        (
            vec2(-reach, 0.0),
            vec2(theme::CROSSHAIR_LENGTH, theme::CROSSHAIR_THICKNESS),
        ),
        (
            vec2(reach, 0.0),
            vec2(theme::CROSSHAIR_LENGTH, theme::CROSSHAIR_THICKNESS),
        ),
    ];
    for (offset, size) in arms {
        tree.add_node()
            .window(Rl(vec2(50.0, 50.0)) + Ab(offset), Ab(size), Anchor::Center)
            .with_rect(0.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(theme::CROSSHAIR_COLOR);
    }
}

fn build_readouts(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let radius_card = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -theme::MARGIN)),
            Ab(vec2(theme::CARD_WIDTH, theme::CARD_HEIGHT)),
            Anchor::BottomCenter,
        )
        .flow(FlowDirection::Vertical, 5.0, 0.0)
        .entity();
    built.radius_label = tree.in_parent(radius_card, |tree| {
        tree.add_node()
            .size_px(theme::CARD_WIDTH - 10.0, theme::CARD_HEIGHT - 10.0)
            .with_text("", theme::BODY_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::ACCENT_COLOR)
            .entity()
    });

    let tally_card = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Ab(vec2(theme::MARGIN, theme::MARGIN)),
            Ab(vec2(theme::CARD_WIDTH, theme::CARD_HEIGHT)),
            Anchor::TopLeft,
        )
        .flow(FlowDirection::Vertical, 5.0, 0.0)
        .entity();
    built.tally_label = tree.in_parent(tally_card, |tree| {
        tree.add_node()
            .size_px(theme::CARD_WIDTH - 10.0, theme::CARD_HEIGHT - 10.0)
            .with_text("", theme::BODY_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::TEXT_COLOR)
            .entity()
    });
}

fn build_title(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    built.title_panel = tree
        .card_node(14.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(theme::TITLE_WIDTH, theme::TITLE_HEIGHT)),
            Anchor::Center,
        )
        .flow(FlowDirection::Vertical, 18.0, 10.0)
        .entity();
    tree.in_parent(built.title_panel, |tree| {
        tree.add_node()
            .size_px(theme::TITLE_WIDTH - 36.0, theme::TITLE_SIZE + 12.0)
            .with_text(theme::TITLE, theme::TITLE_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::ACCENT_COLOR);
        tree.add_node()
            .size_px(theme::TITLE_WIDTH - 36.0, theme::BODY_SIZE + 8.0)
            .with_text(
                "Five storeys of brick on concrete, one eighth of a meter to the block.",
                theme::BODY_SIZE,
            )
            .text_center()
            .color_raw::<UiBase>(theme::TEXT_COLOR);
        tree.add_node()
            .size_px(theme::TITLE_WIDTH - 36.0, theme::SMALL_SIZE + 6.0)
            .with_text("The building is yours to bring down.", theme::SMALL_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::DIM_TEXT_COLOR);
    });
}

pub fn refresh(
    window: Res<Window>,
    aim: Res<Aim>,
    settings: Res<VoxelSettings>,
    mut handles: ResMut<UiHandles>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if !handles.built {
        return;
    }
    let locked = window.cursor_locked;
    if handles.shown_locked != Some(locked) {
        handles.shown_locked = Some(locked);
        ui_set_visible(&mut dirty, &mut commands, handles.hud, locked);
        ui_set_visible(&mut dirty, &mut commands, handles.title_panel, !locked);
    }
    if handles.shown_radius != Some(aim.radius) {
        handles.shown_radius = Some(aim.radius);
        let meters = aim.radius as f32 * settings.block_size;
        let text = format!("Blast {} blocks across, {meters:.1} m", aim.radius * 2 + 1);
        ui_set_text(&mut commands, handles.radius_label, &text);
    }
    if handles.shown_tally != Some(aim.blasted) {
        handles.shown_tally = Some(aim.blasted);
        let text = if aim.blasted == 0 {
            "The building stands whole".to_string()
        } else {
            format!("{} blocks blasted", aim.blasted)
        };
        ui_set_text(&mut commands, handles.tally_label, &text);
    }
}
