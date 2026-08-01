use crate::resources::{DebugViewsResources, UiHandles};
use crate::theme;
use nightshade::prelude::*;
use nightshade_graphics::prelude::DebugDraw;
use nightshade_platform::prelude::*;
use nightshade_render::data::config::PbrDebugMode;
use nightshade_ui::commands::widgets::world_layout::UiLayoutNodes;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};

pub fn build(context: UiBuildContext) -> UiHandles {
    let mut tree = UiTreeBuilder::new(context);
    let panel = tree.add_floating_panel("debug_views", "Debug views", theme::PANEL_RECT);
    let content = tree.panel_content(panel).expect("panel content");
    let mut handles = UiHandles {
        panel,
        ..Default::default()
    };
    tree.in_parent(content, |tree| {
        tree.add_label("Tab steps to the next view. F1 hides this panel.");
        handles.mode_label = tree.add_label(&mode_text(PbrDebugMode::None));
        for mode in PbrDebugMode::ALL {
            let button = tree.add_button(mode.name());
            handles.buttons.push((*mode, button));
        }
    });
    tree.finish();
    handles
}

pub fn poll(
    resources: Res<DebugViewsResources>,
    frame: Res<RetainedUiFrame>,
    mut debug_draw: ResMut<DebugDraw>,
) {
    for event in ui_events(&frame) {
        let UiEvent::ButtonClicked(clicked) = event else {
            continue;
        };
        for (mode, button) in &resources.handles.buttons {
            if button == clicked {
                debug_draw.pbr_debug_mode = *mode;
            }
        }
    }
}

pub fn advance(input: Res<Input>, mut debug_draw: ResMut<DebugDraw>) {
    if !input.keyboard.just_pressed(KeyCode::Tab) {
        return;
    }
    let current = PbrDebugMode::ALL
        .iter()
        .position(|mode| *mode == debug_draw.pbr_debug_mode)
        .unwrap_or(0);
    debug_draw.pbr_debug_mode = PbrDebugMode::ALL[(current + 1) % PbrDebugMode::ALL.len()];
}

pub fn refresh_label(
    resources: Res<DebugViewsResources>,
    debug_draw: Res<DebugDraw>,
    mut commands: Commands,
) {
    ui_set_text(
        &mut commands,
        resources.handles.mode_label,
        &mode_text(debug_draw.pbr_debug_mode),
    );
}

pub fn drain_keyboard(
    resources: Res<DebugViewsResources>,
    reader: EventReader<AppEvent>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut nodes: UiLayoutNodes<'_, '_>,
) {
    for event in reader.iter() {
        if let AppEvent::Keyboard {
            key: KeyCode::F1,
            state: KeyState::Pressed,
        } = event
        {
            ui_toggle_visible(
                &mut nodes,
                &mut dirty,
                &mut commands,
                resources.handles.panel,
            );
        }
    }
}

fn mode_text(mode: PbrDebugMode) -> String {
    format!("View: {}", mode.name())
}
