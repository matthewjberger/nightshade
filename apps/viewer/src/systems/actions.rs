use nightshade_graphics::prelude::Light;
use nightshade_render::components::morph::MorphWeights;
use nightshade_render::data::config::AntiAliasing;

use crate::commands::animation::{
    add_animation_layer, apply_held_clips, blend_to_clip, toggle_playback, with_players,
};
use crate::commands::browsers::{
    fetch_khronos, fetch_polyhaven_model, fetch_polyhaven_sky, refresh_indices,
};
use crate::commands::camera::snap_axis;
use crate::commands::files::{
    export_dialog, open_dialog, open_kept_path, pick_texture, screenshot,
};
use crate::commands::history::{redo, undo};
use crate::commands::library::{add_files, add_folder, forget_path, start_scan};
use crate::commands::material::{reset_material, set_material_field};
use crate::commands::render::{point_sun, set_atmosphere, set_shading_mode, write_fog};
use crate::commands::scene::{
    apply_variant, delete_selected, rename, set_node_visible, toggle_isolate,
    toggle_selected_visible,
};
use crate::commands::selection::{
    apply_transform, select_entity, set_camera_field, set_light_field, toggle_in_selection,
};
use crate::commands::spawn::{add_light_at_focus, add_primitive_at_focus};
use crate::data::{BrowserTab, PanelTab, ViewerAction};
use crate::queries::browsers::{page_count, random_slug};
use crate::queries::render::{fog_of, pbr_mode, tonemap};

pub fn apply_actions(
    mut stage: crate::resources::Stage,
    mut commands: nightshade::prelude::Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let actions = std::mem::take(&mut stage.viewer.actions);
    if actions.is_empty() {
        return;
    }
    for action in actions {
        apply_action(&mut stage, storage, &mut commands, action);
    }
}

