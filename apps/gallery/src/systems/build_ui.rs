use nightshade::ecs::ui::components::{ThemeEffect, ThemeShadow};
use nightshade::prelude::*;

use crate::ecs::GalleryWorld;

const SIDEBAR_WIDTH: f32 = 220.0;
const TOP_BAR_HEIGHT: f32 = 56.0;
const PAGE_PADDING: f32 = 32.0;
const BLOCK_PADDING: f32 = 20.0;
const BLOCK_GAP: f32 = 16.0;
const BLOCK_RADIUS: f32 = 8.0;

struct NavItem {
    label: &'static str,
    icon: IconGlyph,
    description: &'static str,
}

const NAV_ITEMS: [NavItem; 7] = [
    NavItem {
        label: "Buttons",
        icon: icons::material::TOGGLE_ON,
        description: "Buttons, toggles, checkboxes, radios, selectables",
    },
    NavItem {
        label: "Inputs",
        icon: icons::material::KEYBOARD,
        description: "Sliders, drag values, text fields, progress",
    },
    NavItem {
        label: "Pickers",
        icon: icons::material::PALETTE,
        description: "Dropdowns, color, date, multi-select, menus, tabs",
    },
    NavItem {
        label: "Containers",
        icon: icons::material::VIEW_QUILT,
        description: "Scroll areas, splitters, tiles, modals, breadcrumbs",
    },
    NavItem {
        label: "Data",
        icon: icons::material::TABLE_CHART,
        description: "Data grid, property grid, tree view, virtual list",
    },
    NavItem {
        label: "Effects",
        icon: icons::material::AUTO_AWESOME,
        description: "Animations, transitions, theme switching",
    },
    NavItem {
        label: "Polish",
        icon: icons::material::TUNE,
        description: "Hover lift, springs, focus glow, progress bar",
    },
];

pub fn build_gallery_ui(gallery: &mut GalleryWorld, world: &mut World) {
    let mut tree = UiTreeBuilder::new(world);

    let root = tree
        .add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .with_theme_color::<UiBase>(ThemeColor::Background)
        .without_pointer_events()
        .entity();

    tree.push_parent(root);

    let (fps_text, entity_count_text, uptime_text) = build_top_bar(&mut tree);
    let (nav_buttons, nav_indicators, nav_labels, nav_slider, nav_y_offsets) =
        build_sidebar(&mut tree);

    let content_area = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(SIDEBAR_WIDTH, TOP_BAR_HEIGHT)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .with_theme_color::<UiBase>(ThemeColor::Background)
        .without_pointer_events()
        .entity();

    tree.push_parent(content_area);

    let mut screen_roots = Vec::new();
    let (buttons_screen, pulse_entity, disable_toggle, disabled_slider) =
        build_buttons_screen(&mut tree);
    screen_roots.push(buttons_screen);
    let (inputs_screen, progress_entity, submit_input, submit_log, focus_target, focus_button) =
        build_inputs_screen(&mut tree);
    screen_roots.push(inputs_screen);
    let (pickers_screen, palette_btn) = build_pickers_screen(&mut tree);
    screen_roots.push(pickers_screen);
    let (containers_screen, modal_trigger, modal_dialog, confirm_trigger, confirm_dialog) =
        build_containers_screen(&mut tree);
    screen_roots.push(containers_screen);
    let (data_screen, virtual_list, virtual_list_text_nodes) = build_data_screen(&mut tree);
    screen_roots.push(data_screen);
    let (effects_screen, effects_pulse, effects_blend, effects_progress, effects_spinner) =
        build_effects_screen(&mut tree);
    screen_roots.push(effects_screen);
    let (polish_screen, progress_slider, progress_indeterminate_btn, progress_clear_btn) =
        build_polish_screen(&mut tree);
    screen_roots.push(polish_screen);

    tree.pop_parent();

    let palette_entity = tree.add_command_palette(8);
    for (label, shortcut, category) in [
        ("New File", "Ctrl+N", "File"),
        ("Open File", "Ctrl+O", "File"),
        ("Save File", "Ctrl+S", "File"),
        ("Find", "Ctrl+F", "Edit"),
        ("Replace", "Ctrl+H", "Edit"),
        ("Undo", "Ctrl+Z", "Edit"),
        ("Redo", "Ctrl+Y", "Edit"),
        ("Toggle Theme", "", "View"),
        ("Toggle Fullscreen", "F11", "View"),
        ("Quit", "Ctrl+Q", "App"),
    ] {
        ui_command_palette_register(tree.world_mut(), palette_entity, label, shortcut, category);
    }

    tree.pop_parent();
    tree.finish();

    let handles = &mut gallery.resources.handles;
    handles.nav_buttons = nav_buttons.clone();
    handles.nav_indicators = nav_indicators.clone();
    handles.nav_labels = nav_labels.clone();
    handles.screen_roots = screen_roots;
    handles.fps_text = Some(fps_text);
    handles.uptime_text = Some(uptime_text);
    handles.entity_count_text = Some(entity_count_text);
    handles.progress_entity = Some(progress_entity);
    handles.pulse_entity = Some(pulse_entity);
    handles.effects_pulse = Some(effects_pulse);
    handles.effects_blend = Some(effects_blend);
    handles.effects_progress = Some(effects_progress);
    handles.effects_spinner = Some(effects_spinner);
    handles.modal_trigger = Some(modal_trigger);
    handles.modal_dialog = Some(modal_dialog);
    handles.confirm_trigger = Some(confirm_trigger);
    handles.confirm_dialog = Some(confirm_dialog);
    handles.submit_input = Some(submit_input);
    handles.submit_log = Some(submit_log);
    handles.focus_target = Some(focus_target);
    handles.focus_button = Some(focus_button);
    handles.palette_btn = Some(palette_btn);
    handles.palette_entity = Some(palette_entity);
    handles.disable_toggle = Some(disable_toggle);
    handles.disabled_slider = Some(disabled_slider);
    handles.virtual_list = Some(virtual_list);
    handles.virtual_list_text_nodes = virtual_list_text_nodes;
    handles.nav_slider = Some(nav_slider);
    handles.nav_y_offsets = nav_y_offsets.clone();
    handles.nav_slider_y = nav_y_offsets.first().copied().unwrap_or(0.0);
    handles.nav_slider_target_y = handles.nav_slider_y;
    handles.progress_slider = Some(progress_slider);
    handles.progress_indeterminate_btn = Some(progress_indeterminate_btn);
    handles.progress_clear_btn = Some(progress_clear_btn);
}

