use crate::ecs::{ENEMY, SandboxResources, Screen, TitleHandles};
use crate::systems::screens::menu_button;
use crate::systems::world::enemies;
use crate::theme::*;
use nightshade::prelude::*;

const TITLE_TEXT: &str = "SANDBOX";
const SUBTITLE_TEXT: &str = "NIGHTSHADE EXAMPLE";

pub fn build(tree: &mut UiTreeBuilder) -> TitleHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_intro(UiAnimationType::Fade, 0.4)
        .entity();

    let mut play_button = Entity::default();
    let mut quit_button = Entity::default();

    tree.in_parent(root, |tree| {
        tree.add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(VIGNETTE)
            .entity();

        tree.add_node()
            .window(
                Ab(vec2(0.0, 80.0)) + Rl(vec2(50.0, 0.0)),
                Ab(vec2(800.0, 80.0)),
                Anchor::TopCenter,
            )
            .with_text(TITLE_TEXT, 64.0)
            .text_center()
            .with_text_outline(ACCENT, 2.0)
            .color_raw::<UiBase>(WHITE)
            .entity();

        tree.add_node()
            .window(
                Ab(vec2(0.0, 168.0)) + Rl(vec2(50.0, 0.0)),
                Ab(vec2(600.0, 24.0)),
                Anchor::TopCenter,
            )
            .with_text(SUBTITLE_TEXT, 16.0)
            .text_center()
            .color_raw::<UiBase>(TEXT_DIM)
            .entity();

        tree.add_node()
            .window(
                Ab(vec2(0.0, 200.0)) + Rl(vec2(50.0, 0.0)),
                Ab(vec2(120.0, 2.0)),
                Anchor::TopCenter,
            )
            .with_rect(0.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(ACCENT)
            .entity();

        let menu_column = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -96.0)),
                Ab(vec2(MENU_BUTTON_SIZE.x, 112.0)),
                Anchor::BottomCenter,
            )
            .flow(FlowDirection::Vertical, 8.0, 8.0)
            .entity();
        tree.in_parent(menu_column, |tree| {
            play_button = menu_button::build(tree, "PLAY");
            quit_button = menu_button::build(tree, "QUIT");
        });

        tree.add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -32.0)),
                Ab(vec2(700.0, 18.0)),
                Anchor::Center,
            )
            .with_text(
                "WASD MOVE  /  SPACE JUMP  /  MOUSE LOOK  /  ESC PAUSE  /  R RESET",
                12.0,
            )
            .text_center()
            .color_raw::<UiBase>(TEXT_FAINT)
            .entity();
    });

    TitleHandles {
        root,
        play_button,
        quit_button,
    }
}

pub fn handle_input(sandbox_world: &mut SandboxResources, world: &mut World) {
    if !in_state(world, Screen::Title) {
        return;
    }
    let play = sandbox_world.ui_handles.title.play_button;
    let quit = sandbox_world.ui_handles.title.quit_button;
    let mut clicked_play = false;
    let mut clicked_quit = false;
    for entity in ui_button_clicks(world) {
        if entity == play {
            clicked_play = true;
        } else if entity == quit {
            clicked_quit = true;
        }
    }
    if clicked_play {
        if world.ecs.worlds[GAME]
            .query_entities(ENEMY)
            .next()
            .is_none()
        {
            enemies::spawn(world);
        }
        next_state(world, Screen::InGame);
    }
    if clicked_quit {
        world.resources.window.should_exit = true;
    }
}
