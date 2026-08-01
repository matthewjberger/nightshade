use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_platform::winit;
use nightshade_text::prelude::*;
use nightshade_ui::components::{ThemeEffect, ThemeShadow};
use nightshade_ui::prelude::*;

use crate::resources::GalleryResources;

const SIDEBAR_WIDTH: f32 = 220.0;
const SIDEBAR_COMPACT_WIDTH: f32 = 64.0;
const TOP_BAR_HEIGHT: f32 = 56.0;
const PAGE_PADDING: f32 = 32.0;
const BLOCK_PADDING: f32 = 20.0;
const BLOCK_GAP: f32 = 16.0;

struct NavItem {
    label: &'static str,
    icon: IconGlyph,
    description: &'static str,
}

pub fn requested_screen(wanted: Option<&str>) -> Option<usize> {
    let wanted = wanted?;
    NAV_ITEMS
        .iter()
        .position(|item| item.label.eq_ignore_ascii_case(wanted))
}

const NAV_ITEMS: [NavItem; 9] = [
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
        icon: icons::material::GRID_ON,
        description: "Scroll areas, splitters, modals, breadcrumbs, panels",
    },
    NavItem {
        label: "Tiles",
        icon: icons::material::VIEW_QUILT,
        description: "Dockable tile container with drag-to-rearrange tabs",
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
    NavItem {
        label: "Media",
        icon: icons::material::PHOTO,
        description: "Rich text spans, runtime images, context menus",
    },
];

pub fn build_gallery_ui(
    mut gallery: ResMut<GalleryResources>,
    mut build: nightshade_ui::commands::builder::UiBuild<'_>,
) {
    let mut tree = UiTreeBuilder::new(build.context());

    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .fg(ThemeColor::Background)
        .entity();

    let mut screen_roots = Vec::new();
    let mut fps_text = Entity::default();
    let mut uptime_text = Entity::default();
    let mut nav_buttons = Vec::new();
    let mut nav_indicators = Vec::new();
    let mut nav_labels = Vec::new();
    let mut nav_slider = Entity::default();
    let mut nav_y_offsets = Vec::new();
    let mut pulse_entity = Entity::default();
    let mut disable_toggle = Entity::default();
    let mut disabled_slider = Entity::default();
    let mut progress_entity = Entity::default();
    let mut submit_input = Entity::default();
    let mut submit_log = Entity::default();
    let mut focus_target = Entity::default();
    let mut focus_button = Entity::default();
    let mut palette_btn = Entity::default();
    let mut modal_trigger = Entity::default();
    let mut modal_dialog = Entity::default();
    let mut confirm_trigger = Entity::default();
    let mut confirm_dialog = Entity::default();
    let mut canvas_demo = Entity::default();
    let mut virtual_list = Entity::default();
    let mut virtual_list_text_nodes = Vec::new();
    let mut effects_pulse = Entity::default();
    let mut effects_blend = Entity::default();
    let mut effects_progress = Entity::default();
    let mut effects_spinner = Entity::default();
    let mut progress_slider = Entity::default();
    let mut progress_indeterminate_btn = Entity::default();
    let mut progress_clear_btn = Entity::default();
    let mut palette_entity = Entity::default();
    let mut context_menu_target = Entity::default();
    let mut context_menu = Entity::default();
    let mut simple_menu_target = Entity::default();
    let mut simple_menu = Entity::default();
    let mut docked_panels: Vec<Entity> = Vec::new();

    tree.in_parent(root, |tree| {
        (fps_text, uptime_text) = build_top_bar(tree);
        let sidebar = build_sidebar(tree);
        nav_buttons = sidebar.nav_buttons;
        nav_indicators = sidebar.nav_indicators;
        nav_labels = sidebar.nav_labels;
        nav_slider = sidebar.slider;
        nav_y_offsets = sidebar.nav_y_offsets;

        let content_area = tree
            .add_node()
            .boundary(
                Ab(vec2(SIDEBAR_WIDTH, TOP_BAR_HEIGHT)),
                Rl(vec2(100.0, 100.0)),
            )
            .responsive_position_1_at(
                UiBreakpoint::Compact,
                Ab(vec2(SIDEBAR_COMPACT_WIDTH, TOP_BAR_HEIGHT)).into(),
            )
            .fg(ThemeColor::Background)
            .entity();

        tree.in_parent(content_area, |tree| {
            let buttons = build_buttons_screen(tree);
            screen_roots.push(buttons.0);
            pulse_entity = buttons.1;
            disable_toggle = buttons.2;
            disabled_slider = buttons.3;
            let inputs = build_inputs_screen(tree);
            screen_roots.push(inputs.0);
            progress_entity = inputs.1;
            submit_input = inputs.2;
            submit_log = inputs.3;
            focus_target = inputs.4;
            focus_button = inputs.5;
            let pickers = build_pickers_screen(tree);
            screen_roots.push(pickers.0);
            palette_btn = pickers.1;
            let containers = build_containers_screen(tree);
            screen_roots.push(containers.0);
            modal_trigger = containers.1;
            modal_dialog = containers.2;
            confirm_trigger = containers.3;
            confirm_dialog = containers.4;
            canvas_demo = containers.5;
            screen_roots.push(build_tiles_screen(tree));
            let data = build_data_screen(tree);
            screen_roots.push(data.0);
            virtual_list = data.1;
            virtual_list_text_nodes = data.2;
            let effects = build_effects_screen(tree);
            screen_roots.push(effects.0);
            effects_pulse = effects.1;
            effects_blend = effects.2;
            effects_progress = effects.3;
            effects_spinner = effects.4;
            let polish = build_polish_screen(tree);
            screen_roots.push(polish.0);
            progress_slider = polish.1;
            progress_indeterminate_btn = polish.2;
            progress_clear_btn = polish.3;
            let media = build_media_screen(tree);
            screen_roots.push(media.0);
            context_menu_target = media.1;
            simple_menu_target = media.2;
        });

        docked_panels = vec![
            tree.add_floating_panel(
                "gallery_dock_float",
                "Floating",
                Rect {
                    min: vec2(320.0, 180.0),
                    max: vec2(620.0, 400.0),
                },
            ),
            tree.add_docked_panel_top("gallery_dock_top", "Toolbar", 44.0),
            tree.add_docked_panel_bottom("gallery_dock_bottom", "Console", 120.0),
            tree.add_docked_panel_left("gallery_dock_left", "Outline", 200.0),
            tree.add_docked_panel_right("gallery_dock_right", "Inspector", 240.0),
        ];
        for panel in docked_panels.clone() {
            tree.in_parent(panel, |tree| {
                tree.add_node()
                    .fill_width()
                    .auto_size(AutoSizeMode::Height)
                    .with_text("Drag the header to undock. Drag an edge to resize.", 12.0)
                    .with_text_wrap()
                    .fg(ThemeColor::TextDisabled)
                    .entity();
            });
        }

        context_menu = tree.add_context_menu_from_builder(
            ContextMenuBuilder::new()
                .item_tagged("Cut", "Ctrl+X", 1)
                .item_tagged("Copy", "Ctrl+C", 2)
                .item_tagged("Paste", "Ctrl+V", 3)
                .separator()
                .checkable_tagged("Snap to grid", "", true, 4)
                .checkable("Show rulers", "", false)
                .widget_row("Opacity")
                .submenu("Insert", |menu| {
                    menu.item_tagged("Text", "", 5).item_tagged("Image", "", 6)
                }),
        );
        simple_menu = tree.add_context_menu(&[("Reload", Some("F5")), ("", None), ("About", None)]);

        palette_entity = tree.add_command_palette(8);
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
            ui_command_palette_register(
                &mut tree.context_reborrow(),
                palette_entity,
                label,
                shortcut,
                category,
            );
        }
    });

    for (entity, name) in [
        (fps_text, "fps_text"),
        (uptime_text, "uptime_text"),
        (progress_entity, "progress_entity"),
        (pulse_entity, "pulse_entity"),
        (effects_pulse, "effects_pulse"),
        (effects_blend, "effects_blend"),
        (effects_progress, "effects_progress"),
        (effects_spinner, "effects_spinner"),
        (modal_trigger, "modal_trigger"),
        (modal_dialog, "modal_dialog"),
        (confirm_trigger, "confirm_trigger"),
        (confirm_dialog, "confirm_dialog"),
        (submit_input, "submit_input"),
        (submit_log, "submit_log"),
        (focus_target, "focus_target"),
        (focus_button, "focus_button"),
        (canvas_demo, "canvas_demo"),
        (palette_btn, "palette_btn"),
        (palette_entity, "palette_entity"),
        (disable_toggle, "disable_toggle"),
        (disabled_slider, "disabled_slider"),
        (virtual_list, "virtual_list"),
        (nav_slider, "nav_slider"),
        (progress_slider, "progress_slider"),
        (progress_indeterminate_btn, "progress_indeterminate_btn"),
        (progress_clear_btn, "progress_clear_btn"),
        (context_menu_target, "context_menu_target"),
        (context_menu, "context_menu"),
        (simple_menu_target, "simple_menu_target"),
        (simple_menu, "simple_menu"),
    ] {
        tree.accessibility_mut()
            .named_entities
            .insert(name.to_string(), entity);
    }

    let state = &mut gallery.state;
    state.nav_buttons = nav_buttons.clone();
    state.nav_indicators = nav_indicators.clone();
    state.nav_labels = nav_labels.clone();
    for panel in &docked_panels {
        let (dirty, commands) = tree.dirty_and_commands();
        ui_set_visible(dirty, commands, *panel, false);
    }
    state.docked_panels = docked_panels;
    state.screen_roots = screen_roots;
    state.virtual_list_text_nodes = virtual_list_text_nodes;
    state.nav_y_offsets = nav_y_offsets.clone();
    state.nav_slider_y = nav_y_offsets.first().copied().unwrap_or(0.0);
    state.nav_slider_target_y = state.nav_slider_y;

    tree.finish();
}

