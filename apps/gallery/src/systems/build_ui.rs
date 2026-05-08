use nightshade::prelude::*;

use crate::ecs::GalleryWorld;

const CYAN: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.9, 1.0, 1.0);
const CYAN_DIM: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.5, 0.6, 1.0);
const MAGENTA: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(1.0, 0.0, 0.6, 1.0);
const MAGENTA_DIM: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.6, 0.0, 0.35, 1.0);
const WHITE: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0);
const LIGHT_GRAY: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.75, 0.75, 0.8, 1.0);
const DARK_PANEL_HOVER: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.08, 0.08, 0.14, 1.0);
const TRANSPARENT: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0);
const CARD_BG: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.05, 0.05, 0.09, 0.9);
const CARD_BORDER: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.1, 0.1, 0.18, 0.4);
const ACTIVE_INDICATOR: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.9, 1.0, 0.8);
const INACTIVE_INDICATOR: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.9, 1.0, 0.0);

const SIDEBAR_PERCENT: f32 = 22.0;
const CONTENT_OFFSET_PERCENT: f32 = 22.5;

struct NavItem {
    label: &'static str,
    icon: IconGlyph,
    tooltip: &'static str,
}

const NAV_ITEMS: [NavItem; 7] = [
    NavItem {
        label: "BUTTONS",
        icon: icons::material::TOGGLE_ON,
        tooltip: "Buttons, toggles, checkboxes, radios, selectables",
    },
    NavItem {
        label: "INPUTS",
        icon: icons::material::KEYBOARD,
        tooltip: "Sliders, drag values, text inputs, progress",
    },
    NavItem {
        label: "PICKERS",
        icon: icons::material::PALETTE,
        tooltip: "Dropdowns, color, date, multi-select, menus, tabs",
    },
    NavItem {
        label: "CONTAINERS",
        icon: icons::material::VIEW_QUILT,
        tooltip: "Scroll areas, splitters, tiles, modals, breadcrumb",
    },
    NavItem {
        label: "DATA",
        icon: icons::material::TABLE_CHART,
        tooltip: "Data grid, property grid, tree view, virtual list",
    },
    NavItem {
        label: "EFFECTS",
        icon: icons::material::AUTO_AWESOME,
        tooltip: "Animations, transitions, theme switching",
    },
    NavItem {
        label: "POLISH",
        icon: icons::material::TUNE,
        tooltip: "Polish: shadows, focus rings, hover states",
    },
];

pub fn build_gallery_ui(gallery: &mut GalleryWorld, world: &mut World) {
    let mut tree = UiTreeBuilder::new(world);

    let root_panel = tree
        .add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .without_pointer_events()
        .entity();

    tree.push_parent(root_panel);

    build_top_bar(&mut tree);
    let (nav_buttons, nav_indicators, nav_labels, nav_slider, nav_y_offsets) =
        build_sidebar(&mut tree);

    let content_area = tree
        .add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(CONTENT_OFFSET_PERCENT, 0.0))
                + Ab(nalgebra_glm::Vec2::new(0.0, 50.0)),
            Ab(nalgebra_glm::Vec2::new(0.0, -40.0)) + Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .without_pointer_events()
        .entity();

    tree.push_parent(content_area);

    let mut screen_roots = Vec::new();
    let (
        buttons_screen,
        fps_text,
        uptime_text,
        entity_count_text,
        pulse_entity,
        disable_toggle,
        disabled_slider,
    ) = build_buttons_screen(&mut tree);
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

    build_bottom_bar(&mut tree);

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

    gallery.resources.handles.nav_buttons = nav_buttons.clone();
    gallery.resources.handles.nav_indicators = nav_indicators.clone();
    gallery.resources.handles.nav_labels = nav_labels.clone();
    gallery.resources.handles.nav_slider = Some(nav_slider);
    gallery.resources.handles.nav_y_offsets = nav_y_offsets.clone();
    gallery.resources.handles.nav_slider_y = nav_y_offsets.first().copied().unwrap_or(20.0);
    gallery.resources.handles.nav_slider_target_y = gallery.resources.handles.nav_slider_y;
    gallery.resources.handles.progress_slider = Some(progress_slider);
    gallery.resources.handles.progress_indeterminate_btn = Some(progress_indeterminate_btn);
    gallery.resources.handles.progress_clear_btn = Some(progress_clear_btn);
    gallery.resources.handles.screen_roots = screen_roots.clone();
    gallery.resources.handles.fps_text = Some(fps_text);
    gallery.resources.handles.uptime_text = Some(uptime_text);
    gallery.resources.handles.entity_count_text = Some(entity_count_text);
    gallery.resources.handles.progress_entity = Some(progress_entity);
    gallery.resources.handles.pulse_entity = Some(pulse_entity);
    gallery.resources.handles.effects_pulse = Some(effects_pulse);
    gallery.resources.handles.effects_blend = Some(effects_blend);
    gallery.resources.handles.effects_progress = Some(effects_progress);
    gallery.resources.handles.effects_spinner = Some(effects_spinner);
    gallery.resources.handles.modal_trigger = Some(modal_trigger);
    gallery.resources.handles.modal_dialog = Some(modal_dialog);
    gallery.resources.handles.confirm_trigger = Some(confirm_trigger);
    gallery.resources.handles.confirm_dialog = Some(confirm_dialog);
    gallery.resources.handles.submit_input = Some(submit_input);
    gallery.resources.handles.submit_log = Some(submit_log);
    gallery.resources.handles.focus_target = Some(focus_target);
    gallery.resources.handles.focus_button = Some(focus_button);
    gallery.resources.handles.palette_btn = Some(palette_btn);
    gallery.resources.handles.palette_entity = Some(palette_entity);
    gallery.resources.handles.disable_toggle = Some(disable_toggle);
    gallery.resources.handles.disabled_slider = Some(disabled_slider);
    gallery.resources.handles.virtual_list = Some(virtual_list);
    gallery.resources.handles.virtual_list_text_nodes = virtual_list_text_nodes;
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
                    ("Confirmed!", ToastSeverity::Success)
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
            if let Some(label) = nav_labels.get(prev).copied()
                && let Some(color) = world.ui.get_ui_node_color_mut(label)
            {
                color.colors[UiBase::INDEX] = Some(LIGHT_GRAY);
            }
            ui_set_visible(world, screen_roots[index], true);
            if let Some(label) = nav_labels.get(index).copied()
                && let Some(color) = world.ui.get_ui_node_color_mut(label)
            {
                color.colors[UiBase::INDEX] = Some(CYAN);
            }
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

