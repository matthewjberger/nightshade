use crate::data::Tool;
use crate::resources::{HudHandles, SdfScene};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_sdf::contract::render_data::{SdfDebugView, SdfMarchScale};
use nightshade_ui::prelude::*;

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
                Ab(vec2(theme::MARGIN, theme::MARGIN)),
                Ab(vec2(theme::PANEL_WIDTH, theme::PANEL_HEIGHT)),
                Anchor::TopLeft,
            )
            .with_rect(8.0, 1.0, theme::PANEL_BORDER)
            .color_raw::<UiBase>(theme::PANEL_BACKGROUND)
            .entity();

        tree.in_parent(panel, |tree| {
            handles.rate_readout = tree
                .add_node()
                .window(
                    Ab(vec2(12.0, 8.0)),
                    Ab(vec2(theme::PANEL_WIDTH - 24.0, 28.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 24.0)
                .text_left()
                .color_raw::<UiBase>(theme::RATE_BANDS[0].1)
                .entity();

            handles.view_readout = tree
                .add_node()
                .window(
                    Ab(vec2(12.0, 38.0)),
                    Ab(vec2(theme::PANEL_WIDTH - 24.0, 24.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 16.0)
                .text_left()
                .color_raw::<UiBase>(theme::TEXT_DIM)
                .entity();
        });

        let slots = Tool::ALL.len() as f32;
        let bar_width = slots * theme::SLOT_WIDTH + (slots + 1.0) * theme::SLOT_GAP;
        let bar = tree
            .add_node()
            .window(
                Rl(vec2(50.0, theme::BAR_BOTTOM)),
                Ab(vec2(bar_width, theme::SLOT_HEIGHT + 2.0 * theme::SLOT_GAP)),
                Anchor::BottomCenter,
            )
            .with_rect(10.0, 1.0, theme::PANEL_BORDER)
            .color_raw::<UiBase>(theme::PANEL_BACKGROUND)
            .entity();
        tree.in_parent(bar, |tree| {
            for (index, tool) in Tool::ALL.iter().enumerate() {
                let left = theme::SLOT_GAP + index as f32 * (theme::SLOT_WIDTH + theme::SLOT_GAP);
                let slot = tree
                    .add_node()
                    .window(
                        Ab(vec2(left, theme::SLOT_GAP)),
                        Ab(vec2(theme::SLOT_WIDTH, theme::SLOT_HEIGHT)),
                        Anchor::TopLeft,
                    )
                    .with_rect(8.0, 1.0, theme::PANEL_BORDER)
                    .color_raw::<UiBase>(theme::SLOT_IDLE)
                    .entity();
                let label = tree.in_parent(slot, |tree| {
                    tree.add_node()
                        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                        .with_text(tool.name(), 18.0)
                        .color_raw::<UiBase>(theme::TEXT_DIM)
                        .entity()
                });
                handles.tool_slots[index] = slot;
                handles.tool_labels[index] = label;
            }
        });
    });

    handles.root = root;
    handles
}

pub fn show_rate(
    mut scene: ResMut<SdfScene>,
    state: Res<nightshade_render::data::config::RendererState>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let readout = scene.hud.rate_readout;
    if readout == Entity::default() {
        return;
    }
    let rate = time.frames_per_second.round().max(0.0) as u32;
    let gpu_tenths = (state.gpu_frame_time_ms * 10.0).round().max(0.0) as u32;
    if rate == scene.last_rate && gpu_tenths == scene.last_gpu_tenths {
        return;
    }
    scene.last_rate = rate;
    scene.last_gpu_tenths = gpu_tenths;

    let (_, color) = theme::RATE_BANDS
        .iter()
        .find(|(floor, _)| rate as f32 >= *floor)
        .copied()
        .unwrap_or(theme::RATE_BANDS[theme::RATE_BANDS.len() - 1]);
    let gpu = gpu_tenths as f32 / 10.0;
    ui_set_text(&mut commands, readout, &format!("{rate} fps  {gpu:.1} ms"));
    ui_set_color::<UiBase>(&mut commands, readout, color);
}

pub fn show_view(
    mut scene: ResMut<SdfScene>,
    view: Res<SdfDebugView>,
    scale: Res<SdfMarchScale>,
    mut commands: Commands,
) {
    let readout = scene.hud.view_readout;
    if readout == Entity::default() {
        return;
    }
    if scene.last_view == Some(*view) && scene.last_scale == scale.0 {
        return;
    }
    scene.last_view = Some(*view);
    scene.last_scale = scale.0;
    if scale.0 < 1.0 {
        let percent = (scale.0 * 100.0).round() as u32;
        ui_set_text(
            &mut commands,
            readout,
            &format!("{} at {percent}%", view.name()),
        );
    } else {
        ui_set_text(&mut commands, readout, view.name());
    }
}

pub fn show_tool(mut scene: ResMut<SdfScene>, mut commands: Commands) {
    if scene.hud.root == Entity::default() || scene.last_tool == Some(scene.tool) {
        return;
    }
    scene.last_tool = Some(scene.tool);
    for (index, tool) in Tool::ALL.iter().enumerate() {
        let equipped = *tool == scene.tool;
        let (fill, text) = if equipped {
            (theme::SLOT_EQUIPPED, theme::TEXT_BRIGHT)
        } else {
            (theme::SLOT_IDLE, theme::TEXT_DIM)
        };
        ui_set_color::<UiBase>(&mut commands, scene.hud.tool_slots[index], fill);
        ui_set_color::<UiBase>(&mut commands, scene.hud.tool_labels[index], text);
    }
}
