use nightshade::prelude::*;

use crate::data::{
    BROWSER_TABS, BrowserItem, BrowserTab, CameraField, GIZMO_MODES, LIGHTS, LightField,
    MaterialField, PRIMITIVES, RESOLUTION_VALUES, TABS, ViewerAction,
};
use crate::resources::ViewerResources;

pub fn on_click(viewer: &ViewerResources, entity: Entity) -> Vec<ViewerAction> {
    let mut actions = Vec::new();
    let handles = &viewer.handles;

    if entity == handles.scene_button {
        actions.push(ViewerAction::ToggleScenePanel);
    } else if entity == handles.inspect_button {
        actions.push(ViewerAction::ToggleInspectorPanel);
    } else if entity == handles.add_button {
        actions.push(ViewerAction::OpenAddMenu);
    } else if entity == handles.add_close {
        actions.push(ViewerAction::CloseAddMenu);
    } else if entity == handles.grid_button {
        actions.push(ViewerAction::SetGrid(!viewer.view.grid));
    } else if entity == handles.snap_button {
        actions.push(ViewerAction::SetSnap(!viewer.view.snap));
    } else if entity == handles.spin_button {
        actions.push(ViewerAction::SetTurntable(!viewer.camera_input.turntable));
    } else if entity == handles.frame_button {
        actions.push(ViewerAction::FrameModel);
    } else if entity == handles.shot_button {
        actions.push(ViewerAction::Screenshot);
    } else if entity == handles.save_button {
        actions.push(ViewerAction::ExportModel);
    } else if entity == handles.clips_button {
        actions.push(ViewerAction::ApplyHeldClips);
    } else if entity == handles.recent_button {
        actions.push(ViewerAction::OpenRecent);
    } else if entity == handles.recent_close {
        actions.push(ViewerAction::CloseRecent);
    } else if entity == handles.undo_button {
        actions.push(ViewerAction::Undo);
    } else if entity == handles.redo_button {
        actions.push(ViewerAction::Redo);
    } else if entity == handles.visible_button {
        actions.push(ViewerAction::ToggleSelectedVisible);
    } else if entity == handles.isolate_button {
        actions.push(ViewerAction::ToggleIsolate);
    } else if entity == handles.browse_button {
        actions.push(ViewerAction::OpenBrowser(BrowserTab::Library));
    } else if entity == handles.library_add_folder {
        actions.push(ViewerAction::AddLibraryFolder);
    } else if entity == handles.library_add_files {
        actions.push(ViewerAction::AddLibraryFiles);
    } else if entity == handles.library_save {
        actions.push(ViewerAction::ExportModel);
    } else if entity == handles.add_import {
        actions.push(ViewerAction::ImportFile);
    } else if entity == handles.browser_refresh {
        actions.push(ViewerAction::RefreshLibrary);
    } else if entity == handles.browser_close {
        actions.push(ViewerAction::CloseBrowser);
    } else if entity == handles.browser_previous {
        actions.push(ViewerAction::SetBrowserPage(
            viewer.view.browser_page.saturating_sub(1),
        ));
    } else if entity == handles.browser_next {
        actions.push(ViewerAction::SetBrowserPage(viewer.view.browser_page + 1));
    } else if entity == handles.random_model_button {
        actions.push(ViewerAction::RandomModel);
    } else if entity == handles.random_sky_button {
        actions.push(ViewerAction::RandomSky);
    } else if entity == handles.play_button {
        actions.push(ViewerAction::TogglePlayback);
    } else if entity == handles.stop_button {
        actions.push(ViewerAction::StopAnimation);
    } else if entity == handles.loop_button {
        actions.push(ViewerAction::SetAnimationLoop(!viewer.animation.looping));
    } else if entity == handles.show_button {
        actions.push(ViewerAction::ShowInterface);
    } else if entity == handles.add_layer_button {
        if let Some(clip) = next_layer_clip(viewer) {
            actions.push(ViewerAction::AddAnimationLayer { clip, weight: 0.5 });
        }
    } else if entity == handles.material.reset {
        actions.push(ViewerAction::ResetMaterial);
    }

    for (index, mode) in handles.gizmo_buttons.iter().enumerate() {
        if entity == *mode {
            actions.push(ViewerAction::SetGizmoMode(GIZMO_MODES[index]));
        }
    }

    for (index, tab) in handles.tab_buttons.iter().enumerate() {
        if entity == *tab {
            actions.push(ViewerAction::SelectPanelTab(TABS[index]));
        }
    }

    for (index, tab) in handles.browser_tabs.iter().enumerate() {
        if entity == *tab {
            actions.push(ViewerAction::SetBrowserTab(BROWSER_TABS[index]));
        }
    }

    for (index, button) in handles.primitive_buttons.iter().enumerate() {
        if entity == *button {
            actions.push(ViewerAction::AddPrimitive(PRIMITIVES[index]));
        }
    }

    for (index, button) in handles.light_buttons.iter().enumerate() {
        if entity == *button {
            actions.push(ViewerAction::AddLight(LIGHTS[index]));
        }
    }

    for row in &handles.layer_rows {
        if entity == row.remove {
            actions.push(ViewerAction::RemoveAnimationLayer(row.index));
        }
    }

    for row in &handles.tree_rows {
        if entity == row.eye {
            let visible = viewer
                .model
                .rows
                .iter()
                .find(|scene| scene.entity == row.target)
                .is_none_or(|scene| scene.visible);
            actions.push(ViewerAction::SetNodeVisible {
                entity: row.target,
                visible: !visible,
            });
        }
    }

    for row in &handles.material.texture_rows {
        if entity == row.replace {
            actions.push(ViewerAction::ReplaceTexture(row.slot));
        }
    }

    for (button, path) in &handles.recent_buttons {
        if entity == *button {
            actions.push(ViewerAction::LoadRecent(path.clone()));
        }
    }

    for cell in &handles.browser_cells {
        if entity == cell.button {
            actions.push(match &cell.item {
                BrowserItem::Khronos(name) => ViewerAction::LoadKhronos(name.clone()),
                BrowserItem::Model(slug) => ViewerAction::LoadPolyhavenModel(slug.clone()),
                BrowserItem::Sky(slug) => ViewerAction::LoadPolyhavenSky(slug.clone()),
                BrowserItem::File(path) => ViewerAction::OpenLibraryFile(path.clone()),
            });
        }
    }

    for (button, path) in &handles.library_forget {
        if entity == *button {
            actions.push(ViewerAction::ForgetLibraryPath(path.clone()));
        }
    }

    actions
}

