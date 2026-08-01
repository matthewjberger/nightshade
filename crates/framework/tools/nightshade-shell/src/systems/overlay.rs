use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};
use nightshade::ecs::Entity;
use nightshade::ecs::system_param::{Commands, Res, ResMut};
use nightshade_platform::commands::input_capture::set_keyboard_claim;
use nightshade_ui::commands::builder::{UiBuildContext, UiTreeBuilder};
use nightshade_ui::commands::widgets::world_interaction::ui_mark_layout_dirty;
use nightshade_ui::commands::widgets::world_layout::ui_set_visible;
use nightshade_ui::components::layer::UiLayer;
use nightshade_ui::components::layout_types::UiLayoutType;
use nightshade_ui::components::{UiNodeInteraction, UiScrollAreaData};
use nightshade_ui::data::state::{UiBase, UiStateTrait};
use nightshade_ui::data::units::Ab;

use crate::resources::overlay::{LineState, ShellOverlay, ShellOverlayState};
use crate::resources::state::ShellState;

const FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT_RATIO: f32 = 1.4;
const PADDING: f32 = 10.0;
const INPUT_HEIGHT: f32 = 32.0;
const SEPARATOR_HEIGHT: f32 = 1.0;
const RESIZE_HANDLE_HEIGHT: f32 = 6.0;
const MIN_HEIGHT: f32 = 100.0;
const PROMPT_WIDTH: f32 = 18.0;

const MAX_SCROLLBACK_ROWS: usize = 2000;

const BACKGROUND: Vec4 = Vec4::new(0.06, 0.06, 0.08, 0.95);
const BORDER: Vec4 = Vec4::new(0.4, 0.4, 0.45, 1.0);
const PROMPT: Vec4 = Vec4::new(0.39, 0.78, 0.39, 1.0);
const COMMAND_TEXT: Vec4 = Vec4::new(0.59, 0.78, 1.0, 1.0);
const OUTPUT_TEXT: Vec4 = Vec4::new(0.78, 0.78, 0.78, 1.0);
const HANDLE_IDLE: Vec4 = Vec4::new(0.5, 0.5, 0.55, 0.8);
const HANDLE_ACTIVE: Vec4 = Vec4::new(0.6, 0.8, 1.0, 1.0);

fn set_node_color(commands: &mut Commands, entity: Entity, color: Vec4) {
    commands.queue(move |storage, _| {
        if let Some(node_color) = storage.get_mut::<nightshade_ui::components::UiNodeColor>(entity)
        {
            node_color.colors[UiBase::INDEX] = Some(color);
            node_color.computed_color = color;
        }
    });
}

struct OutputRow {
    text: String,
    is_command: bool,
}

fn output_rows<C>(shell: &ShellState<C>) -> Vec<OutputRow> {
    let mut rows = Vec::new();
    for line in &shell.output {
        if line.text.is_empty() {
            rows.push(OutputRow {
                text: String::new(),
                is_command: line.is_command,
            });
            continue;
        }
        for segment in line.text.split('\n') {
            rows.push(OutputRow {
                text: segment.to_string(),
                is_command: line.is_command,
            });
        }
    }
    if rows.len() > MAX_SCROLLBACK_ROWS {
        rows.drain(..rows.len() - MAX_SCROLLBACK_ROWS);
    }
    rows
}

struct DeveloperShell;

fn staged_text_slot(tree: &mut UiTreeBuilder, entity: Entity) -> usize {
    match tree.component::<nightshade_ui::components::UiNodeContent>(entity) {
        Some(nightshade_ui::components::UiNodeContent::Text { text_slot, .. }) => *text_slot,
        _ => 0,
    }
}

