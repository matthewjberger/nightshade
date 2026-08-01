use crate::resources::{PauseHandles, Screen};
use crate::systems::screens::menu_button;
use crate::systems::world::{camera, character, enemies, infinite, player};
use crate::theme::*;
use nightshade::prelude::*;
use nightshade_platform::winit;
use nightshade_state::prelude::*;
use nightshade_ui::components::AccessibleRole;
use nightshade_ui::prelude::*;

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
    let mut third_person_checkbox = Entity::default();
    let mut spheres_checkbox = Entity::default();
    let mut stride_wheel_checkbox = Entity::default();
    let mut skeleton_checkbox = Entity::default();
    let mut fly_checkbox = Entity::default();

    let mut settings_button = Entity::default();
    let mut back_button = Entity::default();
    let mut show_fps_checkbox = Entity::default();
    let mut sensitivity_slider = Entity::default();
    let mut main_panel = Entity::default();
    let mut settings_panel = Entity::default();

    tree.in_parent(root, |tree| {
        tree.add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(BACKDROP)
            .entity();

        main_panel = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(380.0, 400.0)), Anchor::Center)
            .with_rect(8.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG_DEEP)
            .with_shadow(vec4(0.0, 0.0, 0.0, 0.7), vec2(0.0, 12.0), 36.0, 4.0)
            .entity();
        tree.in_parent(main_panel, |tree| {
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
                    Ab(vec2(MENU_BUTTON_SIZE.x, 244.0)),
                    Anchor::TopCenter,
                )
                .flow(FlowDirection::Vertical, 0.0, 12.0)
                .entity();
            tree.in_parent(column, |tree| {
                resume_button = menu_button::build(tree, "RESUME");
                settings_button = menu_button::build(tree, "SETTINGS");
                main_menu_button = menu_button::build(tree, "MAIN MENU");
                quit_button = menu_button::build(tree, "QUIT TO DESKTOP");
            });
        });

        settings_panel = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(380.0, 788.0)), Anchor::Center)
            .with_rect(8.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG_DEEP)
            .with_shadow(vec4(0.0, 0.0, 0.0, 0.7), vec2(0.0, 12.0), 36.0, 4.0)
            .with_visible(false)
            .entity();
        tree.in_parent(settings_panel, |tree| {
            tree.add_node()
                .window(
                    Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 32.0)),
                    Ab(vec2(320.0, 36.0)),
                    Anchor::TopCenter,
                )
                .with_text("SETTINGS", 28.0)
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
                    Ab(vec2(MENU_BUTTON_SIZE.x, 624.0)),
                    Anchor::TopCenter,
                )
                .flow(FlowDirection::Vertical, 0.0, 12.0)
                .entity();
            tree.in_parent(column, |tree| {
                infinite_world_checkbox = build_checkbox(tree, "INFINITE WORLD", false);
                third_person_checkbox = build_checkbox(tree, "THIRD PERSON", true);
                spheres_checkbox = build_checkbox(tree, "DEBUG SPHERES", false);
                stride_wheel_checkbox = build_checkbox(tree, "STRIDE WHEEL", true);
                skeleton_checkbox = build_checkbox(tree, "SKELETON", false);
                fly_checkbox = build_checkbox(tree, "FLY MODE", false);
                show_fps_checkbox = build_checkbox(tree, "SHOW FPS", true);
                sensitivity_slider = build_slider(tree, "LOOK SENSITIVITY", 1.0);
                back_button = menu_button::build(tree, "BACK");
            });
        });
    });

    PauseHandles {
        root,
        resume_button,
        main_menu_button,
        quit_button,
        infinite_world_checkbox,
        third_person_checkbox,
        spheres_checkbox,
        stride_wheel_checkbox,
        skeleton_checkbox,
        fly_checkbox,
        show_fps_checkbox,
        sensitivity_slider,
        main_panel,
        settings_panel,
        settings_button,
        back_button,
    }
}

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
    tree.commands().set(
        row,
        UiCheckboxData {
            value: initial_value,
            changed: false,
            inner_entity: inner,
        },
    );
    tree.edit::<nightshade_ui::components::UiNodeInteraction>(row, |interaction| {
        interaction.accessible_role = Some(AccessibleRole::Checkbox);
    });
    row
}

fn build_slider(tree: &mut UiTreeBuilder, label: &str, initial: f32) -> Entity {
    let min = camera::LOOK_SENSITIVITY_MIN;
    let max = camera::LOOK_SENSITIVITY_MAX;
    let normalized = ((initial - min) / (max - min)).clamp(0.0, 1.0);
    let text_slot = tree
        .text_state_mut()
        .cache
        .add_text(format!("{initial:.2}"));

    let track = tree
        .add_node()
        .flow_child(Ab(vec2(MENU_BUTTON_SIZE.x, MENU_SLIDER_HEIGHT)))
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

    let fill = tree.in_parent(track, |tree| {
        let fill = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(normalized * 100.0, 100.0)))
            .with_rect(1.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(ACCENT_DIM)
            .entity();
        tree.add_node()
            .window(
                Ab(vec2(20.0, 0.0)) + Rl(vec2(0.0, 50.0)),
                Ab(vec2(MENU_BUTTON_SIZE.x - 110.0, MENU_SLIDER_HEIGHT)),
                Anchor::CenterLeft,
            )
            .with_text(label, 18.0)
            .text_left()
            .color_raw::<UiBase>(TEXT_COLOR)
            .color_raw::<UiFocused>(WHITE)
            .with_transition::<UiFocused>(14.0, 8.0)
            .entity();
        tree.add_node()
            .window(
                Ab(vec2(-20.0, 0.0)) + Rl(vec2(100.0, 50.0)),
                Ab(vec2(70.0, MENU_SLIDER_HEIGHT)),
                Anchor::CenterRight,
            )
            .with_text_slot(text_slot, 18.0)
            .text_right()
            .color_raw::<UiBase>(ACCENT)
            .entity();
        fill
    });

    tree.commands().set(
        track,
        UiSliderData {
            min,
            max,
            value: initial,
            changed: false,
            fill_entity: fill,
            text_slot,
            logarithmic: false,
            precision: 2,
            prefix: String::new(),
            suffix: String::new(),
        },
    );
    tree.edit::<nightshade_ui::components::UiNodeInteraction>(track, |interaction| {
        interaction.accessible_role = Some(AccessibleRole::Slider);
    });
    track
}