pub fn poll_gallery(
    mut gallery: ResMut<GalleryResources>,
    input: Res<Input>,
    time: Res<Time>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    storage: &mut nightshade::ecs::Storage,
) {
    let gallery = &mut *gallery;
    let nav_buttons = gallery.state.nav_buttons.clone();
    let nav_labels = gallery.state.nav_labels.clone();
    let screen_roots = gallery.state.screen_roots.clone();
    let docked_panels = gallery.state.docked_panels.clone();
    let modal_trigger = ui_named_entity(build.accessibility(), "modal_trigger");
    let modal_dialog = ui_named_entity(build.accessibility(), "modal_dialog");
    let confirm_trigger = ui_named_entity(build.accessibility(), "confirm_trigger");
    let confirm_dialog = ui_named_entity(build.accessibility(), "confirm_dialog");
    let submit_input = ui_named_entity(build.accessibility(), "submit_input");
    let focus_target = ui_named_entity(build.accessibility(), "focus_target");
    let focus_button = ui_named_entity(build.accessibility(), "focus_button");
    let palette_btn = ui_named_entity(build.accessibility(), "palette_btn");
    let palette_entity = ui_named_entity(build.accessibility(), "palette_entity");
    let disable_toggle = ui_named_entity(build.accessibility(), "disable_toggle");
    let disabled_slider = ui_named_entity(build.accessibility(), "disabled_slider");
    let progress_slider = ui_named_entity(build.accessibility(), "progress_slider");
    let progress_indeterminate_btn =
        ui_named_entity(build.accessibility(), "progress_indeterminate_btn");
    let progress_clear_btn = ui_named_entity(build.accessibility(), "progress_clear_btn");
    let context_menu_target = ui_named_entity(build.accessibility(), "context_menu_target");
    let context_menu = ui_named_entity(build.accessibility(), "context_menu");
    let simple_menu_target = ui_named_entity(build.accessibility(), "simple_menu_target");
    let simple_menu = ui_named_entity(build.accessibility(), "simple_menu");

    let mut new_active: Option<usize> = gallery.state.pending_screen.take();
    let mut show_modal: Option<Entity> = None;
    let mut show_confirm: Option<Entity> = None;
    let mut toast: Option<(&'static str, ToastSeverity)> = None;
    let mut submit_count_increment = 0u32;
    let mut focus_request: Option<Entity> = None;
    let mut palette_request: Option<Entity> = None;
    let mut button_toast: Option<(String, ToastSeverity)> = None;
    let mut progress_action: Option<ProgressAction> = None;
    let mut context_menu_request: Option<(Entity, Vec2)> = None;

    let events: Vec<UiEvent> = ui_events(build.frame()).to_vec();
    for event in &events {
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
                if !consumed && let Some(data) = storage.get::<UiButtonData>(*entity) {
                    let text_slot = data.text_slot;
                    let label = text_slot
                        .and_then(|slot| {
                            build
                                .text_state()
                                .cache
                                .get_text(slot)
                                .map(|s| s.to_string())
                        })
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
            UiEvent::ContextMenuItemClicked {
                entity,
                item_index,
                tag,
            } if Some(*entity) == context_menu || Some(*entity) == simple_menu => {
                let label = if *tag == 0 {
                    format!("Menu item {item_index}")
                } else {
                    format!("Menu tag {tag}")
                };
                button_toast = Some((label, ToastSeverity::Info));
            }
            _ => {}
        }
    }

    for (target, menu) in [
        (context_menu_target, context_menu),
        (simple_menu_target, simple_menu),
    ] {
        if let Some(target) = target
            && let Some(menu) = menu
            && storage
                .get::<UiNodeInteraction>(target)
                .is_some_and(|interaction| interaction.right_clicked)
        {
            context_menu_request = Some((menu, input.mouse.position));
        }
    }

    let visible_start_of = |entity: Entity| {
        storage
            .get::<nightshade_ui::components::UiVirtualListData>(entity)
            .map(|data| data.visible_start)
            .unwrap_or(0)
    };
    let delta_time = time.delta_time.clamp(0.0, 0.05);
    let toggle_disabled = disable_toggle
        .and_then(|toggle| storage.get::<UiToggleData>(toggle))
        .map(|data| data.value)
        .unwrap_or(false);
    let visible_starts: Vec<(Entity, usize)> = storage
        .query_ref::<(&nightshade_ui::components::UiVirtualListData,)>()
        .iter()
        .map(|(entity, (data,))| (entity, data.visible_start))
        .collect();
    let _ = visible_start_of;

    {
        let mut context = build.context();
        let visible_start_of = |entity: Entity| {
            visible_starts
                .iter()
                .find(|(candidate, _)| *candidate == entity)
                .map(|(_, start)| *start)
                .unwrap_or(0)
        };
        if let Some((menu, position)) = context_menu_request {
            ui_open_context_menu(&mut context, storage, menu, position);
        }

        if let Some(index) = new_active {
            let prev = gallery.state.active_screen;
            if index != prev {
                ui_set_visible(context.dirty, context.commands, screen_roots[prev], false);
                update_nav_label_color(&mut context, nav_labels.get(prev).copied(), false);
                ui_set_visible(context.dirty, context.commands, screen_roots[index], true);
                update_nav_label_color(&mut context, nav_labels.get(index).copied(), true);
                let docked_visible = NAV_ITEMS
                    .get(index)
                    .is_some_and(|item| item.label == "Tiles");
                for panel in &docked_panels {
                    ui_set_visible(context.dirty, context.commands, *panel, docked_visible);
                }
                gallery.state.active_screen = index;
                if let Some(target_y) = gallery.state.nav_y_offsets.get(index).copied() {
                    gallery.state.nav_slider_target_y = target_y;
                }
            }
        }

        if let Some(action) = progress_action {
            context.commands.queue(move |_, resources| {
                let bar = resources.res_mut::<TopProgressBar>();
                match action {
                    ProgressAction::Set(value) => bar.set(value),
                    ProgressAction::Indeterminate => bar.set_indeterminate(),
                    ProgressAction::Clear => bar.clear(),
                }
            });
        }

        let slider_entity = ui_named_entity(context.accessibility, "nav_slider");
        let mut slider_y = gallery.state.nav_slider_y;
        let slider_target = gallery.state.nav_slider_target_y;
        let stiffness = 220.0_f32;
        let displacement = slider_y - slider_target;
        let damping_factor = (-stiffness * delta_time * 0.5).exp();
        slider_y = slider_target + displacement * damping_factor;
        if (slider_y - slider_target).abs() < 0.05 {
            slider_y = slider_target;
        }
        gallery.state.nav_slider_y = slider_y;
        if let Some(slider) = slider_entity {
            update_slider_y(&mut context, slider, slider_y);
        }

        if let Some(target) = focus_request {
            ui_focus(context.interaction, target);
        }
        let ctrl_p = input.keyboard.just_pressed(winit::keyboard::KeyCode::KeyP)
            && (input
                .keyboard
                .is_key_pressed(winit::keyboard::KeyCode::ControlLeft)
                || input
                    .keyboard
                    .is_key_pressed(winit::keyboard::KeyCode::ControlRight));
        if ctrl_p && let Some(palette) = palette_entity {
            ui_show_command_palette(&mut context, storage, palette);
        }
        if let Some(palette) = palette_request {
            ui_show_command_palette(&mut context, storage, palette);
        }
        if let (Some(_toggle), Some(slider)) = (disable_toggle, disabled_slider) {
            ui_set_disabled(context.commands, slider, toggle_disabled);
        }
        if submit_count_increment > 0 {
            gallery.state.submit_count += submit_count_increment;
            if let Some(log_entity) = ui_named_entity(context.accessibility, "submit_log") {
                let count = gallery.state.submit_count;
                let message = format!("Submitted {count} time(s)");
                ui_set_text(context.commands, log_entity, &message);
            }
        }
        if let Some(dialog) = show_modal {
            ui_show_modal(&mut context, storage, dialog);
        }
        if let Some(dialog) = show_confirm {
            ui_show_modal(&mut context, storage, dialog);
        }
        if let Some((message, severity)) = toast {
            ui_show_toast(&mut context, message, severity, 2.0);
        }
        if let Some((message, severity)) = button_toast {
            ui_show_toast(&mut context, &message, severity, 2.0);
        }

        if let Some(virtual_list) = ui_named_entity(context.accessibility, "virtual_list") {
            let visible_start = visible_start_of(virtual_list);
            for (pool_index, text_entity) in gallery
                .state
                .virtual_list_text_nodes
                .clone()
                .into_iter()
                .enumerate()
            {
                let item_index = visible_start + pool_index;
                ui_set_text(context.commands, text_entity, &format!("Item {item_index}"));
            }
        }
    }
}

enum ProgressAction {
    Set(f32),
    Indeterminate,
    Clear,
}

fn update_nav_label_color(
    context: &mut nightshade_ui::commands::builder::UiBuildContext<'_, '_>,
    label: Option<Entity>,
    active: bool,
) {
    let Some(label) = label else { return };
    let theme = context.theme.active_theme().clone();
    let color = if active {
        theme.text_color
    } else {
        theme.text_color_disabled
    };
    context.commands.queue(move |storage, _| {
        if let Some(node_color) = storage.get_mut::<nightshade_ui::components::UiNodeColor>(label) {
            node_color.colors[UiBase::INDEX] = Some(color);
        }
    });
}

fn update_slider_y(
    context: &mut nightshade_ui::commands::builder::UiBuildContext<'_, '_>,
    slider: Entity,
    y: f32,
) {
    use nightshade_ui::components::layout_types::{UiLayoutType, WindowLayout};
    use nightshade_ui::data::units::{Ab, UiValue};
    context.commands.queue(move |storage, _| {
        let Some(node) = storage.get_mut::<nightshade_ui::components::UiLayoutNode>(slider) else {
            return;
        };
        let Some(layout) = node.base_layout.as_mut() else {
            return;
        };
        let UiLayoutType::Window(WindowLayout { position, .. }) = layout else {
            return;
        };
        let new_position: UiValue<nalgebra_glm::Vec2> = Ab(vec2(0.0, y)).into();
        *position = new_position;
    });
    nightshade_ui::prelude::ui_mark_layout_dirty(context.dirty);
}

fn build_top_bar(tree: &mut UiTreeBuilder) -> (Entity, Entity) {
    let theme = tree.active_theme().clone();
    let bar = tree
        .add_node()
        .boundary(
            Rl(vec2(0.0, 0.0)),
            Ab(vec2(0.0, TOP_BAR_HEIGHT)) + Rl(vec2(100.0, 0.0)),
        )
        .with_rect(0.0, 0.0, theme.border_color)
        .fg(ThemeColor::Panel)
        .with_depth(UiDepthMode::Set(10.0))
        .entity();

    let mut fps_text = Entity::default();
    let mut uptime_text = Entity::default();
    tree.in_parent(bar, |tree| {
        tree.add_node()
            .boundary(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -1.0)),
                Rl(vec2(100.0, 100.0)),
            )
            .rect(0.0)
            .fg(ThemeColor::Border);

        let bar_center_y = TOP_BAR_HEIGHT * 0.5;
        tree.add_node()
            .window(
                Ab(vec2(PAGE_PADDING, bar_center_y)),
                Ab(vec2(220.0, 24.0)),
                Anchor::CenterLeft,
            )
            .with_text("Nightshade UI", 20.0)
            .text_left()
            .fg(ThemeColor::Text);

        tree.add_node()
            .window(
                Ab(vec2(PAGE_PADDING + 160.0, bar_center_y)),
                Ab(vec2(220.0, 20.0)),
                Anchor::CenterLeft,
            )
            .with_text("Component Gallery", 14.0)
            .text_left()
            .fg(ThemeColor::TextDisabled);

        let picker_width = 200.0_f32;
        let stats_width = 140.0_f32;
        let gap_to_picker = 32.0_f32;
        let stats_x = -(PAGE_PADDING + picker_width + gap_to_picker + stats_width);
        let stats_holder = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(stats_x, bar_center_y)),
                Ab(vec2(stats_width, 30.0)),
                Anchor::CenterLeft,
            )
            .flow_horizontal()
            .padding(0.0)
            .gap(16.0)
            .align_main(FlowAlignment::End)
            .align_cross(FlowAlignment::Center)
            .with_clip()
            .entity();
        tree.in_parent(stats_holder, |tree| {
            fps_text = tree
                .add_node()
                .size((70.0).px(), (18.0).px())
                .with_text("0 fps", 13.0)
                .text_right()
                .fg(ThemeColor::TextDisabled)
                .entity();
            uptime_text = tree
                .add_node()
                .size((50.0).px(), (18.0).px())
                .with_text("0:00", 13.0)
                .text_right()
                .fg(ThemeColor::TextDisabled)
                .entity();
        });

        let picker_holder = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-(PAGE_PADDING + 200.0), bar_center_y)),
                Ab(vec2(200.0, 32.0)),
                Anchor::CenterLeft,
            )
            .entity();
        tree.in_parent(picker_holder, |tree| {
            tree.add_theme_dropdown();
        });
    });

    (fps_text, uptime_text)
}