fn build_overlay(context: UiBuildContext) -> ShellOverlay {
    let forced_ui_visible = !context.runtime.visible;
    let mut tree = UiTreeBuilder::new(context);
    tree.runtime_mut().visible = true;

    let panel = tree
        .add_node()
        .window_at(Ab(Vec2::zeros()), Ab(Vec2::new(0.0, 0.0)))
        .rect(0.0)
        .color_raw::<UiBase>(BACKGROUND)
        .with_layer(UiLayer::Tooltips)
        .with_clip()
        .with_interaction()
        .flow_vertical()
        .padding(0.0)
        .gap(0.0)
        .entity();

    let (scroll, content, input_slot, resize_handle) = tree.in_parent(panel, |tree| {
        let scroll = tree.add_scroll_area_fill(PADDING, 0.0);
        let content = tree
            .component::<UiScrollAreaData>(scroll)
            .map(|data| data.content_entity)
            .unwrap_or(scroll);

        tree.add_node()
            .fill_width()
            .size_px(0.0, SEPARATOR_HEIGHT)
            .rect(0.0)
            .color_raw::<UiBase>(BORDER);

        let input_row = tree
            .add_node()
            .fill_width()
            .size_px(0.0, INPUT_HEIGHT)
            .flow_horizontal()
            .padding(0.0)
            .gap(0.0)
            .align_cross(nightshade_ui::components::layout_types::FlowAlignment::Center)
            .entity();

        let input_text = tree.in_parent(input_row, |tree| {
            tree.add_node()
                .size_px(PROMPT_WIDTH, INPUT_HEIGHT)
                .with_text(">", FONT_SIZE)
                .text_left()
                .color_raw::<UiBase>(PROMPT);
            tree.add_node()
                .fill_width()
                .flex_grow(1.0)
                .with_text("", FONT_SIZE)
                .text_left()
                .color_raw::<UiBase>(OUTPUT_TEXT)
                .entity()
        });
        let input_slot = staged_text_slot(tree, input_text);

        let resize_handle = tree
            .add_node()
            .fill_width()
            .size_px(0.0, RESIZE_HANDLE_HEIGHT)
            .rect(RESIZE_HANDLE_HEIGHT * 0.5)
            .color_raw::<UiBase>(HANDLE_IDLE)
            .with_interaction()
            .with_cursor_icon(nightshade_platform::CursorIcon::NsResize)
            .entity();

        (scroll, content, input_slot, resize_handle)
    });

    tree.finish();

    ShellOverlay {
        panel,
        scroll,
        content,
        lines: Vec::new(),
        input_slot,
        resize_handle,
        panel_visible: false,
        panel_size: Vec2::new(-1.0, -1.0),
        input_cache: String::new(),
        handle_active: false,
        last_content_height: f32::NAN,
        last_output_len: usize::MAX,
        forced_ui_visible,
    }
}

fn sync_output(context: &mut UiBuildContext, overlay: &mut ShellOverlay, rows: &[OutputRow]) {
    if overlay.lines.len() < rows.len() {
        let content = overlay.content;
        let mut tree = UiTreeBuilder::from_parent(context.reborrow(), content);
        for _ in overlay.lines.len()..rows.len() {
            let entity = tree
                .add_node()
                .fill_width()
                .size_px(0.0, (FONT_SIZE * LINE_HEIGHT_RATIO).round())
                .with_text("", FONT_SIZE)
                .text_left()
                .color_raw::<UiBase>(OUTPUT_TEXT)
                .entity();
            let text_slot = staged_text_slot(&mut tree, entity);
            overlay.lines.push(LineState {
                entity,
                text_slot,
                text: String::new(),
                is_command: false,
                visible: true,
            });
        }
        tree.finish();
    }

    for index in 0..overlay.lines.len() {
        match rows.get(index) {
            Some(row) => {
                let entity = overlay.lines[index].entity;
                if !overlay.lines[index].visible {
                    ui_set_visible(context.dirty, context.commands, entity, true);
                    overlay.lines[index].visible = true;
                }
                if overlay.lines[index].text != row.text {
                    context
                        .text_state
                        .cache
                        .set_text(overlay.lines[index].text_slot, &row.text);
                    overlay.lines[index].text.clear();
                    overlay.lines[index].text.push_str(&row.text);
                }
                if overlay.lines[index].is_command != row.is_command {
                    let color = if row.is_command {
                        COMMAND_TEXT
                    } else {
                        OUTPUT_TEXT
                    };
                    set_node_color(context.commands, entity, color);
                    overlay.lines[index].is_command = row.is_command;
                }
            }
            None => {
                if overlay.lines[index].visible {
                    let entity = overlay.lines[index].entity;
                    ui_set_visible(context.dirty, context.commands, entity, false);
                    overlay.lines[index].visible = false;
                }
            }
        }
    }
}

