use nightshade::prelude::*;

use crate::ecs::GalleryResources;

pub fn run_animation_systems(gallery: &mut GalleryResources, world: &mut World) {
    let timing = world.res::<Time>().clone();
    let fps = timing.frames_per_second;
    let uptime_ms = timing.uptime_milliseconds;
    let delta = timing.delta_time;
    if let Some(text) = ui_named_entity(world, "fps_text") {
        ui_set_text(world, text, &format!("{fps:.0} fps"));
    }
    if let Some(text) = ui_named_entity(world, "uptime_text") {
        let seconds = uptime_ms / 1000;
        let minutes = seconds / 60;
        let remaining = seconds % 60;
        ui_set_text(world, text, &format!("{minutes}:{remaining:02}"));
    }

    if let Some(progress) = ui_named_entity(world, "progress_entity") {
        gallery.state.progress_value += delta * 0.1;
        if gallery.state.progress_value > 1.0 {
            gallery.state.progress_value = 0.0;
        }
        ui_progress_bar_set_value(world, progress, gallery.state.progress_value);
    }

    let theme = world
        .res::<RetainedUiState>()
        .theme_state
        .active_theme()
        .clone();
    world.res_mut::<RetainedUiState>().background_color = Some(theme.background_color);
    let accent_dim = vec4(
        theme.accent_color.x,
        theme.accent_color.y,
        theme.accent_color.z,
        0.4,
    );

    if let Some(pulse) = ui_named_entity(world, "pulse_entity") {
        let time = uptime_ms as f32 / 1000.0;
        let pulse_alpha = (time * 2.0).sin() * 0.5 + 0.5;
        let pulse_color = blend_color(accent_dim, theme.accent_color, pulse_alpha);
        if let Some(color_comp) =
            world.get_mut::<nightshade::ecs::ui::components::UiNodeColor>(pulse)
        {
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
    if let Some(entity) = ui_named_entity(world, "effects_pulse") {
        let pulse_alpha = (time * 2.5).sin() * 0.5 + 0.5;
        let pulse_color = blend_color(accent_dim, theme.accent_color, pulse_alpha);
        if let Some(color_comp) =
            world.get_mut::<nightshade::ecs::ui::components::UiNodeColor>(entity)
        {
            color_comp.colors[0] = Some(pulse_color);
        }
    }

    if let Some(entity) = ui_named_entity(world, "effects_blend")
        && let Some(color_comp) =
            world.get_mut::<nightshade::ecs::ui::components::UiNodeColor>(entity)
    {
        color_comp.colors[0] = Some(blended);
    }

    if let Some(entity) = ui_named_entity(world, "effects_progress") {
        gallery.state.effects_progress_value += delta * 0.4;
        if gallery.state.effects_progress_value > 1.0 {
            gallery.state.effects_progress_value = 0.0;
        }
        ui_progress_bar_set_value(world, entity, gallery.state.effects_progress_value);
    }

    if let Some(entity) = ui_named_entity(world, "effects_spinner") {
        gallery.state.effects_spinner_angle += delta * 1.4;
        let angle = gallery.state.effects_spinner_angle;
        let hue = (angle * 0.5).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU;
        let r = (hue * 6.0).sin().abs();
        let g = (hue * 6.0 + 2.0).sin().abs();
        let b = (hue * 6.0 + 4.0).sin().abs();
        let color = vec4(r, g, b, 1.0);
        if let Some(color_comp) =
            world.get_mut::<nightshade::ecs::ui::components::UiNodeColor>(entity)
        {
            color_comp.colors[0] = Some(color);
        }
        if let Some(node) = world.get_mut::<nightshade::ecs::ui::components::UiLayoutNode>(entity) {
            let radius = 8.0;
            let offset_x = radius * angle.cos();
            let offset_y = radius * angle.sin();
            if let Some(nightshade::ecs::ui::layout_types::UiLayoutType::Window(window)) =
                node.base_layout.as_mut()
            {
                window.position = nightshade::ecs::ui::units::Rl(vec2(50.0, 50.0))
                    + nightshade::ecs::ui::units::Ab(vec2(offset_x, offset_y));
            }
        }
    }
}