fn next_layer_clip(viewer: &ViewerResources) -> Option<usize> {
    (0..viewer.model.clips.len()).find(|index| {
        Some(*index) != viewer.animation.clip
            && viewer
                .animation
                .layers
                .iter()
                .all(|layer| layer.clip != *index)
    })
}

pub fn on_color(viewer: &ViewerResources, entity: Entity, color: Vec4) -> Vec<ViewerAction> {
    let mut actions = Vec::new();
    let handles = &viewer.handles;
    if entity == handles.material.base_color {
        actions.push(ViewerAction::SetMaterialField(MaterialField::BaseColor([
            color.x, color.y, color.z, color.w,
        ])));
    } else if entity == handles.material.emissive {
        actions.push(ViewerAction::SetMaterialField(MaterialField::Emissive([
            color.x, color.y, color.z,
        ])));
    } else if entity == handles.light.color {
        actions.push(ViewerAction::SetLightField(LightField::Color([
            color.x, color.y, color.z,
        ])));
    } else if entity == handles.fog_color {
        actions.push(ViewerAction::SetFogColor([color.x, color.y, color.z]));
    }

    actions
}

pub fn on_slider(viewer: &ViewerResources, entity: Entity, value: f32) -> Vec<ViewerAction> {
    let mut actions = Vec::new();
    let handles = &viewer.handles;
    if entity == handles.exposure_slider {
        actions.push(ViewerAction::SetExposure(value));
        return actions;
    }
    if entity == handles.sun_intensity {
        actions.push(ViewerAction::SetSunIntensity(value));
        return actions;
    }
    if entity == handles.render_scale_slider {
        actions.push(ViewerAction::SetRenderScale(value));
        return actions;
    }
    if entity == handles.seek_slider {
        actions.push(ViewerAction::SeekAnimation(
            value * viewer.animation.duration,
        ));
        return actions;
    }
    if entity == handles.light.inner_cone {
        actions.push(ViewerAction::SetLightField(LightField::InnerCone(value)));
        return actions;
    }
    if entity == handles.light.outer_cone {
        actions.push(ViewerAction::SetLightField(LightField::OuterCone(value)));
        return actions;
    }
    for (control_entity, control) in &handles.material.sliders {
        if entity == *control_entity {
            actions.push(ViewerAction::SetMaterialField(control.field(value)));
            return actions;
        }
    }
    for row in &handles.layer_rows {
        if entity == row.slider {
            actions.push(ViewerAction::SetLayerWeight {
                index: row.index,
                weight: value,
            });
            return actions;
        }
    }
    for slider in &handles.morph_sliders {
        if entity == slider.slider {
            actions.push(ViewerAction::SetMorphWeight {
                entity: slider.target,
                index: slider.index,
                weight: value,
            });
        }
    }

    actions
}