pub fn poll_gallery(gallery: &mut GalleryWorld, world: &mut World) {
    let nav_buttons = gallery.resources.handles.nav_buttons.clone();
    let nav_labels = gallery.resources.handles.nav_labels.clone();
    let screen_roots = gallery.resources.handles.screen_roots.clone();
    let modal_trigger = gallery.resources.handles.modal_trigger;
    let modal_dialog = gallery.resources.handles.modal_dialog;
    let confirm_trigger = gallery.resources.handles.confirm_trigger;
    let confirm_dialog = gallery.resources.handles.confirm_dialog;
    let submit_input = gallery.resources.handles.submit_input;
    let focus_target = gallery.resources.handles.focus_target;
    let focus_button = gallery.resources.handles.focus_button;
    let palette_btn = gallery.resources.handles.palette_btn;
    let palette_entity = gallery.resources.handles.palette_entity;
    let disable_toggle = gallery.resources.handles.disable_toggle;
    let disabled_slider = gallery.resources.handles.disabled_slider;
    let progress_slider = gallery.resources.handles.progress_slider;
    let progress_indeterminate_btn = gallery.resources.handles.progress_indeterminate_btn;
    let progress_clear_btn = gallery.resources.handles.progress_clear_btn;

    let mut new_active: Option<usize> = None;
    let mut show_modal: Option<Entity> = None;
    let mut show_confirm: Option<Entity> = None;
    let mut toast: Option<(&'static str, ToastSeverity)> = None;
    let mut submit_count_increment = 0u32;
    let mut focus_request: Option<Entity> = None;
    let mut palette_request: Option<Entity> = None;
    let mut button_toast: Option<(String, ToastSeverity)> = None;
    let mut progress_action: Option<ProgressAction> = None;

    for event in ui_events(world) {
        match event {
            UiEvent::ButtonClicked(entity) => {
                let mut consumed = false;
                if Some(*entity) == modal_trigger
                    && let Some(dialog) = modal_dialog
                {
                    show_modal = Some(dialog);
                    consumed = true;
                }
                if Some(*entity) == confirm_trigger
                    && let Some(dialog) = confirm_dialog
                {
                    show_confirm = Some(dialog);
                    consumed = true;
                }
                if Some(*entity) == focus_button
                    && let Some(target) = focus_target
                {
                    focus_request = Some(target);
                    consumed = true;
                }
                if Some(*entity) == progress_indeterminate_btn {
                    progress_action = Some(ProgressAction::Indeterminate);
                    consumed = true;
                }
                if Some(*entity) == progress_clear_btn {
                    progress_action = Some(ProgressAction::Clear);
                    consumed = true;
                }
                if Some(*entity) == palette_btn
                    && let Some(palette) = palette_entity
                {
                    palette_request = Some(palette);
                    consumed = true;
                }
                for (index, button) in nav_buttons.iter().enumerate() {
                    if entity == button {
                        new_active = Some(index);
                        consumed = true;
                    }
                }
                if !consumed && let Some(data) = widget::<UiButtonData>(world, *entity) {
                    let text_slot = data.text_slot;
                    let label = world
                        .resources
                        .text
                        .cache
                        .get_text(text_slot)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Clicked".to_string());
                    let lower = label.to_lowercase();
                    let severity = if lower.contains("success") {
                        ToastSeverity::Success
                    } else if lower.contains("warning") {
                        ToastSeverity::Warning
                    } else if lower.contains("error") || lower.contains("danger") {
                        ToastSeverity::Error
                    } else {
                        ToastSeverity::Info
                    };
                    button_toast = Some((label, severity));
                }
            }
            UiEvent::TextInputSubmitted { entity, .. } if Some(*entity) == submit_input => {
                submit_count_increment += 1;
            }
            UiEvent::SliderChanged { entity, value } if Some(*entity) == progress_slider => {
                progress_action = Some(ProgressAction::Set(*value));
            }
            UiEvent::ModalClosed { entity, confirmed } if Some(*entity) == confirm_dialog => {
                toast = Some(if *confirmed {
                    ("Confirmed", ToastSeverity::Success)
                } else {
                    ("Cancelled", ToastSeverity::Info)
                });
            }
            _ => {}
        }
    }

    if let Some(index) = new_active {
        let prev = gallery.resources.handles.active_screen;
        if index != prev {
            ui_set_visible(world, screen_roots[prev], false);
            update_nav_label_color(world, nav_labels.get(prev).copied(), false);
            ui_set_visible(world, screen_roots[index], true);
            update_nav_label_color(world, nav_labels.get(index).copied(), true);
            gallery.resources.handles.active_screen = index;
            if let Some(target_y) = gallery.resources.handles.nav_y_offsets.get(index).copied() {
                gallery.resources.handles.nav_slider_target_y = target_y;
            }
        }
    }

    if let Some(action) = progress_action {
        let bar = &mut world.resources.retained_ui.top_progress_bar;
        match action {
            ProgressAction::Set(value) => bar.set(value),
            ProgressAction::Indeterminate => bar.set_indeterminate(),
            ProgressAction::Clear => bar.clear(),
        }
    }

    let slider_entity = gallery.resources.handles.nav_slider;
    let mut slider_y = gallery.resources.handles.nav_slider_y;
    let slider_target = gallery.resources.handles.nav_slider_target_y;
    let delta_time = world.resources.window.timing.delta_time.clamp(0.0, 0.05);
    let stiffness = 220.0_f32;
    let displacement = slider_y - slider_target;
    let damping_factor = (-stiffness * delta_time * 0.5).exp();
    slider_y = slider_target + displacement * damping_factor;
    if (slider_y - slider_target).abs() < 0.05 {
        slider_y = slider_target;
    }
    gallery.resources.handles.nav_slider_y = slider_y;
    if let Some(slider) = slider_entity {
        update_slider_y(world, slider, slider_y);
    }

    if let Some(target) = focus_request {
        ui_focus(world, target);
    }
    let ctrl_p = world
        .resources
        .input
        .keyboard
        .just_pressed(winit::keyboard::KeyCode::KeyP)
        && (world
            .resources
            .input
            .keyboard
            .is_key_pressed(winit::keyboard::KeyCode::ControlLeft)
            || world
                .resources
                .input
                .keyboard
                .is_key_pressed(winit::keyboard::KeyCode::ControlRight));
    if ctrl_p && let Some(palette) = palette_entity {
        ui_show_command_palette(world, palette);
    }
    if let Some(palette) = palette_request {
        ui_show_command_palette(world, palette);
    }
    if let (Some(toggle), Some(slider)) = (disable_toggle, disabled_slider) {
        let disabled = widget::<UiToggleData>(world, toggle)
            .map(|d| d.value)
            .unwrap_or(false);
        ui_set_disabled(world, slider, disabled);
    }
    if submit_count_increment > 0 {
        gallery.resources.handles.submit_count += submit_count_increment;
        if let Some(log_entity) = gallery.resources.handles.submit_log {
            let count = gallery.resources.handles.submit_count;
            let message = format!("Submitted {count} time(s)");
            if let Some(UiNodeContent::Text { text_slot, .. }) =
                world.ui.get_ui_node_content(log_entity)
            {
                let slot = *text_slot;
                world.resources.text.cache.set_text(slot, &message);
            }
        }
    }
    if let Some(dialog) = show_modal {
        ui_show_modal(world, dialog);
    }
    if let Some(dialog) = show_confirm {
        ui_show_modal(world, dialog);
    }
    if let Some((message, severity)) = toast {
        ui_show_toast(world, message, severity, 2.0);
    }
    if let Some((message, severity)) = button_toast {
        ui_show_toast(world, &message, severity, 2.0);
    }

    if let Some(virtual_list) = gallery.resources.handles.virtual_list {
        let visible_start = world
            .ui
            .get_ui_virtual_list(virtual_list)
            .map(|data| data.visible_start)
            .unwrap_or(0);
        for (pool_index, text_entity) in gallery
            .resources
            .handles
            .virtual_list_text_nodes
            .clone()
            .into_iter()
            .enumerate()
        {
            let item_index = visible_start + pool_index;
            ui_set_text(world, text_entity, &format!("Item {item_index}"));
        }
    }
}

enum ProgressAction {
    Set(f32),
    Indeterminate,
    Clear,
}

fn update_nav_label_color(world: &mut World, label: Option<Entity>, active: bool) {
    let Some(label) = label else { return };
    let theme = world
        .resources
        .retained_ui
        .theme_state
        .active_theme()
        .clone();
    if let Some(color) = world.ui.get_ui_node_color_mut(label) {
        color.colors[UiBase::INDEX] = Some(if active {
            theme.text_color
        } else {
            theme.text_color_disabled
        });
    }
}

fn update_slider_y(world: &mut World, slider: Entity, y: f32) {
    use nightshade::ecs::ui::layout_types::{UiLayoutType, WindowLayout};
    use nightshade::ecs::ui::units::{Ab, UiValue};
    let Some(node) = world.ui.get_ui_layout_node_mut(slider) else {
        return;
    };
    let Some(layout) = node.base_layout.as_mut() else {
        return;
    };
    let UiLayoutType::Window(WindowLayout { position, .. }) = layout else {
        return;
    };
    let new_position: UiValue<nalgebra_glm::Vec2> = Ab(nalgebra_glm::Vec2::new(0.0, y)).into();
    *position = new_position;
    nightshade::prelude::ui_mark_layout_dirty(world);
}

fn build_top_bar(tree: &mut UiTreeBuilder) -> (Entity, Entity, Entity) {
    let theme = tree.active_theme().clone();
    let bar = tree
        .add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
            Ab(nalgebra_glm::Vec2::new(0.0, TOP_BAR_HEIGHT))
                + Rl(nalgebra_glm::Vec2::new(100.0, 0.0)),
        )
        .with_rect(0.0, 0.0, theme.border_color)
        .with_theme_color::<UiBase>(ThemeColor::Panel)
        .with_depth(UiDepthMode::Set(10.0))
        .entity();

    tree.push_parent(bar);

    tree.add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(0.0, 100.0)) + Ab(nalgebra_glm::Vec2::new(0.0, -1.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_theme_color::<UiBase>(ThemeColor::Border)
        .without_pointer_events();

    let bar_center_y = TOP_BAR_HEIGHT * 0.5;
    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(PAGE_PADDING, bar_center_y)),
            Ab(nalgebra_glm::Vec2::new(220.0, 24.0)),
            Anchor::CenterLeft,
        )
        .with_text("Nightshade UI", 18.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::Text)
        .without_pointer_events();

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(PAGE_PADDING + 160.0, bar_center_y)),
            Ab(nalgebra_glm::Vec2::new(220.0, 20.0)),
            Anchor::CenterLeft,
        )
        .with_text("Component Gallery", 13.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
        .without_pointer_events();

    let stats_holder = tree
        .add_node()
        .window(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                + Ab(nalgebra_glm::Vec2::new(
                    -(PAGE_PADDING + 220.0 + 32.0),
                    bar_center_y,
                )),
            Ab(nalgebra_glm::Vec2::new(220.0, 30.0)),
            Anchor::CenterLeft,
        )
        .flow_with_alignment(
            FlowDirection::Horizontal,
            0.0,
            16.0,
            FlowAlignment::End,
            FlowAlignment::Center,
        )
        .without_pointer_events()
        .entity();
    tree.push_parent(stats_holder);
    let fps_text = tree
        .add_node()
        .flow_child(Ab(nalgebra_glm::Vec2::new(70.0, 18.0)))
        .with_text("0 fps", 12.0)
        .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
        .without_pointer_events()
        .done();
    let entity_count_text = tree
        .add_node()
        .flow_child(Ab(nalgebra_glm::Vec2::new(80.0, 18.0)))
        .with_text("0 entities", 12.0)
        .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
        .without_pointer_events()
        .done();
    let uptime_text = tree
        .add_node()
        .flow_child(Ab(nalgebra_glm::Vec2::new(50.0, 18.0)))
        .with_text("0:00", 12.0)
        .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
        .without_pointer_events()
        .done();
    tree.pop_parent();

    let picker_holder = tree
        .add_node()
        .window(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                + Ab(nalgebra_glm::Vec2::new(
                    -(PAGE_PADDING + 200.0),
                    bar_center_y,
                )),
            Ab(nalgebra_glm::Vec2::new(200.0, 32.0)),
            Anchor::CenterLeft,
        )
        .entity();
    tree.push_parent(picker_holder);
    tree.add_theme_dropdown();
    tree.pop_parent();

    tree.pop_parent();

    (fps_text, entity_count_text, uptime_text)
}