struct SidebarBuild {
    nav_buttons: Vec<Entity>,
    nav_indicators: Vec<Entity>,
    nav_labels: Vec<Entity>,
    slider: Entity,
    nav_y_offsets: Vec<f32>,
}

fn build_sidebar(tree: &mut UiTreeBuilder) -> SidebarBuild {
    let theme = tree.active_theme().clone();
    let sidebar = tree
        .add_node()
        .boundary(
            Ab(vec2(0.0, TOP_BAR_HEIGHT)),
            Ab(vec2(SIDEBAR_WIDTH, 0.0)) + Rl(vec2(0.0, 100.0)),
        )
        .responsive_position_2_at(
            UiBreakpoint::Compact,
            Ab(vec2(SIDEBAR_COMPACT_WIDTH, 0.0)) + Rl(vec2(0.0, 100.0)),
        )
        .with_rect(0.0, 0.0, theme.border_color)
        .fg(ThemeColor::Panel)
        .entity();

    let mut nav_buttons = Vec::new();
    let mut nav_indicators = Vec::new();
    let mut nav_labels = Vec::new();
    let mut nav_y_offsets = Vec::new();
    let mut slider = Entity::default();

    tree.in_parent(sidebar, |tree| {
        tree.add_node()
            .boundary(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-1.0, 0.0)),
                Rl(vec2(100.0, 100.0)),
            )
            .rect(0.0)
            .fg(ThemeColor::Border);

        let item_height = 36.0_f32;
        let item_gap = 2.0_f32;
        let top_padding = 16.0_f32;

        slider = tree
            .add_node()
            .window(
                Ab(vec2(0.0, top_padding)),
                Ab(vec2(3.0, item_height)),
                Anchor::TopLeft,
            )
            .rect(0.0)
            .fg(ThemeColor::Accent)
            .entity();

        for (index, item) in NAV_ITEMS.iter().enumerate() {
            let y = top_padding + index as f32 * (item_height + item_gap);
            nav_y_offsets.push(y);

            let placeholder = tree
                .add_node()
                .window(
                    Ab(vec2(0.0, y)),
                    Ab(vec2(3.0, item_height)),
                    Anchor::TopLeft,
                )
                .entity();
            nav_indicators.push(placeholder);

            let button = tree
                .add_node()
                .window(
                    Ab(vec2(8.0, y)),
                    Ab(vec2(SIDEBAR_WIDTH - 16.0, item_height)),
                    Anchor::TopLeft,
                )
                .responsive_size_at(
                    UiBreakpoint::Compact,
                    Ab(vec2(SIDEBAR_COMPACT_WIDTH - 16.0, item_height)).into(),
                )
                .rect(6.0)
                .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.0))
                .fg_hover(ThemeColor::BackgroundHover)
                .fg_pressed(ThemeColor::BackgroundActive)
                .with_transition::<UiHover>(12.0, 6.0)
                .with_transition::<UiPressed>(20.0, 10.0)
                .with_interaction()
                .with_tooltip(item.description)
                .with_cursor_icon(winit::window::CursorIcon::Pointer)
                .entity();

            let mut label = Entity::default();
            tree.in_parent(button, |tree| {
                tree.add_node()
                    .window(
                        Ab(vec2(12.0, 0.0)) + Rl(vec2(0.0, 50.0)),
                        Ab(vec2(18.0, 18.0)),
                        Anchor::CenterLeft,
                    )
                    .with_icon(item.icon, 17.0)
                    .color_raw::<UiBase>(if index == 0 {
                        theme.text_color
                    } else {
                        theme.text_color_disabled
                    })
                    .entity();

                label = tree
                    .add_node()
                    .window(
                        Ab(vec2(40.0, 0.0)) + Rl(vec2(0.0, 50.0)),
                        Rl(vec2(100.0, 0.0)) + Ab(vec2(-52.0, 22.0)),
                        Anchor::CenterLeft,
                    )
                    .responsive_visible_at(UiBreakpoint::Compact, false)
                    .with_text(item.label, 15.0)
                    .text_left()
                    .color_raw::<UiBase>(if index == 0 {
                        theme.text_color
                    } else {
                        theme.text_color_disabled
                    })
                    .entity();
            });

            nav_buttons.push(button);
            nav_labels.push(label);
        }
    });

    SidebarBuild {
        nav_buttons,
        nav_indicators,
        nav_labels,
        slider,
        nav_y_offsets,
    }
}