pub fn set_checkbox_value(
    scene: &mut crate::commands::Scene<'_, '_>,
    checkbox: Entity,
    value: bool,
) {
    let inner_entity = scene
        .storage
        .get::<nightshade_ui::components::UiCheckboxData>(checkbox)
        .map(|data| data.inner_entity);
    if let Some(data) = scene
        .storage
        .get_mut::<nightshade_ui::components::UiCheckboxData>(checkbox)
    {
        data.value = value;
    }
    if let Some(inner_entity) = inner_entity {
        {
            let commands = &mut *scene.commands;
            ui_set_visible(scene.stage.dirty, commands, inner_entity, value);
        }
    }
}

fn swap_panels(scene: &mut crate::commands::Scene<'_, '_>, hide: Entity, show: Entity) {
    {
        let commands = &mut *scene.commands;
        let dirty = &mut *scene.stage.dirty;
        ui_set_visible(dirty, commands, hide, false);
        ui_set_visible(dirty, commands, show, true);
    }
}

pub fn handle_input(
    mut sandbox_world: ResMut<crate::resources::SandboxResources>,
    screen_state: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
    stage: crate::commands::Stage,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    if screen_state.current != Screen::Paused {
        return;
    }
    let mut owned = crate::commands::Scene {
        storage,
        commands: &mut commands,
        stage,
    };
    let scene = &mut owned;
    let resume = sandbox_world.ui_handles.pause.resume_button;
    let main_menu = sandbox_world.ui_handles.pause.main_menu_button;
    let quit = sandbox_world.ui_handles.pause.quit_button;
    let settings = sandbox_world.ui_handles.pause.settings_button;
    let back = sandbox_world.ui_handles.pause.back_button;
    let main_panel = sandbox_world.ui_handles.pause.main_panel;
    let settings_panel = sandbox_world.ui_handles.pause.settings_panel;
    let mut clicked_resume = false;
    let mut clicked_main_menu = false;
    let mut clicked_quit = false;
    let mut clicked_settings = false;
    let mut clicked_back = false;
    for entity in ui_button_clicks(scene.stage.frame) {
        if entity == resume {
            clicked_resume = true;
        } else if entity == main_menu {
            clicked_main_menu = true;
        } else if entity == quit {
            clicked_quit = true;
        } else if entity == settings {
            clicked_settings = true;
        } else if entity == back {
            clicked_back = true;
        }
    }
    if clicked_settings {
        swap_panels(scene, main_panel, settings_panel);
    }
    if clicked_back {
        swap_panels(scene, settings_panel, main_panel);
    }
    let checkbox = sandbox_world.ui_handles.pause.infinite_world_checkbox;
    if let Some(value) = ui_checkbox_changed(scene.storage, checkbox) {
        infinite::set_enabled(&mut sandbox_world, scene, value);
    }
    let third_person_box = sandbox_world.ui_handles.pause.third_person_checkbox;
    if let Some(value) = ui_checkbox_changed(scene.storage, third_person_box) {
        character::set_third_person(&mut sandbox_world, scene, value);
    }
    let spheres_box = sandbox_world.ui_handles.pause.spheres_checkbox;
    if let Some(value) = ui_checkbox_changed(scene.storage, spheres_box) {
        sandbox_world.character.spheres_visible = value;
    }
    let stride_wheel_box = sandbox_world.ui_handles.pause.stride_wheel_checkbox;
    if let Some(value) = ui_checkbox_changed(scene.storage, stride_wheel_box) {
        sandbox_world.character.stride_wheel_visible = value;
    }
    let skeleton_box = sandbox_world.ui_handles.pause.skeleton_checkbox;
    if let Some(value) = ui_checkbox_changed(scene.storage, skeleton_box) {
        sandbox_world.character.skeleton_visible = value;
    }
    let fly_box = sandbox_world.ui_handles.pause.fly_checkbox;
    if let Some(value) = ui_checkbox_changed(scene.storage, fly_box) {
        player::set_fly(&mut sandbox_world, scene, value);
    }
    let fps_box = sandbox_world.ui_handles.pause.show_fps_checkbox;
    if let Some(value) = ui_checkbox_changed(scene.storage, fps_box) {
        sandbox_world.screen.show_fps = value;
    }
    let sensitivity = sandbox_world.ui_handles.pause.sensitivity_slider;
    if let Some(value) = ui_slider_value_changed(scene.storage, sensitivity) {
        camera::set_look_sensitivity(&mut sandbox_world, scene, value);
    }
    if clicked_resume {
        swap_panels(scene, settings_panel, main_panel);
        next.pending = Some(Screen::InGame);
    } else if clicked_main_menu {
        player::reset(&mut sandbox_world, scene);
        enemies::despawn(scene);
        next.pending = Some(Screen::Title);
    } else if clicked_quit {
        scene.stage.window.should_exit = true;
    }
}