fn build_sidebar(
    tree: &mut UiTreeBuilder,
) -> (Vec<Entity>, Vec<Entity>, Vec<Entity>, Entity, Vec<f32>) {
    let theme = tree.active_theme().clone();
    let sidebar = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(0.0, TOP_BAR_HEIGHT)),
            Ab(nalgebra_glm::Vec2::new(SIDEBAR_WIDTH, 0.0))
                + Rl(nalgebra_glm::Vec2::new(0.0, 100.0)),
        )
        .with_rect(0.0, 0.0, theme.border_color)
        .with_theme_color::<UiBase>(ThemeColor::Panel)
        .without_pointer_events()
        .entity();

    tree.push_parent(sidebar);

    tree.add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(-1.0, 0.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_theme_color::<UiBase>(ThemeColor::Border)
        .without_pointer_events();

    let item_height = 36.0_f32;
    let item_gap = 2.0_f32;
    let top_padding = 16.0_f32;

    let mut nav_buttons = Vec::new();
    let mut nav_indicators = Vec::new();
    let mut nav_labels = Vec::new();
    let mut nav_y_offsets = Vec::new();

    let slider = tree
        .add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(0.0, top_padding)),
            Ab(nalgebra_glm::Vec2::new(3.0, item_height)),
            Anchor::TopLeft,
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_theme_color::<UiBase>(ThemeColor::Accent)
        .without_pointer_events()
        .done();

    for (index, item) in NAV_ITEMS.iter().enumerate() {
        let y = top_padding + index as f32 * (item_height + item_gap);
        nav_y_offsets.push(y);

        let placeholder = tree
            .add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(0.0, y)),
                Ab(nalgebra_glm::Vec2::new(3.0, item_height)),
                Anchor::TopLeft,
            )
            .with_rect(0.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
            .without_pointer_events()
            .done();
        nav_indicators.push(placeholder);

        let button = tree
            .add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(8.0, y)),
                Ab(nalgebra_glm::Vec2::new(SIDEBAR_WIDTH - 16.0, item_height)),
                Anchor::TopLeft,
            )
            .with_rect(6.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
            .with_transition::<UiHover>(12.0, 6.0)
            .with_transition::<UiPressed>(20.0, 10.0)
            .with_interaction()
            .with_tooltip(item.description)
            .with_cursor_icon(winit::window::CursorIcon::Pointer)
            .entity();

        tree.push_parent(button);

        tree.add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(12.0, 0.0)) + Rl(nalgebra_glm::Vec2::new(0.0, 50.0)),
                Ab(nalgebra_glm::Vec2::new(18.0, 18.0)),
                Anchor::CenterLeft,
            )
            .with_icon(item.icon, 16.0)
            .with_color::<UiBase>(if index == 0 {
                theme.text_color
            } else {
                theme.text_color_disabled
            })
            .without_pointer_events()
            .done();

        let label = tree
            .add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(40.0, 0.0)) + Rl(nalgebra_glm::Vec2::new(0.0, 50.0)),
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(-52.0, 22.0)),
                Anchor::CenterLeft,
            )
            .with_text(item.label, 14.0)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .with_color::<UiBase>(if index == 0 {
                theme.text_color
            } else {
                theme.text_color_disabled
            })
            .without_pointer_events()
            .done();

        tree.pop_parent();
        nav_buttons.push(button);
        nav_labels.push(label);
    }

    tree.pop_parent();

    (
        nav_buttons,
        nav_indicators,
        nav_labels,
        slider,
        nav_y_offsets,
    )
}