fn build_screen_root(tree: &mut UiTreeBuilder, visible: bool) -> Entity {
    tree.add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_visible(visible)
        .with_intro(UiAnimationType::Fade, 0.18)
        .entity()
}

fn build_page_header(tree: &mut UiTreeBuilder, title: &str, subtitle: &str) {
    tree.add_node()
        .size(100.pct(), (32.0).px())
        .with_text(title, 26.0)
        .text_left()
        .fg(ThemeColor::Text)
        .entity();
    tree.add_node()
        .size(100.pct(), (22.0).px())
        .with_text(subtitle, 14.0)
        .with_text_wrap()
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
        .fg(ThemeColor::TextDisabled)
        .entity();
    tree.add_node().size(100.pct(), (BLOCK_GAP).px()).entity();
}

pub(crate) const RECOLOR_CHOICES: [(&str, Vec4); 3] = [
    ("Green", Vec4::new(0.45, 0.92, 0.50, 1.0)),
    ("Amber", Vec4::new(1.0, 0.68, 0.25, 1.0)),
    ("Red", Vec4::new(1.0, 0.36, 0.36, 1.0)),
];

pub(crate) fn recolor_button_name(label: &str) -> String {
    format!("recolor_{label}")
}

fn demo_block(
    tree: &mut UiTreeBuilder,
    title: &str,
    caption: &str,
    body: impl FnOnce(&mut UiTreeBuilder, Entity),
) {
    let title_height = 24.0_f32;

    let block = tree
        .add_node()
        .fill_width()
        .flow_vertical()
        .padding(BLOCK_PADDING)
        .gap(4.0)
        .card()
        .entity();

    tree.in_parent(block, |tree| {
        tree.add_node()
            .size(100.pct(), (title_height).px())
            .with_text(title, 16.0)
            .text_left()
            .fg(ThemeColor::Text)
            .entity();

        if !caption.is_empty() {
            tree.add_node()
                .fill_width()
                .auto_size(AutoSizeMode::Height)
                .with_text(caption, 13.0)
                .with_text_wrap()
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                .fg(ThemeColor::TextDisabled)
                .entity();
        }

        tree.add_node().size(100.pct(), (12.0).px()).entity();

        let content = tree
            .add_node()
            .fill_width()
            .flow_vertical()
            .padding(0.0)
            .gap(8.0)
            .entity();
        tree.in_parent(content, |tree| body(tree, content));
    });
}

fn fill_child<R>(
    tree: &mut UiTreeBuilder,
    height: f32,
    body: impl FnOnce(&mut UiTreeBuilder) -> R,
) -> R {
    let wrapper = tree.add_node().size(100.pct(), (height).px()).entity();
    tree.in_parent(wrapper, body)
}

fn build_page_scroll(tree: &mut UiTreeBuilder) -> Entity {
    let pad_holder = tree
        .add_node()
        .boundary(
            Ab(vec2(PAGE_PADDING, PAGE_PADDING)),
            Ab(vec2(-PAGE_PADDING, -PAGE_PADDING)) + Rl(vec2(100.0, 100.0)),
        )
        .entity();

    tree.in_parent(pad_holder, |tree| {
        let scroll = tree.add_scroll_area_fill(8.0, BLOCK_GAP);
        tree.component::<UiScrollAreaData>(scroll)
            .map(|d| d.content_entity)
            .unwrap()
    })
}

fn build_buttons_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Entity, Entity) {
    let screen = build_screen_root(tree, true);
    let mut pulse = Entity::default();
    let mut disable_toggle = Entity::default();
    let mut disabled_slider = Entity::default();

    tree.in_parent(screen, |tree| {
        let content = build_page_scroll(tree);
        tree.in_parent(content, |tree| {
            build_page_header(
                tree,
                "Buttons",
                "Standard button variants, toggle controls, and selection patterns.",
            );

            demo_block(
                tree,
                "Standard Button",
                "The default button style applies the active theme's button effect and shadow.",
                |t, _| {
                    t.add_button("Primary action");
                },
            );
            demo_block(
                tree,
                "Colored Buttons",
                "Pass a color to add_button_colored. Hover, press, and focus glows still derive from the theme.",
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
                |t, _| {
                    t.add_toggle(false);
                },
            );
            demo_block(
                tree,
                "Checkbox",
                "Inline checkbox with label. Focus indicator uses a focused background tint.",
                |t, _| {
                    t.add_checkbox("Enable notifications", false);
                },
            );
            demo_block(
                tree,
                "Radio Group",
                "Mutually exclusive set sharing a group id.",
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
                |t, _| {
                    t.add_selectable_label("Renderer: wgpu", Some(2));
                    t.add_selectable_label("Audio: rodio", Some(2));
                    t.add_selectable_label("Physics: rapier", Some(2));
                },
            );
            demo_block(
                tree,
                "Spinner & Live Indicator",
                "An idle spinner and a sine-driven status pulse from the gallery animation system.",
                |t, _| {
                    t.add_spinner();
                    let theme = t.active_theme().clone();
                    pulse = t
                        .add_node()
                        .size((12.0).px(), (12.0).px())
                        .rect(6.0)
                        .color_raw::<UiBase>(theme.accent_color)
                        .with_tooltip("Live status indicator")
                        .entity();
                },
            );
            demo_block(
                tree,
                "Disabled State",
                "Toggle the switch to disable the slider below. Disabled widgets ignore pointer input.",
                |t, _| {
                    disable_toggle = t.add_toggle(false);
                    disabled_slider = t.add_slider(0.0, 100.0, 30.0);
                },
            );
        });
    });
    (screen, pulse, disable_toggle, disabled_slider)
}