fn apply_resize<C>(
    shell: &mut ShellState<C>,
    commands: &mut Commands,
    handles: &mut ResizeHandles<'_, '_>,
    overlay: &mut ShellOverlay,
    scale: f32,
    mouse_y: f32,
    max: f32,
) {
    let mut interaction = None;
    handles.for_entity(overlay.resize_handle, |(handle,)| {
        interaction = Some((handle.dragging, handle.drag_start));
    });
    let Some((dragging, drag_start)) = interaction else {
        return;
    };

    let mouse_y = mouse_y / scale;

    if dragging {
        if !shell.dragging_resize {
            shell.dragging_resize = true;
            shell.drag_start_y = drag_start.map(|start| start.y / scale).unwrap_or(mouse_y);
            shell.drag_start_height = shell.height;
        }
        shell.height = (shell.drag_start_height + (mouse_y - shell.drag_start_y))
            .clamp(MIN_HEIGHT.min(max), max);
    } else {
        shell.dragging_resize = false;
    }

    if overlay.handle_active != shell.dragging_resize {
        overlay.handle_active = shell.dragging_resize;
        let color = if shell.dragging_resize {
            HANDLE_ACTIVE
        } else {
            HANDLE_IDLE
        };
        set_node_color(commands, overlay.resize_handle, color);
    }
}

fn set_panel_rect(
    dirty: &mut nightshade_ui::resources::RetainedUiDirty,
    commands: &mut Commands,
    overlay: &mut ShellOverlay,
    size: Vec2,
) {
    if overlay.panel_size == size {
        return;
    }
    overlay.panel_size = size;
    let panel = overlay.panel;
    commands.queue(move |storage, _| {
        if let Some(node) = storage.get_mut::<nightshade_ui::components::UiLayoutNode>(panel)
            && let Some(UiLayoutType::Window(window)) = node.base_layout.as_mut()
        {
            window.position = Ab(Vec2::zeros()).into();
            window.size = Ab(size).into();
        }
    });
    ui_mark_layout_dirty(dirty);
}

fn set_panel_visible(
    dirty: &mut nightshade_ui::resources::RetainedUiDirty,
    commands: &mut Commands,
    overlay: &mut ShellOverlay,
    visible: bool,
) {
    if overlay.panel_visible == visible {
        return;
    }
    overlay.panel_visible = visible;
    ui_set_visible(dirty, commands, overlay.panel, visible);
}

fn apply_scroll_to_bottom<C>(
    shell: &mut ShellState<C>,
    areas: &mut ScrollAreas<'_, '_>,
    overlay: &mut ShellOverlay,
    scale: f32,
) {
    let mut measured = None;
    areas.for_entity(overlay.scroll, |(data,)| {
        measured = Some((data.content_height, data.visible_height));
    });
    let Some((content_height, visible_height)) = measured else {
        return;
    };

    let bottom = ((content_height - visible_height).max(0.0)) / scale;
    areas.for_entity(overlay.scroll, |(data,)| data.scroll_offset = bottom);

    let settled = (content_height - overlay.last_content_height).abs() < f32::EPSILON;
    overlay.last_content_height = content_height;
    if settled {
        shell.scroll_to_bottom = false;
    }
}

pub type ResizeHandles<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item UiNodeInteraction,)>;

pub type ScrollAreas<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item mut UiScrollAreaData,)>;

