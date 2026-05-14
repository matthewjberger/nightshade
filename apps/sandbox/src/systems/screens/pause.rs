use crate::ecs::{PauseHandles, SandboxWorld, Screen};
use crate::systems::lifecycle;
use crate::systems::screens::menu_button;
use crate::systems::world::{enemies, player};
use crate::theme::*;
use nightshade::prelude::*;

pub fn build(tree: &mut UiTreeBuilder) -> PauseHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_intro(UiAnimationType::Fade, 0.18)
        .with_visible(false)
        .entity();

    let mut resume_button = Entity::default();
    let mut main_menu_button = Entity::default();
    let mut quit_button = Entity::default();

    tree.in_parent(root, |tree| {
        tree.add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(BACKDROP)
            .entity();

        let panel = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(380.0, 320.0)), Anchor::Center)
            .with_rect(8.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG_DEEP)
            .with_shadow(vec4(0.0, 0.0, 0.0, 0.7), vec2(0.0, 12.0), 36.0, 4.0)
            .entity();
        tree.in_parent(panel, |tree| {
            tree.add_node()
                .window(
                    Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 32.0)),
                    Ab(vec2(320.0, 36.0)),
                    Anchor::TopCenter,
                )
                .with_text("PAUSED", 28.0)
                .text_center()
                .with_text_outline(ACCENT, 1.5)
                .color_raw::<UiBase>(WHITE)
                .entity();

            tree.add_node()
                .window(
                    Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 76.0)),
                    Ab(vec2(80.0, 2.0)),
                    Anchor::TopCenter,
                )
                .with_rect(0.0, 0.0, TRANSPARENT)
                .color_raw::<UiBase>(ACCENT)
                .entity();

            let column = tree
                .add_node()
                .window(
                    Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 108.0)),
                    Ab(vec2(MENU_BUTTON_SIZE.x, 184.0)),
                    Anchor::TopCenter,
                )
                .flow(FlowDirection::Vertical, 0.0, 12.0)
                .entity();
            tree.in_parent(column, |tree| {
                resume_button = menu_button::build(tree, "RESUME");
                main_menu_button = menu_button::build(tree, "MAIN MENU");
                quit_button = menu_button::build(tree, "QUIT TO DESKTOP");
            });
        });
    });

    PauseHandles {
        root,
        resume_button,
        main_menu_button,
        quit_button,
    }
}

pub fn handle_input(sandbox_world: &mut SandboxWorld, world: &mut World) {
    if !matches!(sandbox_world.resources.screen.current, Screen::Paused) {
        return;
    }
    let resume = sandbox_world.resources.ui_handles.pause.resume_button;
    let main_menu = sandbox_world.resources.ui_handles.pause.main_menu_button;
    let quit = sandbox_world.resources.ui_handles.pause.quit_button;
    let mut clicked_resume = false;
    let mut clicked_main_menu = false;
    let mut clicked_quit = false;
    for entity in ui_button_clicks(world) {
        if entity == resume {
            clicked_resume = true;
        } else if entity == main_menu {
            clicked_main_menu = true;
        } else if entity == quit {
            clicked_quit = true;
        }
    }
    if clicked_resume {
        lifecycle::enter(sandbox_world, world, Screen::InGame);
    } else if clicked_main_menu {
        player::reset(sandbox_world, world);
        enemies::despawn(sandbox_world, world);
        lifecycle::enter(sandbox_world, world, Screen::Title);
    } else if clicked_quit {
        world.resources.window.should_exit = true;
    }
}