fn build_inputs_screen(
    tree: &mut UiTreeBuilder,
) -> (Entity, Entity, Entity, Entity, Entity, Entity) {
    let screen = build_screen_root(tree, false);
    let mut progress_entity = Entity::default();
    let mut submit_input = Entity::default();
    let mut submit_log = Entity::default();
    let mut focus_target = Entity::default();
    let mut focus_button = Entity::default();

    tree.in_parent(screen, |tree| {
        let content = build_page_scroll(tree);
        tree.in_parent(content, |tree| {
            build_page_header(
                tree,
                "Inputs",
                "Sliders, drag values, text fields, and progress feedback.",
            );

            demo_block(
                tree,
                "Slider",
                "Horizontal track with a focus glow when active.",
                |t, _| {
                    t.add_slider(0.0, 100.0, 50.0);
                },
            );
            demo_block(
                tree,
                "Configured Slider",
                "A SliderConfig adds precision, a suffix, and a step for a value shown to the player.",
                |t, _| {
                    t.add_slider_configured(
                        SliderConfig::new(0.0, 100.0, 40.0).precision(1).suffix("%"),
                    );
                },
            );
            demo_block(
                tree,
                "Range Slider",
                "Two thumbs sharing a track for selecting a numeric range.",
                |t, _| {
                    t.add_range_slider(0.0, 100.0, 25.0, 75.0);
                },
            );
            demo_block(
                tree,
                "Drag Value",
                "Click and drag to scrub. Holding focus lets you type a value directly.",
                |t, _| {
                    t.add_drag_value_configured(DragValueConfig::new(-10.0, 10.0, 0.5).speed(0.01));
                },
            );
            demo_block(
                tree,
                "Text Input",
                "Single-line text field. Supports clipboard, selection, undo.",
                |t, _| {
                    t.add_text_input("Type here...");
                },
            );
            demo_block(
                tree,
                "Logarithmic Slider",
                "A linear slider over twenty to twenty thousand spends nine tenths of its travel above two thousand. This one does not.",
                |t, _| {
                    t.add_slider_configured(
                        SliderConfig::new(20.0, 20000.0, 440.0)
                            .logarithmic()
                            .precision(0)
                            .suffix(" Hz"),
                    );
                },
            );
            demo_block(
                tree,
                "Text Input With Value",
                "The same field opened with text already in it.",
                |t, _| {
                    t.add_text_input_with_value("Name...", "Gallery");
                },
            );
            demo_block(
                tree,
                "Submit Detection",
                "Press Enter to fire a TextInputSubmitted event. The log below increments per submit.",
                |t, _| {
                    submit_input = t.add_text_input("Press Enter to submit...");
                    submit_log = t
                        .add_node()
                        .size(100.pct(), (22.0).px())
                        .with_text("Press Enter to submit", 12.0)
                        .fg(ThemeColor::TextDisabled)
                        .entity();
                },
            );
            demo_block(
                tree,
                "Programmatic Focus",
                "Call ui_focus(world.res_mut::<nightshade_ui::resources::RetainedUiInteraction>(), target) to move keyboard focus to any widget.",
                |t, _| {
                    focus_target = t.add_text_input("Focus target");
                    focus_button = t.add_button("Focus the input above");
                },
            );
            demo_block(
                tree,
                "Text Area",
                "Multi-line text with optional row count.",
                |t, _| {
                    t.add_text_area("Multi-line input...", 4);
                },
            );
            demo_block(
                tree,
                "Rich Text Editor",
                "Multi-line editor with inline span styling.",
                |t, _| {
                    t.add_rich_text_editor("Rich text content...", 4);
                },
            );
            demo_block(
                tree,
                "Progress Bar",
                "Live looping bar plus a static 75% fill.",
                |t, _| {
                    progress_entity = t.add_progress_bar(0.0);
                    t.add_progress_bar(0.75);
                },
            );
        });
    });
    (
        screen,
        progress_entity,
        submit_input,
        submit_log,
        focus_target,
        focus_button,
    )
}

fn build_pickers_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity) {
    let screen = build_screen_root(tree, false);
    let mut palette_btn = Entity::default();

    tree.in_parent(screen, |tree| {
        let content = build_page_scroll(tree);
        tree.in_parent(content, |tree| {
            build_page_header(
                tree,
                "Pickers",
                "Selection widgets for discrete values: dropdowns, color, date, multi-select.",
            );

            demo_block(
                tree,
                "Dropdown",
                "Single-select dropdown with searchable variant available.",
                |t, _| {
                    t.add_dropdown(&["Low", "Medium", "High", "Ultra"], 1);
                },
            );
            demo_block(
                tree,
                "Searchable Dropdown",
                "A dropdown with a filter box, for lists longer than about ten options.",
                |t, _| {
                    t.add_dropdown_searchable(
                        &[
                            "Argon", "Boron", "Carbon", "Helium", "Neon", "Oxygen", "Radon",
                        ],
                        0,
                    );
                },
            );
            demo_block(
                tree,
                "Tab Bar",
                "Horizontal tab strip. Each tab is keyboard-navigable.",
                |t, _| {
                    t.add_tab_bar(&["General", "Audio", "Display", "Controls"], 0);
                },
            );
            demo_block(
                tree,
                "Color Picker",
                "Compact color picker with HSV controls.",
                |t, _| {
                    t.add_color_picker(vec4(0.3, 0.5, 0.9, 1.0));
                },
            );
            demo_block(
                tree,
                "Color Picker (HSV)",
                "The same control in hue, saturation, and value.",
                |t, _| {
                    t.add_color_picker_with_mode(vec4(0.3, 0.5, 0.9, 1.0), ColorPickerMode::Hsv);
                },
            );
            demo_block(tree, "Color Wheel", "Circular HSV color wheel.", |t, _| {
                t.add_color_wheel(vec4(0.3, 0.5, 0.9, 1.0));
            });
            demo_block(
                tree,
                "Date Picker",
                "Calendar with month navigation.",
                |t, _| {
                    t.add_date_picker(2026, 5, 7);
                },
            );
            demo_block(
                tree,
                "Multi-Select",
                "Pick multiple values from a list. Selections highlight inline.",
                |t, _| {
                    t.add_multi_select(&["Apple", "Banana", "Cherry", "Date", "Elderberry"]);
                },
            );
            demo_block(tree, "Menu", "Drop-down menu with item list.", |t, _| {
                t.add_menu("File", &["New", "Open", "Save", "Save As...", "Quit"]);
            });
            demo_block(
                tree,
                "Command Palette",
                "Fuzzy-searchable command list. Press Ctrl+P or use the button below.",
                |t, _| {
                    palette_btn = t.add_button("Open Command Palette (Ctrl+P)");
                },
            );
        });
    });
    (screen, palette_btn)
}

