use crate::resources::{HudHandles, SandboxResources};
use crate::theme::*;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::*;

pub fn build(tree: &mut UiTreeBuilder) -> HudHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_visible(false)
        .entity();

    let mut fps_label = Entity::default();
    let mut crosshair = Entity::default();

    tree.in_parent(root, |tree| {
        fps_label = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-24.0, 16.0)),
                Ab(vec2(320.0, 64.0)),
                Anchor::TopRight,
            )
            .with_text("0 FPS", 52.0)
            .text_right()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 2.0)
            .color_raw::<UiBase>(WHITE)
            .entity();

        crosshair = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(6.0, 6.0)), Anchor::Center)
            .with_rect(3.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(CROSSHAIR)
            .entity();
    });

    HudHandles {
        root,
        fps_label,
        crosshair,
    }
}

pub fn update(
    sandbox_world: Res<SandboxResources>,
    time: Res<Time>,
    mut dirty: ResMut<nightshade_ui::resources::RetainedUiDirty>,
    mut commands: Commands,
    mut colors: Query<(&mut nightshade_ui::components::UiNodeColor,)>,
) {
    let fps_label = sandbox_world.ui_handles.hud.fps_label;
    let show_fps = sandbox_world.screen.show_fps;
    let fps = time.frames_per_second;

    ui_set_visible(&mut dirty, &mut commands, fps_label, show_fps);
    if !show_fps {
        return;
    }

    ui_set_text(&mut commands, fps_label, &format!("{fps:.0} FPS"));
    let color = if fps >= 110.0 {
        vec4(0.35, 1.0, 0.45, 1.0)
    } else if fps >= 55.0 {
        vec4(1.0, 0.85, 0.25, 1.0)
    } else {
        vec4(1.0, 0.3, 0.25, 1.0)
    };
    colors.for_entity(fps_label, |(node_color,)| {
        node_color.colors[nightshade_ui::data::state::UiBase::INDEX] = Some(color);
        node_color.computed_color = color;
    });
}
