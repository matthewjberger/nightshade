use crate::resources::{Handles, Panel};
use crate::theme;
use nightshade::prelude::*;
use nightshade_graphics::prelude::{AntiAliasing, RenderSettings};
use nightshade_platform::prelude::*;
use nightshade_render::contract::Light;
use nightshade_render::queries::lights::sun_rotation;
use nightshade_scene::prelude::LocalTransform;
use nightshade_ui::prelude::*;

pub fn build(context: UiBuildContext<'_, '_>, settings: &RenderSettings) -> Panel {
    let air = settings.fog.unwrap_or_default();
    let field = settings.depth_of_field;
    let grading = settings.color_grading;
    let mut panel = Panel::default();

    let mut tree = UiTreeBuilder::new(context);
    let docked = tree.add_docked_panel_left("sponza", "Sponza", theme::PANEL_WIDTH);
    let content = tree
        .component::<UiPanelData>(docked)
        .map(|data| data.content_entity)
        .expect("the panel has a content node");
    tree.edit::<UiLayoutNode>(content, |node| {
        node.flow_layout = Some(FlowLayout {
            direction: FlowDirection::Vertical,
            padding: theme::PANEL_PADDING,
            spacing: theme::PANEL_SPACING,
            alignment: FlowAlignment::Start,
            cross_alignment: FlowAlignment::Start,
            wrap: false,
        });
    });

    let inside = tree.in_parent(content, |tree| {
        let scroll = tree.add_scroll_area_fill(theme::PANEL_PADDING, theme::PANEL_SPACING);
        tree.component::<UiScrollAreaData>(scroll)
            .map(|data| data.content_entity)
            .expect("the scroll area has a content node")
    });

    tree.in_parent(inside, |tree| {
        tree.add_heading("Frame");
        panel.rate = tree.add_label("0 frames a second");
        panel.rate_in_title = tree.add_checkbox("Rate in the title", true);
        tree.add_label("Scale");
        panel.render_scale = tree.add_slider(0.5, 1.0, settings.render_scale);
        tree.add_separator();

        tree.add_heading("Rays");
        tree.add_label("On a device that can trace");
        panel.traced_sun = tree.add_checkbox("Sun shadow", settings.sun_shadow_traced);
        panel.traced_reflections = tree.add_checkbox("Reflections", settings.ssr_traced);
        panel.traced_occlusion = tree.add_checkbox("Occlusion", settings.ssao_traced);
        tree.add_separator();

        tree.add_heading("Screen");
        panel.occlusion = tree.add_checkbox("Occlusion", settings.ssao_enabled);
        tree.add_label("Occlusion strength");
        panel.occlusion_strength = tree.add_slider(0.0, 2.0, settings.ssao_intensity);
        panel.reflections = tree.add_checkbox("Reflections", settings.ssr_enabled);
        tree.add_label("Reflection strength");
        panel.reflection_strength = tree.add_slider(0.0, 2.0, settings.ssr_intensity);
        panel.bounced = tree.add_checkbox("Bounced light", settings.ssgi_enabled);
        tree.add_label("Bounced strength");
        panel.bounced_strength = tree.add_slider(0.0, 2.0, settings.ssgi_intensity);
        tree.add_separator();

        tree.add_heading("Air");
        panel.lit_air = tree.add_checkbox("Lit through", air.volumetric);
        tree.add_label("Density");
        panel.air_density = tree.add_slider(0.0, 0.12, air.density);
        tree.add_label("Thrown forward");
        panel.air_forward = tree.add_slider(0.0, 0.95, air.anisotropy);
        tree.add_label("Thins above");
        panel.air_falloff = tree.add_slider(0.0, 1.0, air.height_falloff);
        tree.add_label("Reaches");
        panel.air_reach = tree.add_slider(10.0, 120.0, air.end);
        tree.add_separator();

        tree.add_heading("Light");
        tree.add_label("Sun");
        panel.sun_intensity = tree.add_slider(0.0, 60.0, theme::SUN_INTENSITY);
        tree.add_label("Sun turn");
        panel.sun_yaw = tree.add_slider(-theme::HALF_TURN, theme::HALF_TURN, theme::SUN_YAW);
        tree.add_label("Sun tilt");
        panel.sun_pitch = tree.add_slider(-1.5, -0.1, theme::SUN_PITCH);
        tree.add_label("Lanterns");
        panel.lantern_intensity = tree.add_slider(0.0, 200.0, theme::LANTERN_INTENSITY);
        tree.add_label("Lantern reach");
        panel.lantern_range = tree.add_slider(1.0, 30.0, theme::LANTERN_RANGE);
        tree.add_separator();

        tree.add_heading("Over the frame");
        panel.bloom = tree.add_checkbox("Bloom", settings.bloom_enabled);
        tree.add_label("Bloom strength");
        panel.bloom_strength = tree.add_slider(0.0, 1.0, settings.bloom_intensity);
        tree.add_label("Smoothing of the edges");
        panel.anti_aliasing = tree.add_dropdown(
            &AntiAliasing::ALL
                .iter()
                .map(|technique| technique.name())
                .collect::<Vec<_>>(),
            AntiAliasing::ALL
                .iter()
                .position(|technique| *technique == settings.anti_aliasing)
                .unwrap_or_default(),
        );
        tree.add_label("Sharpen");
        panel.sharpen = tree.add_slider(0.0, 1.5, settings.taa_sharpness);
        panel.focus = tree.add_checkbox("Focus follows the look", field.enabled);
        tree.add_label("Blur");
        panel.focus_blur = tree.add_slider(0.0, 12.0, field.max_blur_radius);
        panel.exposure_follows = tree.add_checkbox("Exposure follows", grading.auto_exposure);
        tree.add_label("Exposure target");
        panel.exposure_target = tree.add_slider(0.02, 0.6, grading.auto_exposure_target);
    });

    tree.edit::<UiLayoutNode>(docked, |node| node.visible = false);
    tree.runtime_mut().visible = false;
    tree.finish();
    panel.docked = docked;
    panel
}