fn build_containers_screen(
    tree: &mut UiTreeBuilder,
) -> (Entity, Entity, Entity, Entity, Entity, Entity) {
    let screen = build_screen_root(tree, false);
    let mut modal_btn = Entity::default();
    let mut confirm_btn = Entity::default();
    let mut modal_dialog = Entity::default();
    let mut confirm_dialog = Entity::default();
    let mut canvas_demo = Entity::default();

    tree.in_parent(screen, |tree| {
        let content = build_page_scroll(tree);
        tree.in_parent(content, |tree| {
            build_page_header(
                tree,
                "Containers",
                "Layout primitives and overlay surfaces.",
            );

            demo_block(
                tree,
                "Collapsing Header",
                "Click the header to expand and collapse the body.",
                |t, content| {
                    let scope = t.add_collapsing_header("Click to expand", true);
                    let body = t.component::<UiCollapsingHeaderData>(scope)
                        .map(|d| d.content_entity)
                        .unwrap_or(content);
                    t.in_parent(body, |t| {
                        t.add_label("Content nested inside the collapsing header.");
                        t.add_label("Click the header above to toggle visibility.");
                        t.add_button("Nested button");
                    });
                },
            );
            demo_block(
                tree,
                "Splitter",
                "Drag the divider to redistribute space between two panels.",
                |t, _| {
                    fill_child(t, 96.0, |t| {
                        t.add_splitter(SplitDirection::Horizontal, 0.5);
                    });
                },
            );
            demo_block(
                tree,
                "Breadcrumb",
                "Hierarchical navigation trail.",
                |t, _| {
                    t.add_breadcrumb(&["Home", "Library", "Sample Item"]);
                },
            );
            demo_block(
                tree,
                "Scroll Area",
                "Fixed-size container with scrollable content overflow.",
                |t, _| {
                    let scroll = t.add_scroll_area(vec2(0.0, 140.0));
                    let content = t.component::<UiScrollAreaData>(scroll)
                        .map(|d| d.content_entity)
                        .unwrap();
                    t.in_parent(content, |t| {
                        for index in 0..30 {
                            t.add_label(&format!("Scrollable item {index}"));
                        }
                    });
                },
            );
            demo_block(
                tree,
                "Modal Dialog",
                "Open a custom modal or a confirmation prompt with two outcomes.",
                |t, _| {
                    modal_btn = t.add_button("Show Modal Dialog");
                    confirm_btn = t.add_button("Show Confirm Dialog");
                },
            );
            demo_block(
                tree,
                "Canvas",
                "Custom drawing surface that emits draw commands you implement.",
                |t, _| {
                    canvas_demo = t.add_canvas(vec2(280.0, 60.0));
                },
            );
            demo_block(
                tree,
                "Interactive Canvas",
                "The same surface with an interaction component, so it hit-tests and reports clicks.",
                |t, _| {
                    t.add_canvas_interactive(vec2(280.0, 60.0));
                },
            );
        });

        modal_dialog = tree.add_modal_dialog("Custom Modal", 380.0, 220.0);
        let modal_content = tree.component::<UiModalDialogData>(modal_dialog)
            .map(|data| data.content_entity)
            .unwrap_or(modal_dialog);
        tree.build_ui(modal_content, |inner| {
            inner.heading("This is a custom modal");
            inner.separator();
            inner.label("Click outside the dialog to dismiss it.");
            inner.spacing(6.0);
            inner.label("Or close it programmatically from your app.");
        });

        confirm_dialog =
            tree.add_confirm_dialog("Confirm Action", "Are you sure you want to proceed?");
    });
    (
        screen,
        modal_btn,
        modal_dialog,
        confirm_btn,
        confirm_dialog,
        canvas_demo,
    )
}

fn build_tiles_screen(tree: &mut UiTreeBuilder) -> Entity {
    let screen = build_screen_root(tree, false);
    tree.in_parent(screen, |tree| {
        let pad_holder = tree
            .add_node()
            .boundary(
                Ab(vec2(PAGE_PADDING, PAGE_PADDING)),
                Ab(vec2(-PAGE_PADDING, -PAGE_PADDING)) + Rl(vec2(100.0, 100.0)),
            )
            .flow_vertical()
            .padding(0.0)
            .gap(BLOCK_GAP)
            .entity();
        tree.in_parent(pad_holder, |tree| {
            build_page_header(
                tree,
                "Tiles",
                "Dockable tile container with drag-to-rearrange tabs and pane splits.",
            );

            let surface = tree.add_node().fill().flex_grow(1.0).card().entity();
            tree.in_parent(surface, |tree| {
                let inner = tree
                    .add_node()
                    .boundary(
                        Ab(vec2(BLOCK_PADDING, BLOCK_PADDING)),
                        Ab(vec2(-BLOCK_PADDING, -BLOCK_PADDING)) + Rl(vec2(100.0, 100.0)),
                    )
                    .entity();
                tree.in_parent(inner, |tree| {
                    let tile_container = tree.add_tile_container(vec2(0.0, 0.0));
                    tree.edit::<nightshade_ui::components::UiLayoutNode>(
                        tile_container,
                        |node| {
                            if let Some(nightshade_ui::components::layout_types::UiLayoutType::Window(
                                window,
                            )) = node.base_layout.as_mut()
                            {
                                window.size = (Rl(vec2(100.0, 100.0))).into();
                            }
                        },
                    );

                    for title in [
                        "Scene",
                        "Inspector",
                        "Console",
                        "Assets",
                        "Profiler",
                        "Output",
                    ] {
                        if let Some((_pane_id, content)) =
                            ui_tile_add_pane(&mut tree.context_reborrow(), tile_container, title)
                        {
                            tree.in_parent(content, |tree| {
                                tree.add_node()
                                    .size(100.pct(), (22.0).px())
                                    .with_text(title, 14.0)
                                    .text_left()
                                    .fg(ThemeColor::Text)
                                    .entity();
                                tree.add_node()
                                    .size(100.pct(), (18.0).px())
                                    .with_text(
                                        "Drag this tab to a different edge to split, or onto another tab to combine.",
                                        13.0,
                                    )
                                    .with_text_wrap()
                                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                                    .fg(ThemeColor::TextDisabled)
                                    .entity();
                            });
                        }
                    }
                });
            });
        });
    });
    screen
}

