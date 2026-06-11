use crate::ecs::{PauseHandles, SandboxWorld, Screen};
use crate::systems::lifecycle;
use crate::systems::screens::menu_button;
use crate::systems::world::{enemies, grass, infinite, player, terrain};
use crate::theme::*;
use nightshade::ecs::ui::components::AccessibleRole;
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
    let mut infinite_world_checkbox = Entity::default();
    let mut grass_checkbox = Entity::default();
    let mut terrain_checkbox = Entity::default();
    let mut fly_checkbox = Entity::default();

    tree.in_parent(root, |tree| {
        tree.add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(BACKDROP)
            .entity();

        let panel = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(380.0, 512.0)), Anchor::Center)
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
                    Ab(vec2(MENU_BUTTON_SIZE.x, 376.0)),
                    Anchor::TopCenter,
                )
                .flow(FlowDirection::Vertical, 0.0, 12.0)
                .entity();
            tree.in_parent(column, |tree| {
                resume_button = menu_button::build(tree, "RESUME");
                main_menu_button = menu_button::build(tree, "MAIN MENU");
                quit_button = menu_button::build(tree, "QUIT TO DESKTOP");
                infinite_world_checkbox = build_checkbox(tree, "INFINITE WORLD", true);
                grass_checkbox = build_checkbox(tree, "ENABLE GRASS", true);
                terrain_checkbox = build_checkbox(tree, "TERRAIN", true);
                fly_checkbox = build_checkbox(tree, "FLY MODE", false);
            });
        });
    });

    PauseHandles {
        root,
        resume_button,
        main_menu_button,
        quit_button,
        infinite_world_checkbox,
        grass_checkbox,
        terrain_checkbox,
        fly_checkbox,
    }
}

/// A checkbox row styled like the sandbox menu buttons: the whole row
/// is the click target, with a small bordered box on the left whose
/// accent fill toggles with the value. Wiring [`UiCheckboxData`] onto
/// the row lets the engine's checkbox handling drive the toggle.
fn build_checkbox(tree: &mut UiTreeBuilder, label: &str, initial_value: bool) -> Entity {
    let row = tree
        .add_node()
        .flow_child(Ab(vec2(MENU_BUTTON_SIZE.x, MENU_CHECKBOX_HEIGHT)))
        .with_rect(2.0, 1.0, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BG)
        .color_raw::<UiHover>(PANEL_HOVER)
        .color_raw::<UiPressed>(PANEL_PRESSED)
        .color_raw::<UiFocused>(ACCENT_DIM)
        .with_transition::<UiHover>(14.0, 8.0)
        .with_transition::<UiPressed>(20.0, 12.0)
        .with_transition::<UiFocused>(14.0, 8.0)
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .entity();
    let mut inner = Entity::default();
    tree.in_parent(row, |tree| {
        let box_entity = tree
            .add_node()
            .window(
                Ab(vec2(20.0, 0.0)) + Rl(vec2(0.0, 50.0)),
                Ab(vec2(18.0, 18.0)),
                Anchor::CenterLeft,
            )
            .with_rect(2.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(TRANSPARENT)
            .entity();
        inner = tree.in_parent(box_entity, |tree| {
            tree.add_node()
                .boundary(
                    Ab(vec2(3.0, 3.0)),
                    Ab(vec2(-3.0, -3.0)) + Rl(vec2(100.0, 100.0)),
                )
                .with_rect(1.0, 0.0, TRANSPARENT)
                .color_raw::<UiBase>(ACCENT)
                .with_visible(initial_value)
                .entity()
        });
        tree.add_node()
            .window(
                Ab(vec2(50.0, 0.0)) + Rl(vec2(0.0, 50.0)),
                Ab(vec2(MENU_BUTTON_SIZE.x - 70.0, MENU_CHECKBOX_HEIGHT)),
                Anchor::CenterLeft,
            )
            .with_text(label, 18.0)
            .text_left()
            .color_raw::<UiBase>(TEXT_COLOR)
            .color_raw::<UiFocused>(WHITE)
            .with_transition::<UiFocused>(14.0, 8.0)
            .entity();
    });
    tree.world_mut().ui.set_ui_checkbox(
        row,
        UiCheckboxData {
            value: initial_value,
            changed: false,
            inner_entity: inner,
        },
    );
    if let Some(interaction) = tree.world_mut().ui.get_ui_node_interaction_mut(row) {
        interaction.accessible_role = Some(AccessibleRole::Checkbox);
    }
    row
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
    let checkbox = sandbox_world
        .resources
        .ui_handles
        .pause
        .infinite_world_checkbox;
    if let Some(value) = ui_checkbox_changed(world, checkbox) {
        infinite::set_enabled(sandbox_world, world, value);
        grass::sync_domain(sandbox_world, world);
    }
    let grass_box = sandbox_world.resources.ui_handles.pause.grass_checkbox;
    if let Some(value) = ui_checkbox_changed(world, grass_box) {
        grass::set_enabled(sandbox_world, world, value);
    }
    let terrain_box = sandbox_world.resources.ui_handles.pause.terrain_checkbox;
    if let Some(value) = ui_checkbox_changed(world, terrain_box) {
        terrain::set_enabled(sandbox_world, world, value);
    }
    let fly_box = sandbox_world.resources.ui_handles.pause.fly_checkbox;
    if let Some(value) = ui_checkbox_changed(world, fly_box) {
        player::set_fly(sandbox_world, world, value);
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