fn build_top_bar(tree: &mut UiTreeBuilder) {
    let top_bar = tree
        .add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
            Ab(nalgebra_glm::Vec2::new(0.0, 48.0)) + Rl(nalgebra_glm::Vec2::new(100.0, 0.0)),
        )
        .with_rect(0.0, 0.0, TRANSPARENT)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.03, 0.03, 0.06, 0.98))
        .with_depth(UiDepthMode::Set(10.0))
        .entity();

    tree.push_parent(top_bar);

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(20.0, 24.0)),
            Ab(nalgebra_glm::Vec2::new(300.0, 30.0)),
            Anchor::CenterLeft,
        )
        .with_text("NIGHTSHADE", 22.0)
        .with_text_outline(nalgebra_glm::Vec4::new(0.0, 0.4, 0.5, 1.0), 1.5)
        .with_color::<UiBase>(CYAN)
        .without_pointer_events();

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(240.0, 24.0)),
            Ab(nalgebra_glm::Vec2::new(260.0, 22.0)),
            Anchor::CenterLeft,
        )
        .with_text("// RETAINED UI GALLERY", 14.0)
        .with_color::<UiBase>(MAGENTA)
        .without_pointer_events();

    tree.add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(0.0, 46.0)),
            Ab(nalgebra_glm::Vec2::new(0.0, 48.0)) + Rl(nalgebra_glm::Vec2::new(100.0, 0.0)),
        )
        .with_rect(0.0, 0.0, TRANSPARENT)
        .with_color::<UiBase>(CYAN_DIM)
        .without_pointer_events();

    let picker_holder = tree
        .add_node()
        .window(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(-200.0, 10.0)),
            Ab(nalgebra_glm::Vec2::new(180.0, 28.0)),
            Anchor::TopLeft,
        )
        .entity();
    tree.push_parent(picker_holder);
    tree.add_theme_dropdown();
    tree.pop_parent();

    tree.pop_parent();
}

fn build_bottom_bar(tree: &mut UiTreeBuilder) {
    let bottom_bar = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(0.0, -38.0)) + Rl(nalgebra_glm::Vec2::new(0.0, 100.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .with_rect(0.0, 0.0, TRANSPARENT)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.03, 0.03, 0.06, 0.98))
        .with_depth(UiDepthMode::Set(10.0))
        .without_pointer_events()
        .entity();

    tree.push_parent(bottom_bar);

    tree.add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
            Ab(nalgebra_glm::Vec2::new(0.0, 2.0)) + Rl(nalgebra_glm::Vec2::new(100.0, 0.0)),
        )
        .with_rect(0.0, 0.0, TRANSPARENT)
        .with_color::<UiBase>(MAGENTA_DIM)
        .without_pointer_events();

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(20.0, 20.0)),
            Ab(nalgebra_glm::Vec2::new(440.0, 20.0)),
            Anchor::CenterLeft,
        )
        .with_text("NIGHTSHADE ENGINE  //  RETAINED UI WIDGET GALLERY", 11.0)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.4, 0.4, 0.5, 1.0))
        .without_pointer_events();

    tree.pop_parent();
}

fn build_sidebar(
    tree: &mut UiTreeBuilder,
) -> (Vec<Entity>, Vec<Entity>, Vec<Entity>, Entity, Vec<f32>) {
    let mut nav_buttons = Vec::new();
    let mut nav_indicators = Vec::new();
    let mut nav_labels = Vec::new();
    let mut nav_y_offsets = Vec::new();

    let sidebar = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(0.0, 50.0)),
            Rl(nalgebra_glm::Vec2::new(SIDEBAR_PERCENT, 100.0))
                + Ab(nalgebra_glm::Vec2::new(0.0, -40.0)),
        )
        .with_rect(0.0, 0.0, TRANSPARENT)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.03, 0.03, 0.06, 0.95))
        .without_pointer_events()
        .entity();

    tree.push_parent(sidebar);

    let slider_indicator = tree
        .add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(0.0, 20.0)),
            Ab(nalgebra_glm::Vec2::new(3.0, 40.0)),
            Anchor::TopLeft,
        )
        .with_rect(0.0, 0.0, TRANSPARENT)
        .with_color::<UiBase>(ACTIVE_INDICATOR)
        .without_pointer_events()
        .done();

    for (index, item) in NAV_ITEMS.iter().enumerate() {
        let y_offset = 20.0 + index as f32 * 52.0;
        nav_y_offsets.push(y_offset);

        let indicator = tree
            .add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(0.0, y_offset)),
                Ab(nalgebra_glm::Vec2::new(3.0, 40.0)),
                Anchor::TopLeft,
            )
            .with_rect(0.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(INACTIVE_INDICATOR)
            .without_pointer_events()
            .done();
        nav_indicators.push(indicator);

        let button = tree
            .add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(8.0, y_offset)),
                Ab(nalgebra_glm::Vec2::new(0.0, 40.0))
                    + Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                    + Ab(nalgebra_glm::Vec2::new(-12.0, 0.0)),
                Anchor::TopLeft,
            )
            .with_interaction()
            .with_tooltip(item.tooltip)
            .with_cursor_icon(winit::window::CursorIcon::Pointer)
            .with_rect(4.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.05, 0.05, 0.09, 0.0))
            .with_color::<UiHover>(DARK_PANEL_HOVER)
            .with_color::<UiPressed>(nalgebra_glm::Vec4::new(0.04, 0.04, 0.08, 1.0))
            .with_transition::<UiHover>(12.0, 6.0)
            .with_transition::<UiPressed>(20.0, 10.0)
            .entity();

        tree.push_parent(button);
        tree.add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(16.0, 0.0)) + Rl(nalgebra_glm::Vec2::new(0.0, 50.0)),
                Ab(nalgebra_glm::Vec2::new(20.0, 20.0)),
                Anchor::CenterLeft,
            )
            .with_icon(item.icon, 18.0)
            .with_color::<UiBase>(if index == 0 { CYAN } else { LIGHT_GRAY })
            .without_pointer_events()
            .done();
        let label_entity = tree
            .add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(46.0, 0.0)) + Rl(nalgebra_glm::Vec2::new(0.0, 50.0)),
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(-58.0, 30.0)),
                Anchor::CenterLeft,
            )
            .with_text(item.label, 12.0)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .with_color::<UiBase>(if index == 0 { CYAN } else { LIGHT_GRAY })
            .without_pointer_events()
            .done();
        tree.pop_parent();
        nav_buttons.push(button);
        nav_labels.push(label_entity);
    }

    tree.add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(-2.0, 0.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .with_rect(0.0, 0.0, TRANSPARENT)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.1, 0.1, 0.18, 0.5))
        .without_pointer_events();

    tree.pop_parent();

    (
        nav_buttons,
        nav_indicators,
        nav_labels,
        slider_indicator,
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
        .with_intro(UiAnimationType::Fade, 0.2)
        .without_pointer_events()
        .entity()
}