pub fn on_drag_value(viewer: &ViewerResources, entity: Entity, value: f32) -> Vec<ViewerAction> {
    let mut actions = Vec::new();
    let handles = &viewer.handles;
    if entity == handles.speed_drag {
        actions.push(ViewerAction::SetAnimationSpeed(value));
        return actions;
    }
    if entity == handles.blend_drag {
        actions.push(ViewerAction::SetBlendTime(value));
        return actions;
    }
    if entity == handles.sun_yaw {
        actions.push(ViewerAction::SetSunYaw(value));
        return actions;
    }
    if entity == handles.sun_pitch {
        actions.push(ViewerAction::SetSunPitch(value));
        return actions;
    }
    if entity == handles.fog_start {
        actions.push(ViewerAction::SetFogStart(value));
        return actions;
    }
    if entity == handles.fog_end {
        actions.push(ViewerAction::SetFogEnd(value));
        return actions;
    }
    if entity == handles.focus_distance {
        actions.push(ViewerAction::SetFocusDistance(value));
        return actions;
    }
    if entity == handles.focus_range {
        actions.push(ViewerAction::SetFocusRange(value));
        return actions;
    }
    if entity == handles.light.intensity {
        actions.push(ViewerAction::SetLightField(LightField::Intensity(value)));
        return actions;
    }
    if entity == handles.light.range {
        actions.push(ViewerAction::SetLightField(LightField::Range(value)));
        return actions;
    }
    if entity == handles.camera_fields.field_of_view {
        actions.push(ViewerAction::SetCameraField(CameraField::FieldOfView(
            value,
        )));
        return actions;
    }
    if entity == handles.camera_fields.near {
        actions.push(ViewerAction::SetCameraField(CameraField::Near(value)));
        return actions;
    }
    if entity == handles.camera_fields.far {
        actions.push(ViewerAction::SetCameraField(CameraField::Far(value)));
        return actions;
    }
    for (axis, field) in handles.translation.iter().enumerate() {
        if entity == *field {
            let mut values = current_vector(viewer, VectorField::Translation);
            values[axis] = value;
            actions.push(ViewerAction::SetTranslation(values));
        }
    }
    for (axis, field) in handles.rotation.iter().enumerate() {
        if entity == *field {
            let mut values = current_vector(viewer, VectorField::Rotation);
            values[axis] = value;
            actions.push(ViewerAction::SetRotation(values));
        }
    }
    for (axis, field) in handles.scale.iter().enumerate() {
        if entity == *field {
            let mut values = current_vector(viewer, VectorField::Scale);
            values[axis] = value;
            actions.push(ViewerAction::SetScale(values));
        }
    }

    actions
}

