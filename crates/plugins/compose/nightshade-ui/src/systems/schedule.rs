use crate::data::systems::{UiSystemFn, UiSystemPhase, ui_system};
use crate::resources::RetainedUiSchedule;
use nightshade::World;
use nightshade_platform::commands::input_capture::{set_keyboard_claim, set_pointer_claim};

struct RetainedTree;

pub fn run_retained_ui_schedule(world: &mut World) {
    let mut index = 0;
    loop {
        let mut system = {
            let mut systems = world
                .res::<RetainedUiSchedule>()
                .systems
                .lock()
                .expect("retained ui schedule");
            if index >= systems.len() {
                break;
            }
            std::mem::replace(&mut systems[index], Box::new(|_| {}))
        };
        system(world);
        world
            .res::<RetainedUiSchedule>()
            .systems
            .lock()
            .expect("retained ui schedule")[index] = system;
        index += 1;
    }
    report_input_capture(&mut world.resources, &world.storage);
}

fn report_input_capture(
    resources: &mut nightshade::ecs::ResourceMap,
    storage: &nightshade::ecs::Storage,
) {
    let wants_pointer = crate::ui_wants_pointer(resources, storage);
    let wants_keyboard = crate::ui_wants_keyboard(resources, storage);
    let capture = resources.res_mut::<nightshade_platform::contract::input_capture::InputCapture>();
    set_pointer_claim::<RetainedTree>(capture, wants_pointer);
    set_keyboard_claim::<RetainedTree>(capture, wants_keyboard);
}

pub fn ui_systems_with(extra: Vec<(UiSystemPhase, UiSystemFn)>) -> Vec<UiSystemFn> {
    let mut extra = extra;
    let mut systems: Vec<UiSystemFn> = vec![
        ui_system(crate::systems::render_sync::reclaim_frame),
        ui_system(crate::systems::widgets::ui_retained_input_sync_system),
        ui_system(crate::systems::picking::ui_layout_picking_system),
        ui_system(crate::systems::gamepad_navigation::ui_gamepad_navigation_system),
    ];
    let mut push_phase = |systems: &mut Vec<UiSystemFn>, phase: UiSystemPhase| {
        let mut remaining = Vec::new();
        for (entry_phase, system) in extra.drain(..) {
            if entry_phase == phase {
                systems.push(system);
            } else {
                remaining.push((entry_phase, system));
            }
        }
        extra = remaining;
    };

    push_phase(&mut systems, UiSystemPhase::Interaction);
    systems.extend([
        ui_system(crate::systems::widgets::ui_shortcut_system),
        ui_system(crate::systems::widgets::ui_button_system),
        ui_system(crate::systems::widgets::ui_canvas_system),
        ui_system(crate::systems::widgets::ui_context_menu_system),
        ui_system(crate::systems::widgets::ui_drag_value_system),
        ui_system(crate::systems::widgets::ui_tile_container_system),
        ui_system(crate::systems::widgets::ui_tab_bar_system),
        ui_system(crate::systems::widgets::ui_dropdown_system),
        ui_system(crate::systems::widgets::ui_menu_system),
        ui_system(crate::systems::widgets::ui_multi_select_system),
        ui_system(crate::systems::widgets::ui_date_picker_system),
        ui_system(crate::systems::widgets::ui_panel_system),
        ui_system(crate::systems::widgets::ui_color_wheel_system),
        ui_system(crate::systems::widgets::ui_color_picker_system),
        ui_system(crate::systems::widgets::ui_data_grid_system),
        ui_system(crate::systems::widgets::ui_property_grid_system),
        ui_system(crate::systems::widgets::ui_command_palette_system),
        ui_system(crate::systems::widgets::ui_toggle_system),
        ui_system(crate::systems::widgets::ui_checkbox_system),
        ui_system(crate::systems::widgets::ui_radio_system),
        ui_system(crate::systems::widgets::ui_collapsing_header_system),
        ui_system(crate::systems::widgets::ui_spinner_system),
        ui_system(crate::systems::widgets::ui_scroll_area_system),
        ui_system(crate::systems::widgets::ui_virtual_list_system),
        ui_system(crate::systems::widgets::ui_tree_view_system),
        ui_system(crate::systems::widgets::ui_modal_dialog_system),
        ui_system(crate::systems::widgets::ui_slider_system),
        ui_system(crate::systems::widgets::ui_range_slider_system),
        ui_system(crate::systems::widgets::ui_splitter_system),
        ui_system(crate::systems::widgets::ui_breadcrumb_system),
        ui_system(crate::systems::widgets::ui_text_area_system),
        ui_system(crate::systems::widgets::ui_rich_text_editor_system),
        ui_system(crate::systems::widgets::ui_text_input_system),
        ui_system(crate::systems::widgets::ui_selectable_label_system),
        ui_system(crate::systems::widgets::ui_popup_dismiss_system),
        ui_system(crate::systems::widgets::ui_error_border_system),
        ui_system(crate::systems::widgets::ui_event_bubble_system),
        ui_system(crate::systems::widgets::ui_emit_click_events_system),
    ]);

    push_phase(&mut systems, UiSystemPhase::PreLayout);
    systems.extend([
        ui_system(crate::systems::ui_layout_state_update_system),
        ui_system(crate::systems::ui_docked_panel_layout_system),
        ui_system(crate::systems::ui_responsive_apply_system),
        ui_system(crate::systems::ui_layout_compute_system),
        ui_system(crate::systems::ui_theme_transition_tick_system),
        ui_system(crate::systems::ui_theme_apply_system),
        ui_system(crate::systems::ui_layout_color_blend_system),
        ui_system(crate::systems::ui_layout_transform_blend_system),
    ]);

    push_phase(&mut systems, UiSystemPhase::PreDraw);
    systems.push(ui_system(
        crate::systems::render_sync::ui_layout_render_sync_system,
    ));
    systems
}

pub fn default_ui_systems() -> Vec<UiSystemFn> {
    ui_systems_with(Vec::new())
}