fn build_screen_root(tree: &mut UiTreeBuilder, visible: bool) -> Entity {
    tree.add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .with_visible(visible)
        .with_intro(UiAnimationType::Fade, 0.18)
        .without_pointer_events()
        .entity()
}

fn build_page_header(tree: &mut UiTreeBuilder, title: &str, subtitle: &str) {
    tree.add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 32.0)),
        )
        .with_text(title, 24.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::Text)
        .without_pointer_events()
        .done();
    tree.add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 22.0)),
        )
        .with_text(subtitle, 13.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
        .without_pointer_events()
        .done();
    tree.add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, BLOCK_GAP)),
        )
        .without_pointer_events()
        .done();
}

fn demo_block(
    tree: &mut UiTreeBuilder,
    title: &str,
    caption: &str,
    content_height: f32,
    body: impl FnOnce(&mut UiTreeBuilder, Entity),
) {
    let theme = tree.active_theme().clone();
    let header_height = 22.0 + 4.0 + 18.0;
    let total_height = BLOCK_PADDING * 2.0 + header_height + 12.0 + content_height;

    let block = tree
        .add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                + Ab(nalgebra_glm::Vec2::new(0.0, total_height)),
        )
        .with_rect(BLOCK_RADIUS, 1.0, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::Panel)
        .with_theme_effect_role(ThemeEffect::PanelEffect)
        .entity();

    tree.push_parent(block);

    let inner = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(BLOCK_PADDING, BLOCK_PADDING)),
            Ab(nalgebra_glm::Vec2::new(-BLOCK_PADDING, -BLOCK_PADDING))
                + Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .flow_with_alignment(
            FlowDirection::Vertical,
            0.0,
            4.0,
            FlowAlignment::Start,
            FlowAlignment::Start,
        )
        .without_pointer_events()
        .entity();

    tree.push_parent(inner);
    tree.add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 22.0)),
        )
        .with_text(title, 15.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::Text)
        .without_pointer_events()
        .done();
    if !caption.is_empty() {
        tree.add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 18.0)),
            )
            .with_text(caption, 12.0)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
            .without_pointer_events()
            .done();
    }
    tree.add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 12.0)),
        )
        .without_pointer_events()
        .done();

    let content = tree
        .add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                + Ab(nalgebra_glm::Vec2::new(0.0, content_height)),
        )
        .flow_with_alignment(
            FlowDirection::Vertical,
            0.0,
            8.0,
            FlowAlignment::Start,
            FlowAlignment::Start,
        )
        .without_pointer_events()
        .entity();
    tree.push_parent(content);
    body(tree, content);
    tree.pop_parent();

    tree.pop_parent();
    tree.pop_parent();
}

fn build_page_scroll(tree: &mut UiTreeBuilder) -> Entity {
    let pad_holder = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(PAGE_PADDING, PAGE_PADDING)),
            Ab(nalgebra_glm::Vec2::new(-PAGE_PADDING, -PAGE_PADDING))
                + Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .without_pointer_events()
        .entity();

    tree.push_parent(pad_holder);
    let scroll = tree.add_scroll_area_fill(0.0, BLOCK_GAP);
    let content = widget::<UiScrollAreaData>(tree.world_mut(), scroll)
        .map(|d| d.content_entity)
        .unwrap();
    tree.pop_parent();
    content
}