enum VectorField {
    Translation,
    Rotation,
    Scale,
}

fn current_vector(viewer: &ViewerResources, field: VectorField) -> [f32; 3] {
    match field {
        VectorField::Translation => viewer.selection.translation,
        VectorField::Rotation => viewer.selection.rotation,
        VectorField::Scale => viewer.selection.scale,
    }
}

pub fn on_toggle(viewer: &ViewerResources, entity: Entity, value: bool) -> Vec<ViewerAction> {
    let mut actions = Vec::new();
    let handles = &viewer.handles;
    if entity == handles.normals_toggle {
        actions.push(ViewerAction::SetShowNormals(value));
    } else if entity == handles.bounds_toggle {
        actions.push(ViewerAction::SetShowBounds(value));
    } else if entity == handles.skeleton_toggle {
        actions.push(ViewerAction::SetShowSkeleton(value));
    } else if entity == handles.sky_toggle {
        actions.push(ViewerAction::SetShowSky(value));
    } else if entity == handles.sun_shadows {
        actions.push(ViewerAction::SetSunShadows(value));
    } else if entity == handles.ssao_toggle {
        actions.push(ViewerAction::SetSsao(value));
    } else if entity == handles.bloom_toggle {
        actions.push(ViewerAction::SetBloom(value));
    } else if entity == handles.ssr_toggle {
        actions.push(ViewerAction::SetSsr(value));
    } else if entity == handles.ssgi_toggle {
        actions.push(ViewerAction::SetSsgi(value));
    } else if entity == handles.fog_toggle {
        actions.push(ViewerAction::SetFog(value));
    } else if entity == handles.depth_of_field_toggle {
        actions.push(ViewerAction::SetDepthOfField(value));
    } else if entity == handles.material.unlit {
        actions.push(ViewerAction::SetMaterialField(MaterialField::Unlit(value)));
    } else if entity == handles.material.double_sided {
        actions.push(ViewerAction::SetMaterialField(MaterialField::DoubleSided(
            value,
        )));
    } else if entity == handles.light.cast_shadows {
        actions.push(ViewerAction::SetLightField(LightField::CastShadows(value)));
    }

    actions
}

pub fn on_dropdown(viewer: &ViewerResources, entity: Entity, index: usize) -> Vec<ViewerAction> {
    let mut actions = Vec::new();
    let handles = &viewer.handles;
    if entity == handles.shading_dropdown {
        actions.push(ViewerAction::SetShadingMode(index));
    } else if entity == handles.pbr_dropdown {
        actions.push(ViewerAction::SetPbrDebug(index));
    } else if entity == handles.tonemap_dropdown {
        actions.push(ViewerAction::SetTonemap(index));
    } else if entity == handles.atmosphere_dropdown {
        actions.push(ViewerAction::SetAtmosphere(index));
    } else if entity == handles.anti_aliasing_dropdown {
        actions.push(ViewerAction::SetAntiAliasing(index));
    } else if entity == handles.material.alpha_mode {
        actions.push(ViewerAction::SetMaterialField(MaterialField::AlphaMode(
            index,
        )));
    } else if entity == handles.browser_resolution {
        let value = RESOLUTION_VALUES
            .get(index)
            .copied()
            .unwrap_or(RESOLUTION_VALUES[1]);
        actions.push(ViewerAction::SetResolution(value));
    } else if Some(entity) == handles.camera_dropdown {
        let camera = index
            .checked_sub(1)
            .and_then(|position| viewer.model.cameras.get(position))
            .map(|(entity, _)| *entity);
        actions.push(ViewerAction::SetActiveCamera(camera));
    } else if Some(entity) == handles.clip_dropdown {
        actions.push(ViewerAction::BlendToClip(index));
    } else if Some(entity) == handles.variant_dropdown {
        let name = if index == 0 {
            None
        } else {
            viewer.model.variants.get(index - 1).cloned()
        };
        actions.push(ViewerAction::SetVariant(name));
    }

    actions
}