fn build_card(tree: &mut UiTreeBuilder) -> Entity {
    tree.add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(10.0, 10.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)) + Ab(nalgebra_glm::Vec2::new(-10.0, -10.0)),
        )
        .with_rect(6.0, 1.0, CARD_BORDER)
        .with_color::<UiBase>(CARD_BG)
        .entity()
}

fn build_two_column_card(
    tree: &mut UiTreeBuilder,
    left: impl FnOnce(&mut Ui<'_, '_>),
    right: impl FnOnce(&mut Ui<'_, '_>),
) {
    let left_card = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(10.0, 10.0)),
            Rl(nalgebra_glm::Vec2::new(50.0, 0.0))
                + Ab(nalgebra_glm::Vec2::new(-5.0, -10.0))
                + Rl(nalgebra_glm::Vec2::new(0.0, 100.0)),
        )
        .with_rect(6.0, 1.0, CARD_BORDER)
        .with_color::<UiBase>(CARD_BG)
        .entity();

    tree.push_parent(left_card);
    let scroll = tree.add_scroll_area_fill(12.0, 6.0);
    let content = widget::<UiScrollAreaData>(tree.world_mut(), scroll)
        .map(|d| d.content_entity)
        .unwrap();
    tree.build_ui(content, left);
    tree.pop_parent();

    let right_card = tree
        .add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(50.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(5.0, 10.0)),
            Ab(nalgebra_glm::Vec2::new(-10.0, -10.0)) + Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .with_rect(6.0, 1.0, CARD_BORDER)
        .with_color::<UiBase>(CARD_BG)
        .entity();

    tree.push_parent(right_card);
    let scroll = tree.add_scroll_area_fill(12.0, 6.0);
    let content = widget::<UiScrollAreaData>(tree.world_mut(), scroll)
        .map(|d| d.content_entity)
        .unwrap();
    tree.build_ui(content, right);
    tree.pop_parent();
}

fn build_buttons_screen(
    tree: &mut UiTreeBuilder,
) -> (Entity, Entity, Entity, Entity, Entity, Entity, Entity) {
    let screen = build_screen_root(tree, true);
    tree.push_parent(screen);

    let card_data: [(&str, &str, nalgebra_glm::Vec4); 3] = [
        ("FPS", "60", CYAN),
        ("ENTITIES", "0", MAGENTA),
        (
            "UPTIME",
            "0:00",
            nalgebra_glm::Vec4::new(0.4, 1.0, 0.4, 1.0),
        ),
    ];

    let mut card_text_entities = [Entity::default(); 3];
    for (index, (label, initial_text, accent_color)) in card_data.iter().enumerate() {
        let x_percent = 2.0 + index as f32 * 33.0;

        let card = tree
            .add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(x_percent, 0.0))
                    + Ab(nalgebra_glm::Vec2::new(0.0, 10.0)),
                Rl(nalgebra_glm::Vec2::new(x_percent + 31.0, 0.0))
                    + Ab(nalgebra_glm::Vec2::new(0.0, 110.0)),
            )
            .with_rect(6.0, 1.0, CARD_BORDER)
            .with_color::<UiBase>(CARD_BG)
            .with_tooltip(match index {
                0 => "Current frames per second",
                1 => "Total active ECS entities",
                _ => "Time since application start",
            })
            .entity();

        tree.push_parent(card);

        tree.add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
                Ab(nalgebra_glm::Vec2::new(0.0, 3.0)) + Rl(nalgebra_glm::Vec2::new(100.0, 0.0)),
            )
            .with_rect(6.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(*accent_color)
            .without_pointer_events();

        tree.add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(16.0, 18.0)),
                Ab(nalgebra_glm::Vec2::new(150.0, 20.0)),
                Anchor::TopLeft,
            )
            .with_text(label, 12.0)
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.5, 0.5, 0.6, 1.0))
            .without_pointer_events();

        card_text_entities[index] = tree
            .add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(16.0, 45.0)),
                Ab(nalgebra_glm::Vec2::new(200.0, 42.0)),
                Anchor::TopLeft,
            )
            .with_text(initial_text, 32.0)
            .with_color::<UiBase>(WHITE)
            .without_pointer_events()
            .done();

        tree.pop_parent();
    }

    let fps_text = card_text_entities[0];
    let entity_count_text = card_text_entities[1];
    let uptime_text = card_text_entities[2];

    let pulse_entity = tree
        .add_node()
        .window(
            Rl(nalgebra_glm::Vec2::new(0.0, 100.0)) + Ab(nalgebra_glm::Vec2::new(20.0, -60.0)),
            Ab(nalgebra_glm::Vec2::new(12.0, 12.0)),
            Anchor::Center,
        )
        .with_rect(6.0, 0.0, TRANSPARENT)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.0, 1.0, 0.5, 1.0))
        .with_tooltip("Live status indicator (animated)")
        .done();

    let card = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(10.0, 130.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)) + Ab(nalgebra_glm::Vec2::new(-10.0, -10.0)),
        )
        .with_rect(6.0, 1.0, CARD_BORDER)
        .with_color::<UiBase>(CARD_BG)
        .entity();

    tree.push_parent(card);

    let disable_toggle_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let disabled_slider_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let dt = disable_toggle_cell.clone();
    let ds = disabled_slider_cell.clone();

    let scroll = tree.add_scroll_area_fill(12.0, 6.0);
    let content = widget::<UiScrollAreaData>(tree.world_mut(), scroll)
        .map(|d| d.content_entity)
        .unwrap();
    tree.build_ui(content, |ui| {
        ui.heading("Buttons");
        ui.separator();

        ui.label("Standard Button");
        ui.button("Primary Action");

        ui.spacing(4.0);
        ui.label("Colored Buttons");
        let theme = ui.theme().clone();
        ui.button_colored("Success", theme.success_color);
        ui.button_colored("Warning", theme.warning_color);
        ui.button_colored("Danger", theme.error_color);

        ui.spacing(8.0);
        ui.heading("Toggle / Checkbox");
        ui.separator();

        ui.label("Toggle Switch");
        ui.toggle(false);

        ui.spacing(4.0);
        ui.label("Checkbox");
        ui.checkbox("Enable notifications", false);

        ui.spacing(8.0);
        ui.heading("Radio Group");
        ui.separator();
        ui.radio("Option A", 1, 0);
        ui.radio("Option B", 1, 1);
        ui.radio("Option C", 1, 2);

        ui.spacing(8.0);
        ui.heading("Selectable Labels");
        ui.separator();
        ui.selectable_label("Renderer: wgpu", Some(1));
        ui.selectable_label("Audio: rodio", Some(1));
        ui.selectable_label("Physics: rapier", Some(1));

        ui.spacing(8.0);
        ui.heading("Spinner");
        ui.separator();
        ui.spinner();

        ui.spacing(8.0);
        ui.heading("Disabled State");
        ui.separator();
        ui.label("Toggle to disable the slider below:");
        let disable_toggle = ui.toggle(false);
        let disabled_slider = ui.slider(0.0, 100.0, 30.0);
        dt.set(disable_toggle);
        ds.set(disabled_slider);
    });
    tree.pop_parent();

    tree.pop_parent();
    (
        screen,
        fps_text,
        uptime_text,
        entity_count_text,
        pulse_entity,
        disable_toggle_cell.get(),
        disabled_slider_cell.get(),
    )
}