fn build_buttons_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Entity, Entity) {
    let screen = build_screen_root(tree, true);
    tree.push_parent(screen);

    let content = build_page_scroll(tree);
    tree.push_parent(content);

    let pulse_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let disable_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let disabled_slider_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));

    build_page_header(
        tree,
        "Buttons",
        "Standard button variants, toggle controls, and selection patterns.",
    );

    demo_block(
        tree,
        "Standard Button",
        "The default button style applies the active theme's button effect and shadow.",
        36.0,
        |t, _| {
            t.add_button("Primary action");
        },
    );
    demo_block(
        tree,
        "Colored Buttons",
        "Pass a color to add_button_colored. Hover, press, and focus glows still derive from the theme.",
        36.0 * 3.0 + 16.0,
        |t, _| {
            let theme = t.active_theme().clone();
            t.add_button_colored("Success", theme.success_color);
            t.add_button_colored("Warning", theme.warning_color);
            t.add_button_colored("Danger", theme.error_color);
        },
    );
    demo_block(
        tree,
        "Toggle Switch",
        "Two-state switch. Press feels physical via per-state scale on the knob.",
        24.0,
        |t, _| {
            t.add_toggle(false);
        },
    );
    demo_block(
        tree,
        "Checkbox",
        "Inline checkbox with label. Focus indicator uses a focused background tint.",
        28.0,
        |t, _| {
            t.add_checkbox("Enable notifications", false);
        },
    );
    demo_block(
        tree,
        "Radio Group",
        "Mutually exclusive set sharing a group id.",
        96.0,
        |t, _| {
            t.add_radio("Option A", 1, 0);
            t.add_radio("Option B", 1, 1);
            t.add_radio("Option C", 1, 2);
        },
    );
    demo_block(
        tree,
        "Selectable Labels",
        "Toggleable labels that act like buttons but read as list rows.",
        96.0,
        |t, _| {
            t.add_selectable_label("Renderer: wgpu", Some(2));
            t.add_selectable_label("Audio: rodio", Some(2));
            t.add_selectable_label("Physics: rapier", Some(2));
        },
    );
    let pulse_clone = pulse_cell.clone();
    demo_block(
        tree,
        "Spinner & Live Indicator",
        "An idle spinner and a sine-driven status pulse from the gallery animation system.",
        48.0,
        move |t, _| {
            t.add_spinner();
            let theme = t.active_theme().clone();
            let pulse = t
                .add_node()
                .flow_child(Ab(nalgebra_glm::Vec2::new(12.0, 12.0)))
                .with_rect(6.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_color::<UiBase>(theme.accent_color)
                .with_tooltip("Live status indicator")
                .done();
            pulse_clone.set(pulse);
        },
    );
    let disable_clone = disable_cell.clone();
    let disabled_slider_clone = disabled_slider_cell.clone();
    demo_block(
        tree,
        "Disabled State",
        "Toggle the switch to disable the slider below. Disabled widgets ignore pointer input.",
        72.0,
        move |t, _| {
            let toggle = t.add_toggle(false);
            disable_clone.set(toggle);
            let slider = t.add_slider(0.0, 100.0, 30.0);
            disabled_slider_clone.set(slider);
        },
    );

    tree.pop_parent();
    tree.pop_parent();
    (
        screen,
        pulse_cell.get(),
        disable_cell.get(),
        disabled_slider_cell.get(),
    )
}

fn build_inputs_screen(
    tree: &mut UiTreeBuilder,
) -> (Entity, Entity, Entity, Entity, Entity, Entity) {
    let screen = build_screen_root(tree, false);
    tree.push_parent(screen);
    let content = build_page_scroll(tree);
    tree.push_parent(content);

    let progress_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let submit_input_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let submit_log_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let focus_target_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let focus_button_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));

    build_page_header(
        tree,
        "Inputs",
        "Sliders, drag values, text fields, and progress feedback.",
    );

    demo_block(
        tree,
        "Slider",
        "Horizontal track with a focus glow when active.",
        24.0,
        |t, _| {
            t.add_slider(0.0, 100.0, 50.0);
        },
    );
    demo_block(
        tree,
        "Range Slider",
        "Two thumbs sharing a track for selecting a numeric range.",
        24.0,
        |t, _| {
            t.add_range_slider(0.0, 100.0, 25.0, 75.0);
        },
    );
    demo_block(
        tree,
        "Drag Value",
        "Click and drag to scrub. Holding focus lets you type a value directly.",
        32.0,
        |t, _| {
            t.add_drag_value_configured(DragValueConfig::new(-10.0, 10.0, 0.5).speed(0.01));
        },
    );
    demo_block(
        tree,
        "Text Input",
        "Single-line text field. Supports clipboard, selection, undo.",
        32.0,
        |t, _| {
            t.add_text_input("Type here...");
        },
    );
    let submit_input_clone = submit_input_cell.clone();
    let submit_log_clone = submit_log_cell.clone();
    demo_block(
        tree,
        "Submit Detection",
        "Press Enter to fire a TextInputSubmitted event. The log below increments per submit.",
        64.0,
        move |t, _| {
            let input = t.add_text_input("Press Enter to submit...");
            submit_input_clone.set(input);
            let log = t
                .add_node()
                .flow_child(
                    Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 22.0)),
                )
                .with_text("Press Enter to submit", 12.0)
                .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
                .without_pointer_events()
                .done();
            submit_log_clone.set(log);
        },
    );
    let focus_target_clone = focus_target_cell.clone();
    let focus_button_clone = focus_button_cell.clone();
    demo_block(
        tree,
        "Programmatic Focus",
        "Call ui_focus(world, target) to move keyboard focus to any widget.",
        72.0,
        move |t, _| {
            let target = t.add_text_input("Focus target");
            focus_target_clone.set(target);
            let btn = t.add_button("Focus the input above");
            focus_button_clone.set(btn);
        },
    );
    demo_block(
        tree,
        "Text Area",
        "Multi-line text with optional row count.",
        96.0,
        |t, _| {
            t.add_text_area("Multi-line input...", 4);
        },
    );
    demo_block(
        tree,
        "Rich Text Editor",
        "Multi-line editor with inline span styling.",
        96.0,
        |t, _| {
            t.add_rich_text_editor("Rich text content...", 4);
        },
    );
    let progress_clone = progress_cell.clone();
    demo_block(
        tree,
        "Progress Bar",
        "Live looping bar plus a static 75% fill.",
        56.0,
        move |t, _| {
            let progress = t.add_progress_bar(0.0);
            progress_clone.set(progress);
            t.add_progress_bar(0.75);
        },
    );

    tree.pop_parent();
    tree.pop_parent();
    (
        screen,
        progress_cell.get(),
        submit_input_cell.get(),
        submit_log_cell.get(),
        focus_target_cell.get(),
        focus_button_cell.get(),
    )
}

fn build_pickers_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity) {
    let screen = build_screen_root(tree, false);
    tree.push_parent(screen);
    let content = build_page_scroll(tree);
    tree.push_parent(content);

    let palette_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));

    build_page_header(
        tree,
        "Pickers",
        "Selection widgets for discrete values: dropdowns, color, date, multi-select.",
    );

    demo_block(
        tree,
        "Dropdown",
        "Single-select dropdown with searchable variant available.",
        32.0,
        |t, _| {
            t.add_dropdown(&["Low", "Medium", "High", "Ultra"], 1);
        },
    );
    demo_block(
        tree,
        "Tab Bar",
        "Horizontal tab strip. Each tab is keyboard-navigable.",
        36.0,
        |t, _| {
            t.add_tab_bar(&["General", "Audio", "Display", "Controls"], 0);
        },
    );
    demo_block(
        tree,
        "Color Picker",
        "Compact color picker with HSV controls.",
        140.0,
        |t, _| {
            t.add_color_picker(nalgebra_glm::Vec4::new(0.3, 0.5, 0.9, 1.0));
        },
    );
    demo_block(
        tree,
        "Color Wheel",
        "Circular HSV color wheel.",
        180.0,
        |t, _| {
            t.add_color_wheel(nalgebra_glm::Vec4::new(0.3, 0.5, 0.9, 1.0));
        },
    );
    demo_block(
        tree,
        "Date Picker",
        "Calendar with month navigation.",
        260.0,
        |t, _| {
            t.add_date_picker(2026, 5, 7);
        },
    );
    demo_block(
        tree,
        "Multi-Select",
        "Pick multiple values from a list. Selections highlight inline.",
        180.0,
        |t, _| {
            t.add_multi_select(&["Apple", "Banana", "Cherry", "Date", "Elderberry"]);
        },
    );
    demo_block(
        tree,
        "Menu",
        "Drop-down menu with item list.",
        36.0,
        |t, _| {
            t.add_menu("File", &["New", "Open", "Save", "Save As...", "Quit"]);
        },
    );
    let palette_clone = palette_cell.clone();
    demo_block(
        tree,
        "Command Palette",
        "Fuzzy-searchable command list. Press Ctrl+P or use the button below.",
        32.0,
        move |t, _| {
            let btn = t.add_button("Open Command Palette (Ctrl+P)");
            palette_clone.set(btn);
        },
    );

    tree.pop_parent();
    tree.pop_parent();
    (screen, palette_cell.get())
}

