use nightshade::prelude::*;

use crate::ecs::GalleryWorld;

pub fn run_animation_systems(gallery: &mut GalleryWorld, world: &mut World) {
    let timing = world.resources.window.timing.clone();
    let fps = timing.frames_per_second;
    let uptime_ms = timing.uptime_milliseconds;
    let delta = timing.delta_time;
    let entity_count = world.core.entity_count();

    if let Some(text) = gallery.resources.handles.fps_text {
        ui_set_text(world, text, &format!("{fps:.0} fps"));
    }
    if let Some(text) = gallery.resources.handles.uptime_text {
        let seconds = uptime_ms / 1000;
        let minutes = seconds / 60;
        let remaining = seconds % 60;
        ui_set_text(world, text, &format!("{minutes}:{remaining:02}"));
    }
    if let Some(text) = gallery.resources.handles.entity_count_text {
        ui_set_text(world, text, &format!("{entity_count} entities"));
    }

    if let Some(progress) = gallery.resources.handles.progress_entity {
        gallery.resources.handles.progress_value += delta * 0.1;
        if gallery.resources.handles.progress_value > 1.0 {
            gallery.resources.handles.progress_value = 0.0;
        }
        ui_progress_bar_set_value(world, progress, gallery.resources.handles.progress_value);
    }

    let theme = world
        .resources
        .retained_ui
        .theme_state
        .active_theme()
        .clone();
    world.resources.retained_ui.background_color = Some(theme.background_color);
    let accent_dim = nalgebra_glm::Vec4::new(
        theme.accent_color.x,
        theme.accent_color.y,
        theme.accent_color.z,
        0.4,
    );

    if let Some(pulse) = gallery.resources.handles.pulse_entity {
        let time = uptime_ms as f32 / 1000.0;
        let pulse_alpha = (time * 2.0).sin() * 0.5 + 0.5;
        let pulse_color = blend_color(accent_dim, theme.accent_color, pulse_alpha);
        if let Some(color_comp) = world.ui.get_ui_node_color_mut(pulse) {
            color_comp.colors[0] = Some(pulse_color);
        }
    }

    gallery.resources.handles.color_blend_time += delta * 0.3;
    if gallery.resources.handles.color_blend_time > 1.0 {
        gallery.resources.handles.color_blend_time -= 1.0;
    }
    let blended = blend_color(
        theme.accent_color,
        theme.text_color_accent,
        gallery.resources.handles.color_blend_time,
    );

    let time = uptime_ms as f32 / 1000.0;
    if let Some(entity) = gallery.resources.handles.effects_pulse {
        let pulse_alpha = (time * 2.5).sin() * 0.5 + 0.5;
        let pulse_color = blend_color(accent_dim, theme.accent_color, pulse_alpha);
        if let Some(color_comp) = world.ui.get_ui_node_color_mut(entity) {
            color_comp.colors[0] = Some(pulse_color);
        }
    }

    if let Some(entity) = gallery.resources.handles.effects_blend
        && let Some(color_comp) = world.ui.get_ui_node_color_mut(entity)
    {
        color_comp.colors[0] = Some(blended);
    }

    if let Some(entity) = gallery.resources.handles.effects_progress {
        gallery.resources.handles.effects_progress_value += delta * 0.4;
        if gallery.resources.handles.effects_progress_value > 1.0 {
            gallery.resources.handles.effects_progress_value = 0.0;
        }
        ui_progress_bar_set_value(
            world,
            entity,
            gallery.resources.handles.effects_progress_value,
        );
    }

    if let Some(entity) = gallery.resources.handles.effects_spinner {
        gallery.resources.handles.effects_spinner_angle += delta * 1.4;
        let angle = gallery.resources.handles.effects_spinner_angle;
        let hue = (angle * 0.5).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU;
        let r = (hue * 6.0).sin().abs();
        let g = (hue * 6.0 + 2.0).sin().abs();
        let b = (hue * 6.0 + 4.0).sin().abs();
        let color = nalgebra_glm::Vec4::new(r, g, b, 1.0);
        if let Some(color_comp) = world.ui.get_ui_node_color_mut(entity) {
            color_comp.colors[0] = Some(color);
        }
        if let Some(node) = world.ui.get_ui_layout_node_mut(entity) {
            let radius = 8.0;
            let offset_x = radius * angle.cos();
            let offset_y = radius * angle.sin();
            if let Some(nightshade::ecs::ui::layout_types::UiLayoutType::Window(window)) =
                node.base_layout.as_mut()
            {
                window.position =
                    nightshade::ecs::ui::units::Rl(nalgebra_glm::Vec2::new(50.0, 50.0))
                        + nightshade::ecs::ui::units::Ab(nalgebra_glm::Vec2::new(
                            offset_x, offset_y,
                        ));
            }
        }
    }
}