fn build_inputs_screen(
    tree: &mut UiTreeBuilder,
) -> (Entity, Entity, Entity, Entity, Entity, Entity) {
    let screen = build_screen_root(tree, false);
    tree.push_parent(screen);

    let card = build_card(tree);
    tree.push_parent(card);

    let scroll = tree.add_scroll_area_fill(12.0, 6.0);
    let content = widget::<UiScrollAreaData>(tree.world_mut(), scroll)
        .map(|d| d.content_entity)
        .unwrap();

    let progress_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let pc = progress_cell.clone();
    let submit_input_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let si = submit_input_cell.clone();
    let submit_log_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let sl = submit_log_cell.clone();
    let focus_target_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let ft = focus_target_cell.clone();
    let focus_button_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let fb = focus_button_cell.clone();
    tree.build_ui(content, |ui| {
        ui.heading("Slider Family");
        ui.separator();

        ui.label("Slider (linear)");
        ui.slider(0.0, 100.0, 50.0);

        ui.spacing(4.0);
        ui.label("Range Slider");
        ui.range_slider(0.0, 100.0, 25.0, 75.0);

        ui.spacing(4.0);
        ui.label("Drag Value");
        ui.drag_value_configured(DragValueConfig::new(-10.0, 10.0, 0.5).speed(0.01));

        ui.spacing(8.0);
        ui.heading("Text Inputs");
        ui.separator();

        ui.label("Text Input");
        ui.text_input("Type here...");

        ui.spacing(4.0);
        ui.label("Submit detection (press Enter)");
        let submit_input = ui.text_input("Type and press Enter...");
        si.set(submit_input);
        let submit_log = ui
            .tree()
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 24.0)),
            )
            .with_text("Press Enter to submit", 12.0)
            .with_color::<UiBase>(CYAN_DIM)
            .without_pointer_events()
            .done();
        sl.set(submit_log);

        ui.spacing(4.0);
        ui.label("Programmatic Focus");
        let focus_target = ui.text_input("Focus target");
        ft.set(focus_target);
        let focus_button = ui.button("Focus the input above");
        fb.set(focus_button);

        ui.spacing(4.0);
        ui.label("Text Area (multi-line)");
        ui.text_area("Multi-line input...", 4);

        ui.spacing(4.0);
        ui.label("Rich Text Editor");
        ui.rich_text_editor("Rich text content...", 4);

        ui.spacing(8.0);
        ui.heading("Progress");
        ui.separator();
        ui.label("Animated progress bar");
        let progress = ui.progress_bar(0.0);
        pc.set(progress);

        ui.spacing(4.0);
        ui.label("Static progress (75%)");
        ui.progress_bar(0.75);
    });
    tree.pop_parent();
    tree.pop_parent();

    let progress_entity = progress_cell.get();
    (
        screen,
        progress_entity,
        submit_input_cell.get(),
        submit_log_cell.get(),
        focus_target_cell.get(),
        focus_button_cell.get(),
    )
}

fn build_pickers_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity) {
    let screen = build_screen_root(tree, false);
    tree.push_parent(screen);

    let card = build_card(tree);
    tree.push_parent(card);

    let scroll = tree.add_scroll_area_fill(12.0, 6.0);
    let content = widget::<UiScrollAreaData>(tree.world_mut(), scroll)
        .map(|d| d.content_entity)
        .unwrap();

    let palette_btn_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let pb = palette_btn_cell.clone();

    tree.build_ui(content, |ui| {
        ui.heading("Dropdowns");
        ui.separator();

        ui.label("Quality preset");
        ui.dropdown(&["Low", "Medium", "High", "Ultra"], 1);

        ui.spacing(4.0);
        ui.label("Theme picker");
        ui.theme_dropdown();

        ui.spacing(8.0);
        ui.heading("Tab Bar");
        ui.separator();
        ui.tab_bar(&["General", "Audio", "Display", "Controls"], 0);

        ui.spacing(8.0);
        ui.heading("Color Picker");
        ui.separator();
        ui.color_picker(nalgebra_glm::Vec4::new(0.3, 0.5, 0.9, 1.0));

        ui.spacing(8.0);
        ui.heading("Color Wheel");
        ui.separator();
        ui.color_wheel(nalgebra_glm::Vec4::new(0.3, 0.5, 0.9, 1.0));

        ui.spacing(8.0);
        ui.heading("Date Picker");
        ui.separator();
        ui.date_picker(2026, 5, 3);

        ui.spacing(8.0);
        ui.heading("Multi-Select");
        ui.separator();
        ui.multi_select(&["Apple", "Banana", "Cherry", "Date", "Elderberry"]);

        ui.spacing(8.0);
        ui.heading("Menu");
        ui.separator();
        ui.menu("File", &["New", "Open", "Save", "Save As...", "Quit"]);

        ui.spacing(8.0);
        ui.heading("Command Palette");
        ui.separator();
        let palette_btn = ui.button("Open Command Palette (Ctrl+P)");
        pb.set(palette_btn);
    });
    tree.pop_parent();
    tree.pop_parent();
    (screen, palette_btn_cell.get())
}

