use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::*;

use crate::resources::GalleryResources;
use crate::systems::build_ui::{RECOLOR_CHOICES, recolor_button_name};

pub fn run_animation_systems(
    mut gallery: ResMut<GalleryResources>,
    time: Res<Time>,
    accessibility: Res<nightshade_ui::resources::RetainedUiAccessibility>,
    frame: Res<nightshade_ui::resources::RetainedUiFrame>,
    theme_state: Res<ThemeState>,
    mut runtime: ResMut<RetainedUiRuntime>,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let fps = time.frames_per_second;
    let uptime_ms = time.uptime_milliseconds;
    let delta = time.delta_time;
    let theme = theme_state.active_theme().clone();
    if let Some(text) = ui_named_entity(&accessibility, "fps_text") {
        ui_set_text(&mut commands, text, &format!("{fps:.0} fps"));
    }
    if let Some(text) = ui_named_entity(&accessibility, "uptime_text") {
        let seconds = uptime_ms / 1000;
        let minutes = seconds / 60;
        let remaining = seconds % 60;
        ui_set_text(&mut commands, text, &format!("{minutes}:{remaining:02}"));
    }

    apply_recolor_choice(&frame, &accessibility, &mut commands);

    if let Some(progress) = ui_named_entity(&accessibility, "progress_entity") {
        gallery.state.progress_value += delta * 0.1;
        if gallery.state.progress_value > 1.0 {
            gallery.state.progress_value = 0.0;
        }
        ui_progress_bar_set_value(&mut commands, progress, gallery.state.progress_value);
    }

    if let Some(canvas) = ui_named_entity(&accessibility, "canvas_demo") {
        draw_canvas_demo(storage, &theme, canvas, uptime_ms as f32 / 1000.0);
    }

    runtime.background_color = Some(theme.background_color);
    let accent_dim = vec4(
        theme.accent_color.x,
        theme.accent_color.y,
        theme.accent_color.z,
        0.4,
    );

    if let Some(pulse) = ui_named_entity(&accessibility, "pulse_entity") {
        let time = uptime_ms as f32 / 1000.0;
        let pulse_alpha = (time * 2.0).sin() * 0.5 + 0.5;
        let pulse_color = blend_color(accent_dim, theme.accent_color, pulse_alpha);
        if let Some(color_comp) = storage.get_mut::<nightshade_ui::components::UiNodeColor>(pulse) {
            color_comp.colors[0] = Some(pulse_color);
        }
    }

    gallery.state.color_blend_time += delta * 0.3;
    if gallery.state.color_blend_time > 1.0 {
        gallery.state.color_blend_time -= 1.0;
    }
    let blended = blend_color(
        theme.accent_color,
        theme.text_color_accent,
        gallery.state.color_blend_time,
    );

    let time = uptime_ms as f32 / 1000.0;
    if let Some(entity) = ui_named_entity(&accessibility, "effects_pulse") {
        let pulse_alpha = (time * 2.5).sin() * 0.5 + 0.5;
        let pulse_color = blend_color(accent_dim, theme.accent_color, pulse_alpha);
        if let Some(color_comp) = storage.get_mut::<nightshade_ui::components::UiNodeColor>(entity)
        {
            color_comp.colors[0] = Some(pulse_color);
        }
    }

    if let Some(entity) = ui_named_entity(&accessibility, "effects_blend")
        && let Some(color_comp) = storage.get_mut::<nightshade_ui::components::UiNodeColor>(entity)
    {
        color_comp.colors[0] = Some(blended);
    }

    if let Some(entity) = ui_named_entity(&accessibility, "effects_progress") {
        gallery.state.effects_progress_value += delta * 0.4;
        if gallery.state.effects_progress_value > 1.0 {
            gallery.state.effects_progress_value = 0.0;
        }
        ui_progress_bar_set_value(&mut commands, entity, gallery.state.effects_progress_value);
    }

    if let Some(entity) = ui_named_entity(&accessibility, "effects_spinner") {
        gallery.state.effects_spinner_angle += delta * 1.4;
        let angle = gallery.state.effects_spinner_angle;
        let hue = (angle * 0.5).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU;
        let r = (hue * 6.0).sin().abs();
        let g = (hue * 6.0 + 2.0).sin().abs();
        let b = (hue * 6.0 + 4.0).sin().abs();
        let color = vec4(r, g, b, 1.0);
        if let Some(color_comp) = storage.get_mut::<nightshade_ui::components::UiNodeColor>(entity)
        {
            color_comp.colors[0] = Some(color);
        }
        if let Some(node) = storage.get_mut::<nightshade_ui::components::UiLayoutNode>(entity) {
            let radius = 8.0;
            let offset_x = radius * angle.cos();
            let offset_y = radius * angle.sin();
            if let Some(nightshade_ui::components::layout_types::UiLayoutType::Window(window)) =
                node.base_layout.as_mut()
            {
                window.position = nightshade_ui::data::units::Rl(vec2(50.0, 50.0))
                    + nightshade_ui::data::units::Ab(vec2(offset_x, offset_y));
            }
        }
    }
}

