use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::data::icons::material;
use nightshade_ui::prelude::*;

use crate::commands::refresh::{
    apply_visibility, highlight, mark_tree_selection, move_nav_gizmo, open_notice, size_layer_panel,
};
use crate::data::{BrowserTab, PanelTab};
use crate::queries::browsers::{browser_tab_index, page_count};
use crate::queries::interface::{gizmo_index, stat_strings};
use crate::resources::ViewerResources;
use crate::theme::{PANEL_MARGIN, SELECTED_ROW_COLOR};

pub fn refresh_ui(mut viewer: ResMut<ViewerResources>, time: Res<Time>, mut build: UiBuild<'_>) {
    let pages = viewer
        .view
        .browser
        .map(|tab| page_count(&viewer, tab, &viewer.view.search.to_lowercase()));
    let ViewerResources {
        handles,
        view,
        selection,
        animation,
        model,
        incoming,
        camera_input,
        gizmo: viewer_gizmo,
        layout,
        ..
    } = &mut *viewer;
    let layout = *layout;
    let mut context = build.context();

    let fps = time.frames_per_second.round() as u32;
    if view.applied.fps != Some(fps) {
        view.applied.fps = Some(fps);
        ui_set_text(context.commands, handles.fps_label, &format!("{fps} fps"));
    }

    apply_visibility(
        &mut context,
        &mut view.applied.interface_visible,
        handles.interface,
        view.interface_visible,
    );
    apply_visibility(
        &mut context,
        &mut view.applied.scene_open,
        handles.left_panel,
        view.scene_open,
    );
    apply_visibility(
        &mut context,
        &mut view.applied.inspector_open,
        handles.inspector_panel,
        view.inspector_open,
    );
    apply_visibility(
        &mut context,
        &mut view.applied.add_open,
        handles.add_panel,
        view.add_open,
    );
    apply_visibility(
        &mut context,
        &mut view.applied.recent_open,
        handles.recent_panel,
        view.recent_open,
    );
    if view.applied.gizmo_shifted != Some(view.inspector_open) {
        view.applied.gizmo_shifted = Some(view.inspector_open);
        let offset = if view.inspector_open {
            -(PANEL_MARGIN + layout.panel_width + PANEL_MARGIN)
        } else {
            -PANEL_MARGIN
        };
        move_nav_gizmo(&mut context, handles.nav_holder, offset, layout);
    }
    apply_visibility(
        &mut context,
        &mut view.applied.browser_open,
        handles.browser_panel,
        view.browser.is_some(),
    );
    apply_visibility(
        &mut context,
        &mut view.applied.library_strip,
        handles.library_strip,
        view.browser == Some(BrowserTab::Library),
    );
    apply_visibility(
        &mut context,
        &mut view.applied.animation_visible,
        handles.animation_bar,
        !model.clips.is_empty(),
    );
    apply_visibility(
        &mut context,
        &mut view.applied.layers_visible,
        handles.layer_panel,
        !animation.layers.is_empty(),
    );
    apply_visibility(
        &mut context,
        &mut view.applied.inspector_body,
        handles.inspector_body,
        selection.selected.is_some(),
    );

    apply_visibility(
        &mut context,
        &mut view.applied.show_button,
        handles.show_button,
        !view.interface_visible,
    );

    let tab_index = match view.panel_tab {
        PanelTab::Scene => 0,
        PanelTab::Render => 1,
        PanelTab::Stats => 2,
    };
    if view.applied.panel_tab != Some(tab_index) {
        view.applied.panel_tab = Some(tab_index);
        for (index, root) in handles.tab_roots.iter().enumerate() {
            ui_set_visible(context.dirty, context.commands, *root, index == tab_index);
        }
        for (index, button) in handles.tab_buttons.iter().enumerate() {
            let color = if index == tab_index {
                SELECTED_ROW_COLOR
            } else {
                vec4(0.0, 0.0, 0.0, 0.0)
            };
            ui_set_color::<UiBase>(context.commands, *button, color);
        }
    }

    if view.applied.grid != Some(view.grid) {
        view.applied.grid = Some(view.grid);
        highlight(&mut context, handles.grid_button, view.grid);
    }
    if view.applied.snap != Some(view.snap) {
        view.applied.snap = Some(view.snap);
        highlight(&mut context, handles.snap_button, view.snap);
    }
    if view.applied.turntable != Some(camera_input.turntable) {
        view.applied.turntable = Some(camera_input.turntable);
        highlight(&mut context, handles.spin_button, camera_input.turntable);
    }
    let gizmo = gizmo_index(viewer_gizmo);
    if view.applied.gizmo_mode != Some(gizmo) {
        view.applied.gizmo_mode = Some(gizmo);
        for (index, button) in handles.gizmo_buttons.iter().enumerate() {
            highlight(&mut context, *button, Some(index) == gizmo);
        }
    }

    if view.applied.looping != Some(animation.looping) {
        view.applied.looping = Some(animation.looping);
        highlight(&mut context, handles.loop_button, animation.looping);
    }

    let browser_tab = view.browser.map(browser_tab_index);
    if let Some(index) = browser_tab
        && view.applied.browser_tab != Some(index)
    {
        view.applied.browser_tab = Some(index);
        for (position, button) in handles.browser_tabs.iter().enumerate() {
            highlight(&mut context, *button, position == index);
        }
    }

    if view.applied.selected != Some(selection.selected) {
        view.applied.selected = Some(selection.selected);
        let chosen: Vec<Entity> = handles
            .tree_rows
            .iter()
            .filter(|row| {
                selection.selected == Some(row.target) || selection.others.contains(&row.target)
            })
            .map(|row| row.node)
            .collect();
        let nodes: Vec<Entity> = handles.tree_rows.iter().map(|row| row.node).collect();
        mark_tree_selection(&mut context, handles.tree_view, nodes, chosen);
        let name = if selection.selected.is_some() {
            selection.name.clone()
        } else {
            String::new()
        };
        ui_text_input_set_value(context.commands, handles.inspector_name, &name);
        ui_set_text(
            context.commands,
            handles.inspector_mesh,
            selection.mesh.as_deref().unwrap_or(""),
        );
        ui_set_visible(
            context.dirty,
            context.commands,
            handles.inspector_empty,
            selection.selected.is_none(),
        );
    }

    if view.applied.selected_visible != Some(selection.visible) {
        view.applied.selected_visible = Some(selection.visible);
        let label = if selection.visible { "Hide" } else { "Show" };
        ui_button_set_text(context.commands, handles.visible_button, label);
    }
    let isolated = model.isolated.is_some();
    if view.applied.isolated != Some(isolated) {
        view.applied.isolated = Some(isolated);
        let label = if isolated { "Show all" } else { "Isolate" };
        ui_button_set_text(context.commands, handles.isolate_button, label);
        highlight(&mut context, handles.isolate_button, isolated);
    }

    if std::mem::take(&mut view.tree_filter_dirty) {
        let filter = view.tree_filter.clone();
        let view_entity = handles.tree_view;
        context.commands.queue(move |storage, _| {
            if let Some(data) =
                storage.get_mut::<nightshade_ui::components::UiTreeViewData>(view_entity)
            {
                data.filter_active = !filter.is_empty();
                data.filter_text = filter.clone();
            }
        });
    }

    if std::mem::take(&mut view.tree_visibility_dirty) {
        for row in &handles.tree_rows {
            let visible = model
                .rows
                .iter()
                .find(|scene| scene.entity == row.target)
                .is_none_or(|scene| scene.visible);
            let mut buffer = [0u8; 4];
            let glyph = if visible {
                material::VISIBILITY
            } else {
                material::VISIBILITY_OFF
            };
            ui_set_text(
                context.commands,
                row.eye,
                glyph.codepoint.encode_utf8(&mut buffer),
            );
        }
    }

    if std::mem::take(&mut view.inspector_dirty) {
        for (axis, field) in handles.translation.iter().enumerate() {
            ui_drag_value_set_value(context.commands, *field, selection.translation[axis]);
        }
        for (axis, field) in handles.rotation.iter().enumerate() {
            ui_drag_value_set_value(context.commands, *field, selection.rotation[axis]);
        }
        for (axis, field) in handles.scale.iter().enumerate() {
            ui_drag_value_set_value(context.commands, *field, selection.scale[axis]);
        }
    }

    if view.applied.layer_count != Some(animation.layers.len()) {
        view.applied.layer_count = Some(animation.layers.len());
        size_layer_panel(
            &mut context,
            handles.layer_panel,
            animation.layers.len(),
            layout,
        );
    }

    if view.applied.clip != Some(animation.clip)
        && let Some(dropdown) = handles.clip_dropdown
    {
        view.applied.clip = Some(animation.clip);
        if let Some(index) = animation.clip {
            ui_dropdown_set_value(context.commands, dropdown, index);
        }
    }

    if view.applied.blend_time != Some(animation.blend_time) {
        view.applied.blend_time = Some(animation.blend_time);
        ui_drag_value_set_value(context.commands, handles.blend_drag, animation.blend_time);
    }

    if view.applied.playing != Some(animation.playing) {
        view.applied.playing = Some(animation.playing);
        let label = if animation.playing { "Pause" } else { "Play" };
        ui_button_set_text(context.commands, handles.play_button, label);
    }

    if !model.clips.is_empty() {
        let shown = ((animation.time * 10.0).round() / 10.0, animation.duration);
        if view.applied.time != Some(shown) {
            view.applied.time = Some(shown);
            let normalized = if animation.duration > 0.0 {
                animation.time / animation.duration
            } else {
                0.0
            };
            ui_slider_set_value(context.commands, handles.seek_slider, normalized);
            let time_text = format!("{:.1} / {:.1} s", animation.time, animation.duration);
            ui_set_text(context.commands, handles.time_label, &time_text);
        }
    }

    if let Some(pages) = pages {
        let shown = (view.browser_page, pages);
        if view.applied.browser_page != Some(shown) {
            view.applied.browser_page = Some(shown);
            ui_set_text(
                context.commands,
                handles.browser_page_label,
                &format!("{} / {pages}", view.browser_page + 1),
            );
        }
    }

    let loading = incoming.loading.lock().ok().and_then(|guard| guard.clone());
    if view.applied.loading.as_ref() != Some(&loading) {
        view.applied.loading = Some(loading.clone());
        match &loading {
            Some(label) => {
                ui_set_text(
                    context.commands,
                    handles.loading_label,
                    &format!("Loading {label}"),
                );
                ui_set_visible(context.dirty, context.commands, handles.loading_label, true);
            }
            None => {
                ui_set_visible(
                    context.dirty,
                    context.commands,
                    handles.loading_label,
                    false,
                );
            }
        }
    }

    let notice = incoming.notice.lock().ok().and_then(|guard| guard.clone());
    if view.applied.notice.as_ref() != Some(&notice) {
        view.applied.notice = Some(notice.clone());
        if let Some(message) = &notice {
            ui_set_text(context.commands, handles.notice_text, message);
            open_notice(&mut context, handles.notice_dialog, handles.notice_backdrop);
        }
    }

    let stats = stat_strings(model.stats.as_ref());
    if view.applied.stats.as_ref() != Some(&stats) {
        view.applied.stats = Some(stats.clone());
        for (index, value) in stats.iter().enumerate() {
            ui_set_text(context.commands, handles.stat_values[index], value);
        }
    }
}