fn build_containers_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Entity, Entity, Entity) {
    let screen = build_screen_root(tree, false);
    tree.push_parent(screen);

    let confirm_trigger_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let modal_trigger_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let modal_dialog_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let ct = confirm_trigger_cell.clone();
    let mt = modal_trigger_cell.clone();
    let md = modal_dialog_cell.clone();

    let card = build_card(tree);
    tree.push_parent(card);

    let scroll = tree.add_scroll_area_fill(12.0, 6.0);
    let content = widget::<UiScrollAreaData>(tree.world_mut(), scroll)
        .map(|d| d.content_entity)
        .unwrap();

    tree.build_ui(content, |ui| {
        ui.heading("Collapsing Header");
        ui.separator();
        ui.collapsing_header("Click to expand", true, |ui| {
            ui.label("Content nested inside the collapsing header.");
            ui.label("Click the header above to toggle visibility.");
            ui.button("Nested button");
        });

        ui.spacing(8.0);
        ui.heading("Splitter");
        ui.separator();
        let splitter_box = ui
            .tree()
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 80.0)),
            )
            .entity();
        ui.within(splitter_box, |inner| {
            inner.splitter(SplitDirection::Horizontal, 0.5);
        });

        ui.spacing(8.0);
        ui.heading("Tile Container");
        ui.separator();
        let tile_box = ui
            .tree()
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 160.0)),
            )
            .entity();
        let tile_container_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
        let tcc = tile_container_cell.clone();
        ui.within(tile_box, |inner| {
            let tc = inner.tile_container(nalgebra_glm::Vec2::new(420.0, 160.0));
            tcc.set(tc);
        });
        let tile_container = tile_container_cell.get();
        for title in ["Scene", "Inspector", "Console"] {
            let _ = ui_tile_add_pane(ui.world_mut(), tile_container, title);
        }

        ui.spacing(8.0);
        ui.heading("Breadcrumb");
        ui.separator();
        ui.breadcrumb(&["Home", "Library", "Sample Item"]);

        ui.spacing(8.0);
        ui.heading("Scroll Area (fixed size)");
        ui.separator();
        ui.scroll_area(nalgebra_glm::Vec2::new(0.0, 140.0), |ui| {
            for index in 0..30 {
                ui.label(&format!("Scrollable item {index}"));
            }
        });

        ui.spacing(8.0);
        ui.heading("Modal Dialog");
        ui.separator();
        let modal_btn = ui.button("Show Modal Dialog");
        mt.set(modal_btn);

        ui.spacing(4.0);
        ui.label("Confirm Dialog");
        let confirm_btn = ui.button("Show Confirm Dialog");
        ct.set(confirm_btn);

        ui.spacing(8.0);
        ui.heading("Canvas (custom drawing)");
        ui.separator();
        ui.canvas(nalgebra_glm::Vec2::new(280.0, 60.0));
    });
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
    md.set(modal_dialog);

    let confirm_dialog =
        tree.add_confirm_dialog("Confirm Action", "Are you sure you want to proceed?");

    tree.add_floating_panel(
        "gallery_floating",
        "Floating Panel",
        Rect {
            min: nalgebra_glm::Vec2::new(140.0, 220.0),
            max: nalgebra_glm::Vec2::new(380.0, 380.0),
        },
    );

    let modal_trigger = modal_trigger_cell.get();
    let confirm_trigger = confirm_trigger_cell.get();

    tree.pop_parent();
    (
        screen,
        modal_trigger,
        modal_dialog,
        confirm_trigger,
        confirm_dialog,
    )
}