fn build_data_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Vec<Entity>) {
    let screen = build_screen_root(tree, false);
    let mut virtual_list = Entity::default();
    let mut text_nodes: Vec<Entity> = Vec::new();

    tree.in_parent(screen, |tree| {
        let content = build_page_scroll(tree);
        tree.in_parent(content, |tree| {
            build_page_header(tree, "Data", "Tabular and hierarchical data widgets.");

            demo_block(
                tree,
                "Property Grid",
                "Two-column inspector grid with sections, drag values, and toggles.",
                |t, _| {
                    let grid = t.add_property_grid(110.0);
                    let transform_section = t.add_property_section(grid, "Transform");
                    for (label, default) in [("X", 0.0_f32), ("Y", 1.0), ("Z", 0.0)] {
                        let area = t.add_property_row(grid, transform_section, label);
                        t.in_parent(area, |t| {
                            t.add_drag_value(-10.0, 10.0, default);
                        });
                    }
                    let appearance = t.add_property_section(grid, "Appearance");
                    let area = t.add_property_row(grid, appearance, "Opacity");
                    t.in_parent(area, |t| {
                        t.add_drag_value(0.0, 1.0, 0.85);
                    });
                    let area = t.add_property_row(grid, appearance, "Hidden");
                    t.in_parent(area, |t| {
                        t.add_toggle(false);
                    });
                    t.add_property_checkbox(grid, appearance, "Cast shadows", true);
                    t.add_property_drag_value(grid, appearance, "Bias", 0.0, 1.0, 0.25);
                    t.add_property_dropdown(grid, appearance, "Blend", &["Opaque", "Blend"], 0);
                    t.add_property_text_input(grid, appearance, "Tag", "unnamed");
                },
            );
            demo_block(
                tree,
                "Data Grid",
                "Sortable, scrollable, multi-column grid with cell-level mutation.",
                |t, _| {
                    let grid = fill_child(t, 320.0, |t| {
                        t.add_data_grid(
                            &[
                                DataGridColumn::new("Name", 140.0),
                                DataGridColumn::new("Type", 90.0),
                                DataGridColumn::new("Value", 110.0),
                            ],
                            10,
                        )
                    });
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
                    ui_data_grid_set_row_count(&mut t.context_reborrow(), grid, sample_rows.len());
                    for (row, (name, ty, value)) in sample_rows.iter().enumerate() {
                        ui_data_grid_set_cell(&mut t.context_reborrow(), grid, row, 0, name);
                        ui_data_grid_set_cell(&mut t.context_reborrow(), grid, row, 1, ty);
                        ui_data_grid_set_cell(&mut t.context_reborrow(), grid, row, 2, value);
                    }
                },
            );
            demo_block(
                tree,
                "Virtual List",
                "64 items rendered through a small recycling pool. Scroll to see indices update.",
                |t, _| {
                    let vl = fill_child(t, 220.0, |t| t.add_virtual_list(28.0, 8));
                    virtual_list = vl;
                    ui_virtual_list_set_count(t.commands(), vl, 64);
                    let pool_containers: Vec<Entity> = t
                        .component::<nightshade_ui::components::UiVirtualListData>(vl)
                        .map(|d| d.pool_items.iter().map(|p| p.container_entity).collect())
                        .unwrap_or_default();
                    text_nodes.clear();
                    for (pool_index, container) in pool_containers.iter().enumerate() {
                        let text_entity = t.in_parent(*container, |t| {
                            t.add_node()
                                .fill()
                                .with_text(&format!("Item {pool_index}"), 13.0)
                                .text_left()
                                .fg(ThemeColor::Text)
                                .entity()
                        });
                        text_nodes.push(text_entity);
                    }
                },
            );
            demo_block(
                tree,
                "Tree View",
                "Hierarchical tree with expansion state and selection.",
                |t, _| {
                    fill_child(t, 260.0, |t| {
                        let tree_view = t.add_tree_view(false);
                        let tv_content = t
                            .component::<UiTreeViewData>(tree_view)
                            .map(|d| d.content_entity)
                            .unwrap();
                        let project = t.add_tree_node(tree_view, tv_content, "Project", 0, 0);
                        ui_tree_node_expand_on_build(&mut t.context_reborrow(), project, true);
                        let project_children = t
                            .component::<UiTreeNodeData>(project)
                            .map(|d| d.children_container)
                            .unwrap();
                        t.add_tree_node(tree_view, project_children, "Assets", 1, 1);
                        t.add_tree_node(tree_view, project_children, "Scripts", 1, 2);
                        let scenes = t.add_tree_node(tree_view, project_children, "Scenes", 1, 3);
                        ui_tree_node_expand_on_build(&mut t.context_reborrow(), scenes, true);
                        let scene_children = t
                            .component::<UiTreeNodeData>(scenes)
                            .map(|d| d.children_container)
                            .unwrap();
                        t.add_tree_node_with_icon(
                            tree_view,
                            scene_children,
                            Some(icons::material::INSERT_DRIVE_FILE),
                            "main.scn",
                            2,
                            4,
                        );
                        t.add_tree_node_with_icon(
                            tree_view,
                            scene_children,
                            Some(icons::material::INSERT_DRIVE_FILE),
                            "menu.scn",
                            2,
                            5,
                        );
                    });
                },
            );
        });
    });
    (screen, virtual_list, text_nodes)
}

fn build_effects_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Entity, Entity, Entity) {
    let screen = build_screen_root(tree, false);
    let mut pulse = Entity::default();
    let mut blend = Entity::default();
    let mut progress = Entity::default();
    let mut spinner = Entity::default();

    tree.in_parent(screen, |tree| {
        let content = build_page_scroll(tree);
        tree.in_parent(content, |tree| {
            build_page_header(
                tree,
                "Effects",
                "State transitions, tooltips, toasts, theme switching, and live animation.",
            );

            demo_block(
                tree,
                "Hover & Press Transitions",
                "Eased state-weight color blending. Hover to see CubicOut + BackOut on entry.",
                |t, _| {
                    for label in ["Smooth Hover", "Smooth Press", "Both Effects"] {
                        t.add_button(label);
                    }
                },
            );
            demo_block(
                tree,
                "Rotation",
                "with_rotation turns a node and everything under it about its own center, and the hit test turns with it, so a hover lands on the card you see.",
                |t, _| {
                    let theme = t.active_theme().clone();
                    for lean in [-0.14, 0.0, 0.14] {
                        t.add_node()
                            .size((150.0).px(), (44.0).px())
                            .with_rect(8.0, 1.0, theme.border_color)
                            .with_theme_border_color(ThemeColor::Border)
                            .fg(ThemeColor::Background)
                            .fg_hover(ThemeColor::BackgroundHover)
                            .with_rotation(lean)
                            .with_interaction()
                            .with_cursor_icon(winit::window::CursorIcon::Pointer)
                            .with_children(|inner| {
                                inner
                                    .add_node()
                                    .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                                    .with_text("hover me", 16.0)
                                    .fg(ThemeColor::Text);
                            });
                    }
                },
            );
            demo_block(
                tree,
                "Live Recoloring",
                "ui_set_color rewrites a node color after the tree is built, so a readout can change color without being torn down and made again.",
                |t, _| {
                    let swatch = t
                        .add_node()
                        .size(100.pct(), (34.0).px())
                        .with_text("Pick a color below", 18.0)
                        .text_left()
                        .color_raw::<UiBase>(RECOLOR_CHOICES[0].1)
                        .entity();
                    ui_set_named(t.accessibility_mut(), swatch, "recolor_swatch");

                    for (label, _) in RECOLOR_CHOICES {
                        let button = t.add_button(label);
                        ui_set_named(t.accessibility_mut(), button, &recolor_button_name(label));
                    }
                },
            );
            demo_block(
                tree,
                "Tooltips",
                "Hover the button below for a tooltip. Configurable per-entity via with_tooltip().",
                |t, _| {
                    let theme = t.active_theme().clone();
                    t.add_node()
                        .size(100.pct(), (32.0).px())
                        .with_rect(6.0, 1.0, theme.border_color)
                        .with_theme_border_color(ThemeColor::Border)
                        .fg(ThemeColor::Background)
                        .fg_hover(ThemeColor::BackgroundHover)
                        .with_transition::<UiHover>(8.0, 6.0)
                        .with_interaction()
                        .with_tooltip("This is a tooltip on a custom button")
                        .with_cursor_icon(winit::window::CursorIcon::Pointer)
                        .with_children(|inner| {
                            inner
                                .add_node()
                                .window(
                                    Rl(vec2(50.0, 50.0)),
                                    Ab(vec2(220.0, 22.0)),
                                    Anchor::Center,
                                )
                                .with_text("Hover for tooltip", 13.0)
                                .fg(ThemeColor::Text)
                                ;
                        })
                        .entity();
                },
            );
            demo_block(
                tree,
                "Toast Notifications",
                "Click any of the buttons to fire a toast at that severity. They auto-dismiss after 2s.",
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
                |_, _| {},
            );
            demo_block(
                tree,
                "Live Animation",
                "Pulse, color blend, looping progress, and a rotating swatch driven from gallery animation systems.",
                |t, _| {
                    let theme = t.active_theme().clone();
                    pulse = t
                        .add_node()
                        .size((64.0).px(), (64.0).px())
                        .rect(8.0)
                        .color_raw::<UiBase>(theme.accent_color)
                        .entity();
                    blend = t
                        .add_node()
                        .size(100.pct(), (6.0).px())
                        .rect(3.0)
                        .color_raw::<UiBase>(theme.accent_color)
                        .entity();
                    progress = t.add_progress_bar(0.0);
                    let spinner_holder = t
                        .add_node()
                        .size((60.0).px(), (60.0).px())
                        .entity();
                    spinner = t.in_parent(spinner_holder, |t| {
                        t.add_node()
                            .window(
                                Rl(vec2(50.0, 50.0)),
                                Ab(vec2(20.0, 20.0)),
                                Anchor::Center,
                            )
                            .rect(4.0)
                            .color_raw::<UiBase>(theme.accent_color)
                            .entity()
                    });
                },
            );
        });
    });
    (screen, pulse, blend, progress, spinner)
}

