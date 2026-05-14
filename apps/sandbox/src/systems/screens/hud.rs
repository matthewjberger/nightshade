use crate::ecs::{HudHandles, SandboxWorld};
use crate::theme::*;
use nightshade::prelude::*;

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
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-16.0, 16.0)),
                Ab(vec2(140.0, 18.0)),
                Anchor::TopRight,
            )
            .with_text("0 fps", 12.0)
            .text_right()
            .color_raw::<UiBase>(TEXT_FAINT)
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

pub fn update(sandbox_world: &SandboxWorld, world: &mut World) {
    let fps = world.resources.window.timing.frames_per_second;
    let fps_label = sandbox_world.resources.ui_handles.hud.fps_label;
    ui_set_text(world, fps_label, &format!("{:.0} fps", fps));
}