fn build_data_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Vec<Entity>) {
    let screen = build_screen_root(tree, false);
    tree.push_parent(screen);
    let virtual_list_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let text_nodes_cell = std::rc::Rc::new(std::cell::RefCell::new(Vec::<Entity>::new()));

    build_two_column_card(
        tree,
        |ui| {
            ui.heading("Property Grid");
            ui.separator();
            let grid = ui.property_grid(110.0);
            let transform_section = ui.property_section(grid, "Transform");
            for (label, default) in [("X", 0.0_f32), ("Y", 1.0), ("Z", 0.0)] {
                let area = ui.property_row(grid, transform_section, label);
                ui.tree().push_parent(area);
                ui.tree().add_drag_value(-10.0, 10.0, default);
                ui.tree().pop_parent();
            }
            let appearance = ui.property_section(grid, "Appearance");
            let area = ui.property_row(grid, appearance, "Opacity");
            ui.tree().push_parent(area);
            ui.tree().add_drag_value(0.0, 1.0, 0.85);
            ui.tree().pop_parent();
            let area = ui.property_row(grid, appearance, "Hidden");
            ui.tree().push_parent(area);
            ui.tree().add_toggle(false);
            ui.tree().pop_parent();

            ui.spacing(8.0);
            ui.heading("Data Grid");
            ui.separator();
            let data_grid_box = ui
                .tree()
                .add_node()
                .flow_child(
                    Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 320.0)),
                )
                .entity();
            let data_grid_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
            let dgc = data_grid_cell.clone();
            ui.within(data_grid_box, |inner| {
                let grid = inner.data_grid(
                    &[
                        DataGridColumn::new("Name", 120.0),
                        DataGridColumn::new("Type", 90.0),
                        DataGridColumn::new("Value", 90.0),
                    ],
                    10,
                );
                dgc.set(grid);
            });
            let grid = data_grid_cell.get();
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
            ui_data_grid_set_row_count(ui.world_mut(), grid, sample_rows.len());
            for (row, (name, ty, value)) in sample_rows.iter().enumerate() {
                ui_data_grid_set_cell(ui.world_mut(), grid, row, 0, name);
                ui_data_grid_set_cell(ui.world_mut(), grid, row, 1, ty);
                ui_data_grid_set_cell(ui.world_mut(), grid, row, 2, value);
            }

            ui.spacing(8.0);
            ui.heading("Virtual List");
            ui.separator();
            let virtual_list_box = ui
                .tree()
                .add_node()
                .flow_child(
                    Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 220.0)),
                )
                .entity();
            let vl_cell = virtual_list_cell.clone();
            ui.within(virtual_list_box, |inner| {
                let vl = inner.tree().add_virtual_list(28.0, 8);
                vl_cell.set(vl);
            });
            let vl = virtual_list_cell.get();
            ui_virtual_list_set_count(ui.world_mut(), vl, 64);
            let pool_containers: Vec<Entity> = ui
                .world_mut()
                .ui
                .get_ui_virtual_list(vl)
                .map(|d| d.pool_items.iter().map(|p| p.container_entity).collect())
                .unwrap_or_default();
            let mut text_nodes = text_nodes_cell.borrow_mut();
            text_nodes.clear();
            for (pool_index, container) in pool_containers.iter().enumerate() {
                ui.tree().push_parent(*container);
                let text_entity = ui
                    .tree()
                    .add_node()
                    .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 100.0)))
                    .with_text(&format!("Item {pool_index}"), 14.0)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .without_pointer_events()
                    .entity();
                text_nodes.push(text_entity);
                ui.tree().pop_parent();
            }
        },
        |ui| {
            ui.heading("Tree View");
            ui.separator();
            let tree_view_box = ui
                .tree()
                .add_node()
                .flow_child(
                    Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 240.0)),
                )
                .entity();
            ui.tree().push_parent(tree_view_box);
            let tree_view = ui.tree().add_tree_view(false);
            let tv_content = widget::<UiTreeViewData>(ui.world_mut(), tree_view)
                .map(|d| d.content_entity)
                .unwrap();
            let project = ui
                .tree()
                .add_tree_node(tree_view, tv_content, "Project", 0, 0);
            ui_tree_node_set_expanded(ui.world_mut(), project, true);
            let project_children = widget::<UiTreeNodeData>(ui.world_mut(), project)
                .map(|d| d.children_container)
                .unwrap();
            ui.tree()
                .add_tree_node(tree_view, project_children, "Assets", 1, 1);
            ui.tree()
                .add_tree_node(tree_view, project_children, "Scripts", 1, 2);
            let scenes = ui
                .tree()
                .add_tree_node(tree_view, project_children, "Scenes", 1, 3);
            ui_tree_node_set_expanded(ui.world_mut(), scenes, true);
            let scene_children = widget::<UiTreeNodeData>(ui.world_mut(), scenes)
                .map(|d| d.children_container)
                .unwrap();
            ui.tree()
                .add_tree_node(tree_view, scene_children, "main.scn", 2, 4);
            ui.tree()
                .add_tree_node(tree_view, scene_children, "menu.scn", 2, 5);
            ui.tree().pop_parent();

            ui.spacing(8.0);
            ui.heading("Live Status");
            ui.separator();

            ui.tree()
                .add_node()
                .flow_child(
                    Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 80.0)),
                )
                .with_text("Adjust widgets on left to see live values", 12.0)
                .with_color::<UiBase>(CYAN_DIM)
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                .without_pointer_events()
                .done();
        },
    );

    tree.pop_parent();
    let virtual_list = virtual_list_cell.get();
    let text_nodes = text_nodes_cell.borrow().clone();
    (screen, virtual_list, text_nodes)
}