pub fn toggle_the_panel(
    input: Res<Input>,
    mut panel: ResMut<Panel>,
    mut runtime: ResMut<RetainedUiRuntime>,
    mut dirty: ResMut<nightshade_ui::resources::RetainedUiDirty>,
    mut commands: Commands,
) {
    let asked = input
        .keyboard
        .just_pressed(nightshade_platform::winit::keyboard::KeyCode::F1);
    if !asked && !panel.wanted {
        return;
    }
    panel.shown = if panel.wanted { true } else { !panel.shown };
    panel.wanted = false;
    runtime.visible = panel.shown;
    ui_set_visible(&mut dirty, &mut commands, panel.docked, panel.shown);
}

pub fn read_the_panel(
    mut panel: ResMut<Panel>,
    handles: Res<Handles>,
    mut settings: ResMut<RenderSettings>,
    mut lights: Query<(&mut Light,)>,
    mut suns: Query<(&mut LocalTransform,)>,
    mut widgets: Query<(
        Option<&UiSliderData>,
        Option<&UiCheckboxData>,
        Option<&UiDropdownData>,
    )>,
) {
    if let Some(value) = flipped(&mut widgets, panel.rate_in_title) {
        panel.rate_in_title_on = value;
    }
    if let Some(value) = moved(&mut widgets, panel.render_scale) {
        settings.render_scale = value;
    }

    if let Some(value) = flipped(&mut widgets, panel.traced_sun) {
        settings.sun_shadow_traced = value;
    }
    if let Some(value) = flipped(&mut widgets, panel.traced_reflections) {
        settings.ssr_traced = value;
    }
    if let Some(value) = flipped(&mut widgets, panel.traced_occlusion) {
        settings.ssao_traced = value;
    }

    if let Some(value) = flipped(&mut widgets, panel.occlusion) {
        settings.ssao_enabled = value;
    }
    if let Some(value) = moved(&mut widgets, panel.occlusion_strength) {
        settings.ssao_intensity = value;
    }
    if let Some(value) = flipped(&mut widgets, panel.reflections) {
        settings.ssr_enabled = value;
    }
    if let Some(value) = moved(&mut widgets, panel.reflection_strength) {
        settings.ssr_intensity = value;
    }
    if let Some(value) = flipped(&mut widgets, panel.bounced) {
        settings.ssgi_enabled = value;
    }
    if let Some(value) = moved(&mut widgets, panel.bounced_strength) {
        settings.ssgi_intensity = value;
    }

    let lit_air = flipped(&mut widgets, panel.lit_air);
    let density = moved(&mut widgets, panel.air_density);
    let forward = moved(&mut widgets, panel.air_forward);
    let falloff = moved(&mut widgets, panel.air_falloff);
    let reach = moved(&mut widgets, panel.air_reach);
    if let Some(air) = settings.fog.as_mut() {
        if let Some(value) = lit_air {
            air.volumetric = value;
        }
        if let Some(value) = density {
            air.density = value;
        }
        if let Some(value) = forward {
            air.anisotropy = value;
        }
        if let Some(value) = falloff {
            air.height_falloff = value;
        }
        if let Some(value) = reach {
            air.end = value;
        }
    }

    if let Some(value) = flipped(&mut widgets, panel.bloom) {
        settings.bloom_enabled = value;
    }
    if let Some(value) = moved(&mut widgets, panel.bloom_strength) {
        settings.bloom_intensity = value;
    }
    if let Some(index) = picked(&mut widgets, panel.anti_aliasing)
        && let Some(technique) = AntiAliasing::ALL.get(index)
    {
        settings.anti_aliasing = *technique;
    }
    if let Some(value) = moved(&mut widgets, panel.sharpen) {
        settings.taa_sharpness = value;
    }
    if let Some(value) = flipped(&mut widgets, panel.focus) {
        settings.depth_of_field.enabled = value;
    }
    if let Some(value) = moved(&mut widgets, panel.focus_blur) {
        settings.depth_of_field.max_blur_radius = value;
    }
    if let Some(value) = flipped(&mut widgets, panel.exposure_follows) {
        settings.color_grading.auto_exposure = value;
    }
    if let Some(value) = moved(&mut widgets, panel.exposure_target) {
        settings.color_grading.auto_exposure_target = value;
    }

    if let (Some(sun), Some(intensity)) = (handles.sun, moved(&mut widgets, panel.sun_intensity)) {
        lights.for_entity(sun, |(light,)| light.intensity = intensity);
    }
    let turned = moved(&mut widgets, panel.sun_yaw);
    let tilted = moved(&mut widgets, panel.sun_pitch);
    if let (Some(sun), true) = (handles.sun, turned.is_some() || tilted.is_some()) {
        let yaw = turned.or_else(|| now(&mut widgets, panel.sun_yaw));
        let pitch = tilted.or_else(|| now(&mut widgets, panel.sun_pitch));
        if let (Some(yaw), Some(pitch)) = (yaw, pitch) {
            suns.for_entity(sun, |(transform,)| {
                transform.rotation = sun_rotation(yaw, pitch);
            });
        }
    }

    let intensity = moved(&mut widgets, panel.lantern_intensity);
    let range = moved(&mut widgets, panel.lantern_range);
    if intensity.is_none() && range.is_none() {
        return;
    }
    for lantern in handles.lanterns.iter().flatten() {
        lights.for_entity(*lantern, |(light,)| {
            if let Some(intensity) = intensity {
                light.intensity = intensity;
            }
            if let Some(range) = range {
                light.range = range;
            }
        });
    }
}