fn build_containers_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Entity, Entity, Entity) {
    let screen = build_screen_root(tree, false);
    tree.push_parent(screen);
    let content = build_page_scroll(tree);
    tree.push_parent(content);

    let modal_btn_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let confirm_btn_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));

    build_page_header(
        tree,
        "Containers",
        "Layout primitives and overlay surfaces.",
    );

    demo_block(
        tree,
        "Collapsing Header",
        "Click the header to expand and collapse the body.",
        140.0,
        |t, content| {
            let scope = t.add_collapsing_header("Click to expand", true);
            let body = widget::<UiCollapsingHeaderData>(t.world_mut(), scope)
                .map(|d| d.content_entity)
                .unwrap_or(content);
            t.push_parent(body);
            t.add_label("Content nested inside the collapsing header.");
            t.add_label("Click the header above to toggle visibility.");
            t.add_button("Nested button");
            t.pop_parent();
        },
    );
    demo_block(
        tree,
        "Splitter",
        "Drag the divider to redistribute space between two panels.",
        96.0,
        |t, _| {
            t.add_splitter(SplitDirection::Horizontal, 0.5);
        },
    );
    demo_block(
        tree,
        "Tile Container",
        "Dockable panes with drag-to-rearrange and tab-style organization.",
        180.0,
        |t, _| {
            let tc = t.add_tile_container(nalgebra_glm::Vec2::new(420.0, 160.0));
            for title in ["Scene", "Inspector", "Console"] {
                let _ = ui_tile_add_pane(t.world_mut(), tc, title);
            }
        },
    );
    demo_block(
        tree,
        "Breadcrumb",
        "Hierarchical navigation trail.",
        28.0,
        |t, _| {
            t.add_breadcrumb(&["Home", "Library", "Sample Item"]);
        },
    );
    demo_block(
        tree,
        "Scroll Area",
        "Fixed-size container with scrollable content overflow.",
        160.0,
        |t, _| {
            let scroll = t.add_scroll_area(nalgebra_glm::Vec2::new(0.0, 140.0));
            let content = widget::<UiScrollAreaData>(t.world_mut(), scroll)
                .map(|d| d.content_entity)
                .unwrap();
            t.push_parent(content);
            for index in 0..30 {
                t.add_label(&format!("Scrollable item {index}"));
            }
            t.pop_parent();
        },
    );
    let modal_btn_clone = modal_btn_cell.clone();
    let confirm_btn_clone = confirm_btn_cell.clone();
    demo_block(
        tree,
        "Modal Dialog",
        "Open a custom modal or a confirmation prompt with two outcomes.",
        72.0,
        move |t, _| {
            let modal_btn = t.add_button("Show Modal Dialog");
            modal_btn_clone.set(modal_btn);
            let confirm_btn = t.add_button("Show Confirm Dialog");
            confirm_btn_clone.set(confirm_btn);
        },
    );
    demo_block(
        tree,
        "Canvas",
        "Custom drawing surface that emits draw commands you implement.",
        72.0,
        |t, _| {
            t.add_canvas(nalgebra_glm::Vec2::new(280.0, 60.0));
        },
    );

    tree.pop_parent();

    let modal_dialog = tree.add_modal_dialog("Custom Modal", 380.0, 220.0);
    let modal_content = widget::<UiModalDialogData>(tree.world_mut(), modal_dialog)
        .map(|data| data.content_entity)
        .unwrap_or(modal_dialog);
    tree.build_ui(modal_content, |inner| {
        inner.heading("This is a custom modal");
        inner.separator();
        inner.label("Click outside the dialog to dismiss it.");
        inner.spacing(6.0);
        inner.label("Or close it programmatically from your app.");
    });

    let confirm_dialog =
        tree.add_confirm_dialog("Confirm Action", "Are you sure you want to proceed?");

    tree.add_floating_panel(
        "gallery_floating",
        "Floating Panel",
        Rect {
            min: nalgebra_glm::Vec2::new(SIDEBAR_WIDTH + 280.0, TOP_BAR_HEIGHT + 200.0),
            max: nalgebra_glm::Vec2::new(SIDEBAR_WIDTH + 520.0, TOP_BAR_HEIGHT + 360.0),
        },
    );

    tree.pop_parent();
    (
        screen,
        modal_btn_cell.get(),
        modal_dialog,
        confirm_btn_cell.get(),
        confirm_dialog,
    )
}