fn build_effects_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Entity, Entity, Entity) {
    let screen = build_screen_root(tree, false);
    tree.push_parent(screen);

    let card = build_card(tree);
    tree.push_parent(card);

    let scroll = tree.add_scroll_area_fill(12.0, 6.0);
    let content = widget::<UiScrollAreaData>(tree.world_mut(), scroll)
        .map(|d| d.content_entity)
        .unwrap();

    let effects_pulse_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let effects_blend_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let effects_progress_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let effects_spinner_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let ept = effects_pulse_cell.clone();
    let ebl = effects_blend_cell.clone();
    let epb = effects_progress_cell.clone();
    let esp = effects_spinner_cell.clone();

    tree.build_ui(content, |ui| {
        ui.heading("Hover & Press Transitions");
        ui.separator();
        ui.label("Hover and press the buttons to see smooth color transitions:");
        for label in ["Smooth Hover", "Smooth Press", "Both Effects"] {
            ui.button(label);
        }

        ui.spacing(8.0);
        ui.heading("Tooltips");
        ui.separator();
        ui.label("Hover the widgets below to see tooltips:");

        let tooltip_btn = ui
            .tree()
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 30.0)),
            )
            .with_rect(4.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(CARD_BORDER)
            .with_color::<UiHover>(DARK_PANEL_HOVER)
            .with_transition::<UiHover>(8.0, 6.0)
            .with_interaction()
            .with_tooltip("This is a tooltip on a custom button")
            .with_cursor_icon(winit::window::CursorIcon::Pointer)
            .with_children(|tree| {
                tree.add_node()
                    .window(
                        Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                        Ab(nalgebra_glm::Vec2::new(200.0, 24.0)),
                        Anchor::Center,
                    )
                    .with_text("Hover for tooltip", 13.0)
                    .with_color::<UiBase>(LIGHT_GRAY)
                    .without_pointer_events();
            })
            .done();
        let _ = tooltip_btn;

        ui.spacing(8.0);
        ui.heading("Toast Notifications");
        ui.separator();
        ui.button("Show Info Toast");
        ui.button("Show Success Toast");
        ui.button("Show Warning Toast");
        ui.button("Show Error Toast");

        ui.spacing(8.0);
        ui.heading("Theme Switching");
        ui.separator();
        ui.label("Use the theme picker in the top bar to cycle through built-in themes.");

        ui.spacing(8.0);
        ui.heading("Live Animation");
        ui.separator();
        ui.label("Pulsing alpha (sine wave):");
        let effects_pulse = ui
            .tree()
            .add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(80.0, 80.0)))
            .with_rect(8.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.0, 1.0, 0.5, 1.0))
            .without_pointer_events()
            .entity();
        ept.set(effects_pulse);

        ui.spacing(6.0);
        ui.label("Cyan-to-magenta blend:");
        let effects_blend = ui
            .tree()
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 14.0)),
            )
            .with_rect(4.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(CYAN)
            .without_pointer_events()
            .entity();
        ebl.set(effects_blend);

        ui.spacing(6.0);
        ui.label("Looping progress bar:");
        let effects_progress = ui.progress_bar(0.0);
        epb.set(effects_progress);

        ui.spacing(6.0);
        ui.label("Rotating swatch:");
        let effects_spinner_holder = ui
            .tree()
            .add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(60.0, 60.0)))
            .without_pointer_events()
            .entity();
        ui.tree().push_parent(effects_spinner_holder);
        let effects_spinner = ui
            .tree()
            .add_node()
            .window(
                Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                Ab(nalgebra_glm::Vec2::new(40.0, 40.0)),
                Anchor::Center,
            )
            .with_rect(6.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(CYAN)
            .without_pointer_events()
            .entity();
        esp.set(effects_spinner);
        ui.tree().pop_parent();

        ui.spacing(8.0);
        ui.heading("Intro Animations");
        ui.separator();
        ui.label("Each screen fades in via with_intro(UiAnimationType::Fade, 0.2)");
        ui.label("Switch screens via the sidebar to see the effect.");

        ui.spacing(8.0);
        ui.heading("SDF Drop Shadows");
        ui.separator();
        ui.label("Shadows are computed in the rect shader as a second SDF pass.");
        ui.spacing(4.0);

        let shadow_demo_row = ui
            .tree()
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 140.0)),
            )
            .flow(FlowDirection::Horizontal, 16.0, 24.0)
            .with_clip()
            .without_pointer_events()
            .entity();
        ui.tree().push_parent(shadow_demo_row);

        ui.tree()
            .add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(140.0, 100.0)))
            .with_rect(8.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.15, 0.15, 0.22, 1.0))
            .with_shadow(
                nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.7),
                nalgebra_glm::Vec2::new(0.0, 6.0),
                12.0,
                0.0,
            )
            .with_children(|tree| {
                tree.add_node()
                    .window(
                        Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                        Ab(nalgebra_glm::Vec2::new(120.0, 24.0)),
                        Anchor::Center,
                    )
                    .with_text("Soft", 14.0)
                    .with_color::<UiBase>(WHITE)
                    .without_pointer_events();
            })
            .done();

        ui.tree()
            .add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(140.0, 100.0)))
            .with_rect(8.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.15, 0.15, 0.22, 1.0))
            .with_shadow(
                nalgebra_glm::Vec4::new(0.0, 0.6, 0.8, 0.6),
                nalgebra_glm::Vec2::new(0.0, 0.0),
                20.0,
                4.0,
            )
            .with_children(|tree| {
                tree.add_node()
                    .window(
                        Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                        Ab(nalgebra_glm::Vec2::new(120.0, 24.0)),
                        Anchor::Center,
                    )
                    .with_text("Glow", 14.0)
                    .with_color::<UiBase>(CYAN)
                    .without_pointer_events();
            })
            .done();

        ui.tree()
            .add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(140.0, 100.0)))
            .with_rect(8.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.15, 0.15, 0.22, 1.0))
            .with_shadow(
                nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.85),
                nalgebra_glm::Vec2::new(8.0, 8.0),
                4.0,
                0.0,
            )
            .with_children(|tree| {
                tree.add_node()
                    .window(
                        Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                        Ab(nalgebra_glm::Vec2::new(120.0, 24.0)),
                        Anchor::Center,
                    )
                    .with_text("Hard", 14.0)
                    .with_color::<UiBase>(WHITE)
                    .without_pointer_events();
            })
            .done();

        ui.tree().pop_parent();
    });
    tree.pop_parent();
    tree.pop_parent();
    (
        screen,
        effects_pulse_cell.get(),
        effects_blend_cell.get(),
        effects_progress_cell.get(),
        effects_spinner_cell.get(),
    )
}