fn apply_action(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    commands: &mut nightshade::prelude::Commands,
    action: ViewerAction,
) {
    let active_camera = stage.active_camera.0;

    match action {
        ViewerAction::ToggleScenePanel => {
            let viewer = &mut *stage.viewer;
            let narrow = viewer.layout.narrow;
            viewer.view.scene_open = !viewer.view.scene_open;
            if narrow && viewer.view.scene_open {
                viewer.view.inspector_open = false;
            }
        }
        ViewerAction::ToggleInspectorPanel => {
            let viewer = &mut *stage.viewer;
            let narrow = viewer.layout.narrow;
            viewer.view.inspector_open = !viewer.view.inspector_open;
            if narrow && viewer.view.inspector_open {
                viewer.view.scene_open = false;
            }
        }
        ViewerAction::OpenAddMenu => {
            stage.viewer.view.add_open = true;
        }
        ViewerAction::CloseAddMenu => {
            stage.viewer.view.add_open = false;
        }
        ViewerAction::SetGrid(enabled) => {
            stage.debug_draw.show_grid = enabled;
            stage.viewer.view.grid = enabled;
        }
        ViewerAction::SetSnap(enabled) => {
            stage.viewer.view.snap = enabled;
        }
        ViewerAction::SetTurntable(enabled) => {
            stage.viewer.camera_input.turntable = enabled;
        }
        ViewerAction::FrameModel => {
            stage.viewer.camera_input.frame_requested = true;
        }
        ViewerAction::OpenBrowser(tab) | ViewerAction::SetBrowserTab(tab) => {
            let viewer = &mut *stage.viewer;
            let changed = viewer.view.browser != Some(tab);
            viewer.view.browser = Some(tab);
            viewer.view.browser_page = 0;
            viewer.view.search.clear();
            viewer.view.browser_dirty = true;
            viewer.browsers.reset_thumbnails |= changed;
        }
        ViewerAction::SetBrowserPage(page) => {
            let viewer = &mut *stage.viewer;
            let last = viewer
                .view
                .browser
                .map(|tab| page_count(viewer, tab, &viewer.view.search.to_lowercase()))
                .unwrap_or(1)
                - 1;
            viewer.view.browser_page = page.min(last);
            viewer.view.browser_dirty = true;
        }
        ViewerAction::ImportFile => {
            let viewer = &mut *stage.viewer;
            viewer.view.add_open = false;
            open_dialog(viewer);
        }
        ViewerAction::OpenRecent => {
            stage.viewer.view.recent_open = true;
        }
        ViewerAction::CloseRecent => {
            stage.viewer.view.recent_open = false;
        }
        ViewerAction::LoadRecent(path) => {
            stage.viewer.view.recent_open = false;
            open_kept_path(stage, &path);
        }
        ViewerAction::RefreshLibrary => {
            let viewer = &mut *stage.viewer;
            if viewer.view.browser == Some(BrowserTab::Library) {
                start_scan(viewer);
            } else {
                refresh_indices(viewer);
            }
        }
        ViewerAction::AddLibraryFolder => add_folder(stage.viewer),
        ViewerAction::AddLibraryFiles => add_files(stage.viewer),
        ViewerAction::ForgetLibraryPath(path) => forget_path(stage.viewer, &path),
        ViewerAction::OpenLibraryFile(path) => {
            stage.viewer.view.browser = None;
            stage.viewer.view.browser_dirty = true;
            open_kept_path(stage, &path);
        }
        ViewerAction::CloseBrowser => {
            let viewer = &mut *stage.viewer;
            viewer.view.browser = None;
            viewer.view.browser_dirty = true;
        }
        ViewerAction::SetResolution(value) => {
            stage.viewer.view.resolution = value;
        }
        ViewerAction::SelectPanelTab(tab) => {
            stage.viewer.view.panel_tab = tab;
            if tab == PanelTab::Scene {
                stage.viewer.view.scene_open = true;
            }
        }
        ViewerAction::RandomModel => {
            let viewer = &mut *stage.viewer;
            if let Some(slug) = random_slug(&viewer.browsers.models) {
                fetch_polyhaven_model(viewer, &slug, viewer.view.resolution);
            }
        }
        ViewerAction::RandomSky => {
            let viewer = &mut *stage.viewer;
            if let Some(slug) = random_slug(&viewer.browsers.skies) {
                fetch_polyhaven_sky(viewer, &slug, 4);
            }
        }
        ViewerAction::LoadKhronos(name) => {
            let viewer = &mut *stage.viewer;
            fetch_khronos(viewer, &name);
            stage.viewer.view.browser = None;
        }
        ViewerAction::LoadPolyhavenModel(slug) => {
            let viewer = &mut *stage.viewer;
            fetch_polyhaven_model(viewer, &slug, viewer.view.resolution);
            stage.viewer.view.browser = None;
        }
        ViewerAction::LoadPolyhavenSky(slug) => {
            let viewer = &mut *stage.viewer;
            fetch_polyhaven_sky(viewer, &slug, viewer.view.resolution);
            stage.viewer.view.browser = None;
        }
        ViewerAction::Select(entity) => select_entity(stage, storage, entity),
        ViewerAction::ToggleInSelection(entity) => toggle_in_selection(stage, storage, entity),
        ViewerAction::SetMaterialField(field) => set_material_field(stage, field),
        ViewerAction::ResetMaterial => reset_material(stage),
        ViewerAction::ReplaceTexture(slot) => pick_texture(stage.viewer, slot),
        ViewerAction::SetLightField(field) => set_light_field(stage.viewer, storage, field),
        ViewerAction::SetCameraField(field) => set_camera_field(stage.viewer, storage, field),
        ViewerAction::DeleteSelected => delete_selected(stage, storage),
        ViewerAction::SetTranslation(values) => {
            let viewer = &mut *stage.viewer;
            viewer.selection.translation = values;
            apply_transform(viewer, storage);
        }
        ViewerAction::SetRotation(values) => {
            let viewer = &mut *stage.viewer;
            viewer.selection.rotation = values;
            apply_transform(viewer, storage);
        }
        ViewerAction::SetScale(values) => {
            let viewer = &mut *stage.viewer;
            viewer.selection.scale = values;
            apply_transform(viewer, storage);
        }
        ViewerAction::Rename(name) => rename(stage.viewer, storage, name),
        ViewerAction::SetNodeVisible { entity, visible } => {
            set_node_visible(stage, storage, entity, visible)
        }
        ViewerAction::ToggleSelectedVisible => toggle_selected_visible(stage, storage),
        ViewerAction::ToggleIsolate => toggle_isolate(stage, storage),
        ViewerAction::AddPrimitive(kind) => {
            add_primitive_at_focus(stage, storage, commands, active_camera, kind)
        }
        ViewerAction::AddLight(kind) => {
            add_light_at_focus(stage, storage, commands, active_camera, kind)
        }
        ViewerAction::SetShadingMode(index) => {
            set_shading_mode(stage, storage, active_camera, index)
        }
        ViewerAction::SetPbrDebug(index) => {
            stage.viewer.view.pbr_debug = index;
            stage.debug_draw.pbr_debug_mode = pbr_mode(index);
        }
        ViewerAction::SetShowNormals(enabled) => {
            stage.viewer.view.show_normals = enabled;
            stage.debug_draw.show_normals = enabled;
        }
        ViewerAction::SetShowBounds(enabled) => {
            stage.viewer.view.show_bounds = enabled;
            stage.debug_draw.show_bounding_volumes = enabled;
        }
        ViewerAction::SetShowSkeleton(enabled) => {
            stage.viewer.skeleton.enabled = enabled;
        }
        ViewerAction::SetShowSky(show) => {
            stage.viewer.view.show_sky = show;
            stage.settings.show_sky = show;
        }
        ViewerAction::SetAtmosphere(index) => set_atmosphere(stage, index),
        ViewerAction::SetExposure(exposure) => {
            stage.viewer.view.exposure = exposure;
            stage.settings.color_grading.exposure = exposure;
        }
        ViewerAction::SetTonemap(index) => {
            stage.viewer.view.tonemap = index;
            stage.settings.color_grading.tonemap_algorithm = tonemap(index);
        }
        ViewerAction::SetSsao(enabled) => {
            stage.viewer.view.ssao = enabled;
            stage.settings.ssao_enabled = enabled;
        }
        ViewerAction::SetBloom(enabled) => {
            stage.viewer.view.bloom = enabled;
            stage.settings.bloom_enabled = enabled;
        }
        ViewerAction::SetSsr(enabled) => {
            stage.viewer.view.ssr = enabled;
            stage.settings.ssr_enabled = enabled;
        }
        ViewerAction::SetSsgi(enabled) => {
            stage.viewer.view.ssgi = enabled;
            stage.settings.ssgi_enabled = enabled;
        }
        ViewerAction::SetAntiAliasing(index) => {
            stage.viewer.view.anti_aliasing = index;
            stage.settings.anti_aliasing = match index {
                0 => AntiAliasing::Off,
                2 => AntiAliasing::Taa,
                _ => AntiAliasing::Fxaa,
            };
        }
        ViewerAction::SetRenderScale(scale) => {
            let scale = scale.clamp(0.25, 2.0);
            stage.viewer.view.render_scale = scale;
            stage.settings.render_scale = scale;
        }
        ViewerAction::SetFog(enabled) => {
            stage.viewer.view.fog = enabled;
            let fog = enabled.then(|| fog_of(stage.viewer));
            stage.settings.fog = fog;
        }
        ViewerAction::SetFogStart(start) => {
            let view = &mut stage.viewer.view;
            view.fog_start = start.max(0.0);
            view.fog_end = view.fog_end.max(view.fog_start + 0.1);
            write_fog(stage);
        }
        ViewerAction::SetFogEnd(end) => {
            let view = &mut stage.viewer.view;
            view.fog_end = end.max(view.fog_start + 0.1);
            write_fog(stage);
        }
        ViewerAction::SetFogColor(color) => {
            stage.viewer.view.fog_color = color;
            write_fog(stage);
        }
        ViewerAction::SetDepthOfField(enabled) => {
            stage.viewer.view.depth_of_field = enabled;
            stage.settings.depth_of_field.enabled = enabled;
        }
        ViewerAction::SetFocusDistance(distance) => {
            let distance = distance.max(0.01);
            let viewer = &mut *stage.viewer;
            viewer.view.focus_distance = distance;
            viewer.view.render_panel_dirty = true;
            stage.settings.depth_of_field.focus_distance = distance;
        }
        ViewerAction::SetFocusRange(range) => {
            let range = range.max(0.01);
            stage.viewer.view.focus_range = range;
            stage.settings.depth_of_field.focus_range = range;
        }
        ViewerAction::SetSunIntensity(intensity) => {
            let viewer = &mut *stage.viewer;
            viewer.sun.intensity = intensity;
            if let Some(light) = viewer
                .sun
                .entity
                .and_then(|sun| storage.get_mut::<Light>(sun))
            {
                light.intensity = intensity;
            }
        }
        ViewerAction::SetSunShadows(enabled) => {
            let viewer = &mut *stage.viewer;
            viewer.sun.shadows = enabled;
            if let Some(light) = viewer
                .sun
                .entity
                .and_then(|sun| storage.get_mut::<Light>(sun))
            {
                light.cast_shadows = enabled;
            }
        }
        ViewerAction::SetSunYaw(yaw) => {
            let viewer = &mut *stage.viewer;
            viewer.sun.yaw = yaw;
            if let Some(sun) = viewer.sun.entity {
                point_sun(storage, sun, viewer.sun.yaw, viewer.sun.pitch);
            }
        }
        ViewerAction::SetSunPitch(pitch) => {
            let viewer = &mut *stage.viewer;
            viewer.sun.pitch = pitch.clamp(-89.0, 0.0);
            if let Some(sun) = viewer.sun.entity {
                point_sun(storage, sun, viewer.sun.yaw, viewer.sun.pitch);
            }
        }
        ViewerAction::SetVariant(name) => {
            stage.viewer.view.active_variant = name.clone();
            apply_variant(stage, storage, name);
        }
        ViewerAction::SetMorphWeight {
            entity,
            index,
            weight,
        } => {
            if let Some(weights) = storage.get_mut::<MorphWeights>(entity) {
                weights.set_weight(index, weight);
            }
            let viewer = &mut *stage.viewer;
            if let Some(morph) = viewer
                .model
                .morphs
                .iter_mut()
                .find(|morph| morph.entity == entity)
                && let Some(slot) = morph.weights.get_mut(index)
            {
                *slot = weight;
            }
        }
        ViewerAction::BlendToClip(index) => blend_to_clip(stage, storage, index),
        ViewerAction::SetBlendTime(value) => {
            stage.viewer.animation.blend_time = value.max(0.0);
        }
        ViewerAction::AddAnimationLayer { clip, weight } => {
            add_animation_layer(stage, storage, clip, weight)
        }
        ViewerAction::SetLayerWeight { index, weight } => {
            with_players(stage, storage, move |player| {
                if let Some(layer) = player.layers.get_mut(index) {
                    layer.weight = weight;
                }
            });
        }
        ViewerAction::RemoveAnimationLayer(index) => {
            with_players(stage, storage, move |player| {
                if index < player.layers.len() {
                    player.layers.remove(index);
                }
            });
            stage.viewer.view.layers_dirty = true;
        }
        ViewerAction::TogglePlayback => toggle_playback(stage, storage),
        ViewerAction::StopAnimation => {
            with_players(stage, storage, |player| player.stop());
            let viewer = &mut *stage.viewer;
            viewer.animation.playing = false;
            viewer.animation.time = 0.0;
        }
        ViewerAction::SeekAnimation(time) => {
            let clamped = time.max(0.0);
            with_players(stage, storage, move |player| player.time = clamped);
            stage.viewer.animation.time = clamped;
        }
        ViewerAction::SetAnimationSpeed(speed) => {
            with_players(stage, storage, move |player| player.speed = speed);
            stage.viewer.animation.speed = speed;
        }
        ViewerAction::SetAnimationLoop(looping) => {
            with_players(stage, storage, move |player| player.looping = looping);
            stage.viewer.animation.looping = looping;
        }
        ViewerAction::SnapAxis(axis) => snap_axis(active_camera, storage, axis),
        ViewerAction::SetGizmoMode(mode) => {
            let gizmo = &mut stage.viewer.gizmo;
            if gizmo.mode == mode {
                gizmo.enabled = !gizmo.enabled;
            } else {
                gizmo.mode = mode;
                gizmo.enabled = true;
            }
            gizmo.drag = None;
            gizmo.hovered = None;
        }
        ViewerAction::SetActiveCamera(entity) => {
            let fallback = stage.viewer.camera_input.viewer_camera;
            stage.active_camera.0 = entity.or(fallback);
        }
        ViewerAction::Screenshot => screenshot(stage),
        ViewerAction::ExportModel => export_dialog(stage, storage),
        ViewerAction::ApplyHeldClips => {
            stage.viewer.view.add_open = false;
            apply_held_clips(stage, storage)
        }
        ViewerAction::Undo => undo(stage, storage),
        ViewerAction::Redo => redo(stage, storage),
        ViewerAction::ToggleInterface => {
            let view = &mut stage.viewer.view;
            view.interface_visible = !view.interface_visible;
        }
        ViewerAction::ShowInterface => {
            stage.viewer.view.interface_visible = true;
        }
    }
}