fn build_data_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Vec<Entity>) {
    let screen = build_screen_root(tree, false);
    tree.push_parent(screen);
    let content = build_page_scroll(tree);
    tree.push_parent(content);

    let virtual_list_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let text_nodes_cell = std::rc::Rc::new(std::cell::RefCell::new(Vec::<Entity>::new()));

    build_page_header(tree, "Data", "Tabular and hierarchical data widgets.");

    demo_block(
        tree,
        "Property Grid",
        "Two-column inspector grid with sections, drag values, and toggles.",
        220.0,
        |t, _| {
            let grid = t.add_property_grid(110.0);
            let transform_section = t.add_property_section(grid, "Transform");
            for (label, default) in [("X", 0.0_f32), ("Y", 1.0), ("Z", 0.0)] {
                let area = t.add_property_row(grid, transform_section, label);
                t.push_parent(area);
                t.add_drag_value(-10.0, 10.0, default);
                t.pop_parent();
            }
            let appearance = t.add_property_section(grid, "Appearance");
            let area = t.add_property_row(grid, appearance, "Opacity");
            t.push_parent(area);
            t.add_drag_value(0.0, 1.0, 0.85);
            t.pop_parent();
            let area = t.add_property_row(grid, appearance, "Hidden");
            t.push_parent(area);
            t.add_toggle(false);
            t.pop_parent();
        },
    );
    demo_block(
        tree,
        "Data Grid",
        "Sortable, scrollable, multi-column grid with cell-level mutation.",
        320.0,
        |t, _| {
            let grid = t.add_data_grid(
                &[
                    DataGridColumn::new("Name", 140.0),
                    DataGridColumn::new("Type", 90.0),
                    DataGridColumn::new("Value", 110.0),
                ],
                10,
            );
            let sample_rows = [
                ("position.x", "f32", "0.00"),
                ("position.y", "f32", "1.00"),
                ("position.z", "f32", "0.00"),
                ("scale", "f32", "1.00"),
                ("visible", "bool", "true"),
                ("layer", "i32", "0"),
                ("name", "str", "Player"),
                ("tag", "str", "main"),
            ];
            ui_data_grid_set_row_count(t.world_mut(), grid, sample_rows.len());
            for (row, (name, ty, value)) in sample_rows.iter().enumerate() {
                ui_data_grid_set_cell(t.world_mut(), grid, row, 0, name);
                ui_data_grid_set_cell(t.world_mut(), grid, row, 1, ty);
                ui_data_grid_set_cell(t.world_mut(), grid, row, 2, value);
            }
        },
    );
    let vl_clone = virtual_list_cell.clone();
    let text_nodes_clone = text_nodes_cell.clone();
    demo_block(
        tree,
        "Virtual List",
        "64 items rendered through a small recycling pool. Scroll to see indices update.",
        220.0,
        move |t, _| {
            let vl = t.add_virtual_list(28.0, 8);
            vl_clone.set(vl);
            ui_virtual_list_set_count(t.world_mut(), vl, 64);
            let pool_containers: Vec<Entity> = t
                .world_mut()
                .ui
                .get_ui_virtual_list(vl)
                .map(|d| d.pool_items.iter().map(|p| p.container_entity).collect())
                .unwrap_or_default();
            let mut text_nodes = text_nodes_clone.borrow_mut();
            text_nodes.clear();
            for (pool_index, container) in pool_containers.iter().enumerate() {
                t.push_parent(*container);
                let text_entity = t
                    .add_node()
                    .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 100.0)))
                    .with_text(&format!("Item {pool_index}"), 13.0)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .without_pointer_events()
                    .entity();
                text_nodes.push(text_entity);
                t.pop_parent();
            }
        },
    );
    demo_block(
        tree,
        "Tree View",
        "Hierarchical tree with expansion state and selection.",
        260.0,
        |t, _| {
            let tree_view = t.add_tree_view(false);
            let tv_content = widget::<UiTreeViewData>(t.world_mut(), tree_view)
                .map(|d| d.content_entity)
                .unwrap();
            let project = t.add_tree_node(tree_view, tv_content, "Project", 0, 0);
            ui_tree_node_set_expanded(t.world_mut(), project, true);
            let project_children = widget::<UiTreeNodeData>(t.world_mut(), project)
                .map(|d| d.children_container)
                .unwrap();
            t.add_tree_node(tree_view, project_children, "Assets", 1, 1);
            t.add_tree_node(tree_view, project_children, "Scripts", 1, 2);
            let scenes = t.add_tree_node(tree_view, project_children, "Scenes", 1, 3);
            ui_tree_node_set_expanded(t.world_mut(), scenes, true);
            let scene_children = widget::<UiTreeNodeData>(t.world_mut(), scenes)
                .map(|d| d.children_container)
                .unwrap();
            t.add_tree_node(tree_view, scene_children, "main.scn", 2, 4);
            t.add_tree_node(tree_view, scene_children, "menu.scn", 2, 5);
        },
    );

    tree.pop_parent();
    tree.pop_parent();
    let virtual_list = virtual_list_cell.get();
    let text_nodes = text_nodes_cell.borrow().clone();
    (screen, virtual_list, text_nodes)
}

fn build_effects_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Entity, Entity, Entity) {
    let screen = build_screen_root(tree, false);
    tree.push_parent(screen);
    let content = build_page_scroll(tree);
    tree.push_parent(content);

    let pulse_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let blend_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let progress_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let spinner_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));

    build_page_header(
        tree,
        "Effects",
        "State transitions, tooltips, toasts, theme switching, and live animation.",
    );

    demo_block(
        tree,
        "Hover & Press Transitions",
        "Eased state-weight color blending. Hover to see CubicOut + BackOut on entry.",
        36.0 * 3.0 + 16.0,
        |t, _| {
            for label in ["Smooth Hover", "Smooth Press", "Both Effects"] {
                t.add_button(label);
            }
        },
    );
    demo_block(
        tree,
        "Tooltips",
        "Hover the button below for a tooltip. Configurable per-entity via with_tooltip().",
        40.0,
        |t, _| {
            let theme = t.active_theme().clone();
            t.add_node()
                .flow_child(
                    Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 32.0)),
                )
                .with_rect(6.0, 1.0, theme.border_color)
                .with_theme_border_color(ThemeColor::Border)
                .with_theme_color::<UiBase>(ThemeColor::Background)
                .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                .with_transition::<UiHover>(8.0, 6.0)
                .with_interaction()
                .with_tooltip("This is a tooltip on a custom button")
                .with_cursor_icon(winit::window::CursorIcon::Pointer)
                .with_children(|inner| {
                    inner
                        .add_node()
                        .window(
                            Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                            Ab(nalgebra_glm::Vec2::new(220.0, 22.0)),
                            Anchor::Center,
                        )
                        .with_text("Hover for tooltip", 13.0)
                        .with_theme_color::<UiBase>(ThemeColor::Text)
                        .without_pointer_events();
                })
                .done();
        },
    );
    demo_block(
        tree,
        "Toast Notifications",
        "Click any of the buttons to fire a toast at that severity. They auto-dismiss after 2s.",
        36.0 * 4.0 + 24.0,
        |t, _| {
            t.add_button("Show Info Toast");
            t.add_button("Show Success Toast");
            t.add_button("Show Warning Toast");
            t.add_button("Show Error Toast");
        },
    );
    demo_block(
        tree,
        "Theme Switching",
        "Use the dropdown in the top bar. Every color token crossfades over 250ms.",
        24.0,
        |_, _| {},
    );
    let pulse_clone = pulse_cell.clone();
    let blend_clone = blend_cell.clone();
    let progress_clone = progress_cell.clone();
    let spinner_clone = spinner_cell.clone();
    demo_block(
        tree,
        "Live Animation",
        "Pulse, color blend, looping progress, and a rotating swatch driven from gallery animation systems.",
        220.0,
        move |t, _| {
            let theme = t.active_theme().clone();
            let pulse = t
                .add_node()
                .flow_child(Ab(nalgebra_glm::Vec2::new(64.0, 64.0)))
                .with_rect(8.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_color::<UiBase>(theme.accent_color)
                .without_pointer_events()
                .entity();
            pulse_clone.set(pulse);
            let blend = t
                .add_node()
                .flow_child(
                    Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 6.0)),
                )
                .with_rect(3.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_color::<UiBase>(theme.accent_color)
                .without_pointer_events()
                .entity();
            blend_clone.set(blend);
            let progress = t.add_progress_bar(0.0);
            progress_clone.set(progress);
            let spinner_holder = t
                .add_node()
                .flow_child(Ab(nalgebra_glm::Vec2::new(60.0, 60.0)))
                .without_pointer_events()
                .entity();
            t.push_parent(spinner_holder);
            let spinner = t
                .add_node()
                .window(
                    Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                    Ab(nalgebra_glm::Vec2::new(20.0, 20.0)),
                    Anchor::Center,
                )
                .with_rect(4.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_color::<UiBase>(theme.accent_color)
                .without_pointer_events()
                .done();
            spinner_clone.set(spinner);
            t.pop_parent();
        },
    );

    tree.pop_parent();
    tree.pop_parent();
    (
        screen,
        pulse_cell.get(),
        blend_cell.get(),
        progress_cell.get(),
        spinner_cell.get(),
    )
}

