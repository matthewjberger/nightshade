use nalgebra_glm;
mod events;
mod handle_basic;
pub mod handle_context_menu;
mod handle_data_grid;
mod handle_drag_value;
pub mod handle_frame;
mod handle_input;
mod handle_navigation;
mod handle_panel;
mod handle_rich_text;
pub mod handle_scroll;
mod handle_slider;
mod handle_text_area;
mod handle_tiles;
mod handle_tree;
mod input_sync;
mod pool_window;
pub(crate) mod range_slider_visuals;
pub mod text_cursor;
mod text_edit;

pub use events::{ui_emit_click_events_system, ui_event_bubble_system};
pub use handle_basic::{
    ui_checkbox_system, ui_collapsing_header_system, ui_radio_system, ui_spinner_system,
    ui_toggle_system,
};
pub use handle_context_menu::ui_context_menu_system;
pub use handle_data_grid::{
    ui_command_palette_system, ui_data_grid_start_edit, ui_data_grid_stop_edit,
    ui_data_grid_system, ui_property_grid_system,
};
pub use handle_drag_value::ui_drag_value_system;
pub use handle_frame::{
    ui_button_system, ui_canvas_system, ui_error_border_system, ui_popup_dismiss_system,
    ui_shortcut_system,
};
pub use handle_input::{ui_selectable_label_system, ui_text_input_system};
pub use handle_navigation::{
    ui_date_picker_system, ui_dropdown_system, ui_menu_system, ui_multi_select_system,
    ui_tab_bar_system,
};
pub use handle_panel::{ui_color_picker_system, ui_color_wheel_system, ui_panel_system};
pub use handle_rich_text::ui_rich_text_editor_system;
pub use handle_scroll::{ui_scroll_area_system, ui_virtual_list_system};
pub use handle_slider::{
    ui_breadcrumb_system, ui_range_slider_system, ui_slider_system, ui_splitter_system,
};
pub use handle_text_area::ui_text_area_system;
pub use handle_tiles::ui_tile_container_system;
pub use handle_tree::{ui_modal_dialog_system, ui_tree_view_system};
pub use input_sync::ui_retained_input_sync_system;

use nalgebra_glm::Vec2;
use nightshade_platform::KeyCode;

struct TextEditContext<'a> {
    focused_entity: Option<nightshade::ecs::Entity>,
    frame_chars: &'a [char],
    frame_keys: &'a [(KeyCode, bool)],
    ctrl_held: bool,
    shift_held: bool,
    mouse_position: Vec2,
    current_time: f64,
    dpi_scale: f32,
}