fn build_polish_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Entity, Entity) {
    use nightshade::ecs::ui::components::Spring;
    use nightshade::ecs::ui::state::{UiFocused, UiHover};

    let screen = build_screen_root(tree, false);
    tree.push_parent(screen);

    let progress_slider_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let progress_indet_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let progress_clear_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
    let ps = progress_slider_cell.clone();
    let pi = progress_indet_cell.clone();
    let pc = progress_clear_cell.clone();

    build_two_column_card(
        tree,
        |ui| {
            ui.heading("Hover Lift + Press Compress");
            ui.separator();
            ui.label("Hover any button to see it rise. Click to see it sink.");
            ui.spacing(4.0);
            ui.button("Standard Button");
            ui.button("Another Button");
            let theme = ui.theme().clone();
            ui.button_colored("Success", theme.success_color);
            ui.button_colored("Warning", theme.warning_color);
            ui.button_colored("Danger", theme.error_color);

            ui.spacing(12.0);
            ui.heading("Focus Glow");
            ui.separator();
            ui.label("Tab through these to see focus rings:");
            ui.spacing(4.0);
            ui.text_input("type to focus");
            ui.button("Focusable Button A");
            ui.button("Focusable Button B");

            ui.spacing(12.0);
            ui.heading("Toggle + Slider Scale");
            ui.separator();
            ui.label("Toggle scales up on hover, compresses on press:");
            ui.spacing(4.0);
            ui.toggle(false);
            ui.spacing(6.0);
            ui.label("Slider track gains focus ring when active:");
            ui.spacing(4.0);
            ui.slider(0.0, 100.0, 50.0);
        },
        |ui| {
            ui.heading("Spring Physics Tuning");
            ui.separator();
            ui.label("Each button below has the same hover-lift but a different spring config.");
            ui.spacing(6.0);

            ui.label("Soft (low stiffness, low damping)");
            spring_demo_button(
                ui,
                "Soft Spring",
                Spring {
                    stiffness: 80.0,
                    damping: 12.0,
                    mass: 1.0,
                },
            );
            ui.spacing(4.0);

            ui.label("Default (Framer baseline)");
            spring_demo_button(ui, "Default Spring", Spring::default());
            ui.spacing(4.0);

            ui.label("Stiff (high stiffness, well damped)");
            spring_demo_button(
                ui,
                "Stiff Spring",
                Spring {
                    stiffness: 400.0,
                    damping: 40.0,
                    mass: 1.0,
                },
            );
            ui.spacing(4.0);

            ui.label("Bouncy (high stiffness, low damping)");
            spring_demo_button(
                ui,
                "Bouncy Spring",
                Spring {
                    stiffness: 350.0,
                    damping: 14.0,
                    mass: 1.0,
                },
            );

            ui.spacing(12.0);
            ui.heading("Linear vs Spring");
            ui.separator();
            ui.label("Linear easing (stiff, mechanical):");
            linear_demo_button(ui, "Linear Hover");
            ui.spacing(4.0);
            ui.label("Spring (settles with overshoot and damping):");
            spring_demo_button(ui, "Spring Hover", Spring::default());

            ui.spacing(12.0);
            ui.heading("Focus Radius Grow");
            ui.separator();
            ui.label("Click in this input. The corner radius grows on focus.");
            ui.spacing(4.0);
            let focus_input = ui
                .tree()
                .add_node()
                .flow_child(
                    Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 32.0)),
                )
                .with_rect(4.0, 1.0, CARD_BORDER)
                .with_theme_color::<UiBase>(ThemeColor::InputBackground)
                .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                .with_theme_color::<UiFocused>(ThemeColor::InputBackgroundFocused)
                .with_theme_shadow_role::<UiFocused>(
                    nightshade::ecs::ui::components::ThemeShadow::FocusGlow,
                )
                .with_state_radius::<UiFocused>(12.0)
                .with_transition::<UiHover>(8.0, 6.0)
                .with_transition::<UiFocused>(8.0, 6.0)
                .with_interaction()
                .with_cursor_icon(winit::window::CursorIcon::Text)
                .with_children(|tree| {
                    tree.add_node()
                        .window(
                            Ab(nalgebra_glm::Vec2::new(12.0, 0.0)),
                            Ab(nalgebra_glm::Vec2::new(0.0, 32.0))
                                + Rl(nalgebra_glm::Vec2::new(100.0, 0.0)),
                            Anchor::CenterLeft,
                        )
                        .with_text("Click to focus", 14.0)
                        .with_color::<UiBase>(LIGHT_GRAY)
                        .without_pointer_events();
                })
                .done();
            let _ = focus_input;

            ui.spacing(12.0);
            ui.heading("Top-of-Window Progress");
            ui.separator();
            ui.label("Drag the slider to set determinate progress, or use the buttons.");
            ui.spacing(4.0);
            let slider_entity = ui.slider(0.0, 1.0, 0.0);
            ps.set(slider_entity);
            ui.spacing(6.0);
            let indet_entity = ui.button("Set Indeterminate");
            pi.set(indet_entity);
            let clear_entity = ui.button("Clear Progress Bar");
            pc.set(clear_entity);
        },
    );

    tree.pop_parent();
    (
        screen,
        progress_slider_cell.get(),
        progress_indet_cell.get(),
        progress_clear_cell.get(),
    )
}

fn spring_demo_button(
    ui: &mut nightshade::ecs::ui::scope::Ui<'_, '_>,
    label: &str,
    spring: nightshade::ecs::ui::components::Spring,
) -> Entity {
    use nalgebra_glm::Vec2;
    use nightshade::ecs::ui::state::{UiHover, UiPressed};

    let theme = ui.theme().clone();
    ui.tree()
        .add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 32.0)),
        )
        .with_rect(theme.corner_radius, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<nightshade::ecs::ui::state::UiBase>(ThemeColor::Background)
        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
        .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
        .with_theme_effect_role(nightshade::ecs::ui::components::ThemeEffect::ButtonEffect)
        .with_theme_shadow_role::<nightshade::ecs::ui::state::UiBase>(
            nightshade::ecs::ui::components::ThemeShadow::ButtonShadow,
        )
        .with_theme_shadow_role::<UiHover>(
            nightshade::ecs::ui::components::ThemeShadow::ButtonShadowHover,
        )
        .with_theme_shadow_role::<UiPressed>(
            nightshade::ecs::ui::components::ThemeShadow::ButtonShadowPressed,
        )
        .with_state_offset::<UiHover>(Vec2::new(0.0, -3.0))
        .with_state_offset::<UiPressed>(Vec2::new(0.0, 2.0))
        .with_spring_transition::<UiHover>(spring)
        .with_spring_transition::<UiPressed>(spring)
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                    Ab(nalgebra_glm::Vec2::new(200.0, 24.0)),
                    Anchor::Center,
                )
                .with_text(label, 14.0)
                .with_theme_color::<nightshade::ecs::ui::state::UiBase>(ThemeColor::Text)
                .without_pointer_events();
        })
        .done()
}

fn linear_demo_button(ui: &mut nightshade::ecs::ui::scope::Ui<'_, '_>, label: &str) -> Entity {
    use nalgebra_glm::Vec2;
    use nightshade::ecs::tween::easing::EasingFunction;
    use nightshade::ecs::ui::state::{UiHover, UiPressed};

    let theme = ui.theme().clone();
    ui.tree()
        .add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 32.0)),
        )
        .with_rect(theme.corner_radius, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<nightshade::ecs::ui::state::UiBase>(ThemeColor::Background)
        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
        .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
        .with_theme_effect_role(nightshade::ecs::ui::components::ThemeEffect::ButtonEffect)
        .with_theme_shadow_role::<nightshade::ecs::ui::state::UiBase>(
            nightshade::ecs::ui::components::ThemeShadow::ButtonShadow,
        )
        .with_theme_shadow_role::<UiHover>(
            nightshade::ecs::ui::components::ThemeShadow::ButtonShadowHover,
        )
        .with_state_offset::<UiHover>(Vec2::new(0.0, -3.0))
        .with_state_offset::<UiPressed>(Vec2::new(0.0, 2.0))
        .with_eased_transition::<UiHover>(8.0, 6.0, EasingFunction::Linear)
        .with_eased_transition::<UiPressed>(12.0, 8.0, EasingFunction::Linear)
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                    Ab(nalgebra_glm::Vec2::new(200.0, 24.0)),
                    Anchor::Center,
                )
                .with_text(label, 14.0)
                .with_theme_color::<nightshade::ecs::ui::state::UiBase>(ThemeColor::Text)
                .without_pointer_events();
        })
        .done()
}