fn build_media_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Entity) {
    let screen = build_screen_root(tree, false);
    let mut menu_target = Entity::default();
    let mut simple_menu_target = Entity::default();

    tree.in_parent(screen, |tree| {
        let content = build_page_scroll(tree);
        tree.in_parent(content, |tree| {
            build_page_header(
                tree,
                "Media",
                "Rich text spans, images uploaded at runtime, and a right-click context menu.",
            );

            demo_block(
                tree,
                "Rich Text",
                "One paragraph whose runs carry their own weight, slant, size, and color.",
                |t, _| {
                    t.add_rich_text(&[
                        TextSpan::new("A paragraph mixes "),
                        TextSpan::new("bold").with_bold(),
                        TextSpan::new(", "),
                        TextSpan::new("italic").with_italic(),
                        TextSpan::new(", "),
                        TextSpan::new("underline").with_underline(),
                        TextSpan::new(", "),
                        TextSpan::colored("color", vec4(0.45, 0.78, 1.0, 1.0)),
                        TextSpan::new(", and "),
                        TextSpan::sized("size", 22.0),
                        TextSpan::new(" in one node."),
                    ]);
                },
            );

            demo_block(
                tree,
                "Rich Button",
                "A button label built from spans rather than one string.",
                |t, _| {
                    t.add_button_rich(&[
                        TextSpan::colored("Save", vec4(0.55, 0.9, 0.6, 1.0)).with_bold(),
                        TextSpan::new(" Project"),
                    ]);
                },
            );

            demo_block(
                tree,
                "Image",
                "Pixels uploaded into the UI atlas at runtime, then drawn by an image node.",
                |t, _| {
                    let pixels = nightshade_texture::queries::surface::checkers(
                        128,
                        16,
                        [94, 134, 214, 255],
                        [28, 34, 48, 255],
                    );
                    let (images, runtime) = t.images_and_runtime();
                    if let Some(upload) = ui_upload_image(images, runtime, &pixels, 128, 128) {
                        t.add_node()
                            .flow_child(Ab(vec2(128.0, 128.0)))
                            .with_image_uv(upload.layer, upload.uv_min, upload.uv_max)
                            .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 1.0))
                            .entity();
                    }
                },
            );

            demo_block(
                tree,
                "Context Menu",
                "Right-click the panel below. The menu carries shortcuts, a checkable row, and a submenu.",
                |t, _| {
                    menu_target = t
                        .add_node()
                        .fill_width()
                        .size(100.pct(), (96.0).px())
                        .card()
                        .with_interaction()
                        .with_text("Right-click here", 14.0)
                        .fg(ThemeColor::TextDisabled)
                        .entity();
                },
            );

            demo_block(
                tree,
                "Context Menu (short form)",
                "The same widget from a list of label and shortcut pairs. An empty label is a separator.",
                |t, _| {
                    simple_menu_target = t
                        .add_node()
                        .fill_width()
                        .size(100.pct(), (96.0).px())
                        .card()
                        .with_interaction()
                        .with_text("Right-click here too", 14.0)
                        .fg(ThemeColor::TextDisabled)
                        .entity();
                },
            );
        });
    });

    (screen, menu_target, simple_menu_target)
}

fn build_polish_screen(tree: &mut UiTreeBuilder) -> (Entity, Entity, Entity, Entity) {
    use nightshade_ui::components::Spring;

    let screen = build_screen_root(tree, false);
    let mut progress_slider = Entity::default();
    let mut progress_indet = Entity::default();
    let mut progress_clear = Entity::default();

    tree.in_parent(screen, |tree| {
        let content = build_page_scroll(tree);
        tree.in_parent(content, |tree| {
            build_page_header(
                tree,
                "Polish",
                "Hover lift, press compression, focus glow, spring tuning, and the top progress bar.",
            );

            demo_block(
                tree,
                "Hover Lift + Press Compress",
                "Standard buttons lift 2px on hover (BackOut spring) and sink 2px on press (snappy spring).",
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
                |t, _| {
                    t.add_toggle(false);
                    t.add_slider(0.0, 100.0, 50.0);
                },
            );
            demo_block(
                tree,
                "Focus Glow",
                "Tab through the inputs to see the focused border tint and tight focus ring.",
                |t, _| {
                    t.add_text_input("First input");
                    t.add_text_input("Second input");
                },
            );
            demo_block(
                tree,
                "Spring Tuning",
                "Same hover-lift offset, four different spring configs. Hover each to compare overshoot and damping.",
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
                |t, _| {
                    linear_demo_button(t, "Linear hover");
                    spring_demo_button(t, "Spring hover", Spring::lively());
                },
            );
            demo_block(
                tree,
                "Top-of-Window Progress Bar",
                "Drag the slider to set a determinate value, click indeterminate for a sliding bar, or clear to hide.",
                |t, _| {
                    progress_slider = t.add_slider(0.0, 1.0, 0.0);
                    progress_indet = t.add_button("Set Indeterminate");
                    progress_clear = t.add_button("Clear Progress Bar");
                },
            );
        });
    });
    (screen, progress_slider, progress_indet, progress_clear)
}

fn spring_demo_button(
    tree: &mut UiTreeBuilder,
    label: &str,
    spring: nightshade_ui::components::Spring,
) -> Entity {
    let theme = tree.active_theme().clone();
    tree.add_node()
        .size(100.pct(), (32.0).px())
        .with_rect(theme.corner_radius, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .fg(ThemeColor::Background)
        .fg_hover(ThemeColor::BackgroundHover)
        .fg_pressed(ThemeColor::BackgroundActive)
        .with_theme_effect_role(ThemeEffect::ButtonEffect)
        .with_theme_shadow_role::<UiBase>(ThemeShadow::ButtonShadow)
        .with_theme_shadow_role::<UiHover>(ThemeShadow::ButtonShadowHover)
        .with_theme_shadow_role::<UiPressed>(ThemeShadow::ButtonShadowPressed)
        .with_state_offset::<UiHover>(vec2(0.0, -3.0))
        .with_state_offset::<UiPressed>(vec2(0.0, 2.0))
        .with_spring_transition::<UiHover>(spring)
        .with_spring_transition::<UiPressed>(spring)
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .with_children(|inner| {
            inner
                .add_node()
                .window(Rl(vec2(50.0, 50.0)), Ab(vec2(240.0, 22.0)), Anchor::Center)
                .with_text(label, 14.0)
                .fg(ThemeColor::Text);
        })
        .entity()
}

fn linear_demo_button(tree: &mut UiTreeBuilder, label: &str) -> Entity {
    use nightshade_ui::data::easing::EasingFunction;
    let theme = tree.active_theme().clone();
    tree.add_node()
        .size(100.pct(), (32.0).px())
        .with_rect(theme.corner_radius, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .fg(ThemeColor::Background)
        .fg_hover(ThemeColor::BackgroundHover)
        .fg_pressed(ThemeColor::BackgroundActive)
        .with_theme_effect_role(ThemeEffect::ButtonEffect)
        .with_theme_shadow_role::<UiBase>(ThemeShadow::ButtonShadow)
        .with_theme_shadow_role::<UiHover>(ThemeShadow::ButtonShadowHover)
        .with_state_offset::<UiHover>(vec2(0.0, -3.0))
        .with_state_offset::<UiPressed>(vec2(0.0, 2.0))
        .with_eased_transition::<UiHover>(8.0, 6.0, EasingFunction::Linear)
        .with_eased_transition::<UiPressed>(12.0, 8.0, EasingFunction::Linear)
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .with_children(|inner| {
            inner
                .add_node()
                .window(Rl(vec2(50.0, 50.0)), Ab(vec2(240.0, 22.0)), Anchor::Center)
                .with_text(label, 14.0)
                .fg(ThemeColor::Text);
        })
        .entity()
}