fn draw_canvas_demo(
    world: &mut nightshade::ecs::Storage,
    theme: &nightshade_ui::resources::theme::UiTheme,
    entity: Entity,
    time: f32,
) {
    let size = vec2(280.0, 60.0);
    let accent = theme.accent_color;
    let text_color = theme.text_color;

    ui_canvas_clear(world, entity);
    ui_canvas_rect(
        world,
        entity,
        vec2(0.0, 0.0),
        size,
        vec4(accent.x, accent.y, accent.z, 0.10),
        4.0,
    );
    ui_canvas_rect_stroke(
        world,
        entity,
        vec2(0.5, 0.5),
        size - vec2(1.0, 1.0),
        vec4(accent.x, accent.y, accent.z, 0.55),
        1.0,
        4.0,
    );

    let wave: Vec<_> = (0..=56)
        .map(|step| {
            let progress = step as f32 / 56.0;
            let x = 8.0 + progress * (size.x - 16.0);
            let y = size.y * 0.5 + (progress * 12.0 + time * 2.0).sin() * 14.0;
            vec2(x, y)
        })
        .collect();
    ui_canvas_polyline(world, entity, &wave, 2.0, accent, false);

    let sweep = time.rem_euclid(std::f32::consts::TAU);
    ui_canvas_arc(
        world,
        entity,
        vec2(30.0, 30.0),
        16.0,
        (sweep, sweep + 2.2),
        2.0,
        vec4(text_color.x, text_color.y, text_color.z, 0.8),
    );
    ui_canvas_quadratic_bezier(
        world,
        entity,
        vec2(size.x - 70.0, size.y - 8.0),
        vec2(size.x - 40.0, 8.0),
        vec2(size.x - 10.0, size.y - 8.0),
        2.0,
        vec4(text_color.x, text_color.y, text_color.z, 0.5),
    );

    let head = wave[wave.len() - 1];
    ui_canvas_circle(world, entity, head, 3.5, accent);
}

fn apply_recolor_choice(
    frame: &nightshade_ui::resources::RetainedUiFrame,
    accessibility: &nightshade_ui::resources::RetainedUiAccessibility,
    commands: &mut Commands,
) {
    let clicked: Vec<Entity> = ui_button_clicks(frame).collect();
    if clicked.is_empty() {
        return;
    }

    let Some(swatch) = ui_named_entity(accessibility, "recolor_swatch") else {
        return;
    };

    for (label, color) in RECOLOR_CHOICES {
        let Some(button) = ui_named_entity(accessibility, &recolor_button_name(label)) else {
            continue;
        };
        if clicked.contains(&button) {
            ui_set_color::<UiBase>(commands, swatch, color);
            ui_set_text(commands, swatch, &format!("{label}, set while running"));
        }
    }
}