pub fn refresh_the_rate(
    time: Res<Time>,
    mut panel: ResMut<Panel>,
    mut window: ResMut<Window>,
    mut commands: Commands,
) {
    let rate = time.frames_per_second.round() as i32;
    let wanted = panel.rate_in_title_on.then_some(rate);
    if panel.title_rate != wanted {
        window.title = match wanted {
            Some(rate) => format!("{} - {rate} fps", theme::TITLE),
            None => theme::TITLE.to_string(),
        };
        panel.title_rate = wanted;
    }
    if panel.shown && panel.label_rate != rate {
        ui_set_text(
            &mut commands,
            panel.rate,
            &format!("{rate} frames a second"),
        );
        panel.label_rate = rate;
    }
}

fn now(
    widgets: &mut Query<
        '_,
        (
            Option<&UiSliderData>,
            Option<&UiCheckboxData>,
            Option<&UiDropdownData>,
        ),
    >,
    entity: Entity,
) -> Option<f32> {
    let mut value = None;
    widgets.for_entity(entity, |(slider, _, _)| {
        value = slider.map(|slider| slider.value);
    });
    value
}

fn moved(
    widgets: &mut Query<
        '_,
        (
            Option<&UiSliderData>,
            Option<&UiCheckboxData>,
            Option<&UiDropdownData>,
        ),
    >,
    entity: Entity,
) -> Option<f32> {
    let mut value = None;
    widgets.for_entity(entity, |(slider, _, _)| {
        value = slider
            .filter(|slider| slider.changed)
            .map(|slider| slider.value);
    });
    value
}

fn flipped(
    widgets: &mut Query<
        '_,
        (
            Option<&UiSliderData>,
            Option<&UiCheckboxData>,
            Option<&UiDropdownData>,
        ),
    >,
    entity: Entity,
) -> Option<bool> {
    let mut value = None;
    widgets.for_entity(entity, |(_, checkbox, _)| {
        value = checkbox
            .filter(|checkbox| checkbox.changed)
            .map(|checkbox| checkbox.value);
    });
    value
}

fn picked(
    widgets: &mut Query<
        '_,
        (
            Option<&UiSliderData>,
            Option<&UiCheckboxData>,
            Option<&UiDropdownData>,
        ),
    >,
    entity: Entity,
) -> Option<usize> {
    let mut value = None;
    widgets.for_entity(entity, |(_, _, dropdown)| {
        value = dropdown
            .filter(|dropdown| dropdown.changed)
            .map(|dropdown| dropdown.selected_index);
    });
    value
}