pub fn shell_retained_ui_system<C: Send + Sync + 'static>(
    mut shell: ResMut<ShellState<C>>,
    mut overlay_state: ResMut<ShellOverlayState>,
    mut capture: ResMut<nightshade_platform::contract::input_capture::InputCapture>,
    window: Res<nightshade_platform::contract::window::Window>,
    input: Res<nightshade_platform::contract::input::Input>,
    mut build: nightshade_ui::commands::builder::UiBuild<'_>,
    mut handles: ResizeHandles<'_, '_>,
    mut areas: ScrollAreas<'_, '_>,
) {
    shell_retained_ui(
        &mut shell,
        &mut overlay_state,
        &mut capture,
        &window,
        &input,
        &mut build,
        &mut handles,
        &mut areas,
    );
}

fn shell_retained_ui<C: Send + Sync + 'static>(
    shell: &mut ShellState<C>,
    overlay_state: &mut ShellOverlayState,
    capture: &mut nightshade_platform::contract::input_capture::InputCapture,
    window: &nightshade_platform::contract::window::Window,
    input: &nightshade_platform::contract::input::Input,
    build: &mut nightshade_ui::commands::builder::UiBuild<'_>,
    handles: &mut ResizeHandles<'_, '_>,
    areas: &mut ScrollAreas<'_, '_>,
) {
    if !shell.should_render() {
        if let Some(mut overlay) = overlay_state.0.take() {
            let forced = overlay.forced_ui_visible;
            {
                let context = build.context();
                set_panel_visible(context.dirty, context.commands, &mut overlay, false);
                if forced {
                    context.runtime.visible = false;
                }
            }
            overlay_state.0 = Some(overlay);
        }
        shell.pending_enter = false;
        shell.pending_up = false;
        shell.pending_down = false;
        shell.pending_escape = false;
        shell.dragging_resize = false;
        set_keyboard_claim::<DeveloperShell>(capture, false);
        return;
    }

    let mut overlay = match overlay_state.0.take() {
        Some(overlay) => overlay,
        None => build_overlay(build.context()),
    };

    if overlay.forced_ui_visible {
        build.context().runtime.visible = true;
    }

    set_keyboard_claim::<DeveloperShell>(capture, shell.visible);

    let scale = window.cached_scale_factor.max(f32::EPSILON);
    let viewport = window
        .cached_viewport_size
        .map(|(width, height)| Vec2::new(width as f32, height as f32))
        .unwrap_or(Vec2::new(1920.0, 1080.0))
        / scale;
    let mouse_y = input.mouse.position.y;

    let max_height = (viewport.y * 0.9).max(0.0);
    shell.height = shell.height.clamp(MIN_HEIGHT.min(max_height), max_height);

    apply_resize(
        shell,
        build.context().commands,
        handles,
        &mut overlay,
        scale,
        mouse_y,
        max_height,
    );

    let open_height = shell.height * shell.animation_progress;
    let rows = (overlay.last_output_len != shell.output.len()).then(|| {
        overlay.last_output_len = shell.output.len();
        output_rows(shell)
    });

    let cursor = if shell.visible && shell.animation_progress > 0.9 {
        "_"
    } else {
        ""
    };
    let input_display = format!("{}{}", shell.input_buffer, cursor);
    let input_changed = overlay.input_cache != input_display;

    {
        let mut context = build.context();
        set_panel_visible(context.dirty, context.commands, &mut overlay, true);
        set_panel_rect(
            context.dirty,
            context.commands,
            &mut overlay,
            Vec2::new(viewport.x, open_height),
        );
        if let Some(rows) = rows {
            sync_output(&mut context, &mut overlay, &rows);
        }
        if input_changed {
            context
                .text_state
                .cache
                .set_text(overlay.input_slot, &input_display);
        }
    }
    if input_changed {
        overlay.input_cache = input_display;
    }

    if shell.scroll_to_bottom {
        apply_scroll_to_bottom(shell, areas, &mut overlay, scale);
    }

    if shell.visible && shell.animation_progress > 0.9 {
        if shell.pending_up {
            shell.pending_up = false;
            shell.history_up();
        }
        if shell.pending_down {
            shell.pending_down = false;
            shell.history_down();
        }
        if shell.pending_escape {
            shell.pending_escape = false;
            shell.visible = false;
        }
    }

    overlay_state.0 = Some(overlay);
}