fn build_polish_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Entity, Entity) {
    use nightshade::ecs::ui::components::Spring;

    let screen = build_screen_root(tree, false);
    tree.push_parent(screen);
    let content = build_page_scroll(tree);
    tree.push_parent(content);

    let progress_slider_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let progress_indet_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let progress_clear_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));

    build_page_header(
        tree,
        "Polish",
        "Hover lift, press compression, focus glow, spring tuning, and the top progress bar.",
    );

    demo_block(
        tree,
        "Hover Lift + Press Compress",
        "Standard buttons lift 2px on hover (BackOut spring) and sink 2px on press (snappy spring).",
        36.0 * 3.0 + 16.0,
        |t, _| {
            t.add_button("Hover me");
            t.add_button("Then press me");
            let theme = t.active_theme().clone();
            t.add_button_colored("Colored too", theme.accent_color);
        },
    );
    demo_block(
        tree,
        "Toggle & Slider Scale",
        "Toggle scales up subtly on hover, compresses on press. Slider gains a focus glow when active.",
        96.0,
        |t, _| {
            t.add_toggle(false);
            t.add_slider(0.0, 100.0, 50.0);
        },
    );
    demo_block(
        tree,
        "Focus Glow",
        "Tab through the inputs to see the focused border tint and tight focus ring.",
        96.0,
        |t, _| {
            t.add_text_input("First input");
            t.add_text_input("Second input");
        },
    );
    demo_block(
        tree,
        "Spring Tuning",
        "Same hover-lift offset, four different spring configs. Hover each to compare overshoot and damping.",
        36.0 * 4.0 + 24.0,
        |t, _| {
            spring_demo_button(
                t,
                "Soft (80, 12)",
                Spring {
                    stiffness: 80.0,
                    damping: 12.0,
                    mass: 1.0,
                },
            );
            spring_demo_button(t, "Lively (default)", Spring::lively());
            spring_demo_button(t, "Snappy", Spring::snappy());
            spring_demo_button(
                t,
                "Bouncy (350, 14)",
                Spring {
                    stiffness: 350.0,
                    damping: 14.0,
                    mass: 1.0,
                },
            );
        },
    );
    demo_block(
        tree,
        "Linear vs Spring",
        "Same offset, different motion. Linear feels mechanical; spring overshoots and settles.",
        36.0 * 2.0 + 8.0,
        |t, _| {
            linear_demo_button(t, "Linear hover");
            spring_demo_button(t, "Spring hover", Spring::lively());
        },
    );
    let ps = progress_slider_cell.clone();
    let pi = progress_indet_cell.clone();
    let pc = progress_clear_cell.clone();
    demo_block(
        tree,
        "Top-of-Window Progress Bar",
        "Drag the slider to set a determinate value, click indeterminate for a sliding bar, or clear to hide.",
        36.0 * 2.0 + 24.0 + 28.0,
        move |t, _| {
            let slider = t.add_slider(0.0, 1.0, 0.0);
            ps.set(slider);
            let indet = t.add_button("Set Indeterminate");
            pi.set(indet);
            let clear = t.add_button("Clear Progress Bar");
            pc.set(clear);
        },
    );

    tree.pop_parent();
    tree.pop_parent();
    (
        screen,
        progress_slider_cell.get(),
        progress_indet_cell.get(),
        progress_clear_cell.get(),
    )
}

fn spring_demo_button(
    tree: &mut UiTreeBuilder,
    label: &str,
    spring: nightshade::ecs::ui::components::Spring,
) -> Entity {
    use nalgebra_glm::Vec2;
    let theme = tree.active_theme().clone();
    tree.add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 32.0)),
        )
        .with_rect(theme.corner_radius, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::Background)
        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
        .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
        .with_theme_effect_role(ThemeEffect::ButtonEffect)
        .with_theme_shadow_role::<UiBase>(ThemeShadow::ButtonShadow)
        .with_theme_shadow_role::<UiHover>(ThemeShadow::ButtonShadowHover)
        .with_theme_shadow_role::<UiPressed>(ThemeShadow::ButtonShadowPressed)
        .with_state_offset::<UiHover>(Vec2::new(0.0, -3.0))
        .with_state_offset::<UiPressed>(Vec2::new(0.0, 2.0))
        .with_spring_transition::<UiHover>(spring)
        .with_spring_transition::<UiPressed>(spring)
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .with_children(|inner| {
            inner
                .add_node()
                .window(
                    Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                    Ab(nalgebra_glm::Vec2::new(240.0, 22.0)),
                    Anchor::Center,
                )
                .with_text(label, 14.0)
                .with_theme_color::<UiBase>(ThemeColor::Text)
                .without_pointer_events();
        })
        .done()
}

fn linear_demo_button(tree: &mut UiTreeBuilder, label: &str) -> Entity {
    use nalgebra_glm::Vec2;
    use nightshade::ecs::tween::easing::EasingFunction;
    let theme = tree.active_theme().clone();
    tree.add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 32.0)),
        )
        .with_rect(theme.corner_radius, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::Background)
        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
        .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
        .with_theme_effect_role(ThemeEffect::ButtonEffect)
        .with_theme_shadow_role::<UiBase>(ThemeShadow::ButtonShadow)
        .with_theme_shadow_role::<UiHover>(ThemeShadow::ButtonShadowHover)
        .with_state_offset::<UiHover>(Vec2::new(0.0, -3.0))
        .with_state_offset::<UiPressed>(Vec2::new(0.0, 2.0))
        .with_eased_transition::<UiHover>(8.0, 6.0, EasingFunction::Linear)
        .with_eased_transition::<UiPressed>(12.0, 8.0, EasingFunction::Linear)
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .with_children(|inner| {
            inner
                .add_node()
                .window(
                    Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                    Ab(nalgebra_glm::Vec2::new(240.0, 22.0)),
                    Anchor::Center,
                )
                .with_text(label, 14.0)
                .with_theme_color::<UiBase>(ThemeColor::Text)
                .without_pointer_events();
        })
        .done()
}
