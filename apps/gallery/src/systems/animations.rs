use nightshade::prelude::*;

use crate::ecs::GalleryWorld;

const CYAN: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.9, 1.0, 1.0);
const MAGENTA: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(1.0, 0.0, 0.6, 1.0);

pub fn run_animation_systems(gallery: &mut GalleryWorld, world: &mut World) {
    let timing = world.resources.window.timing.clone();
    let fps = timing.frames_per_second;
    let uptime_ms = timing.uptime_milliseconds;
    let delta = timing.delta_time;
    let entity_count = world.core.entity_count();

    if let Some(text) = gallery.resources.handles.fps_text {
        world.ui_set_text(text, &format!("{fps:.0}"));
    }
    if let Some(text) = gallery.resources.handles.uptime_text {
        let seconds = uptime_ms / 1000;
        let minutes = seconds / 60;
        let remaining = seconds % 60;
        world.ui_set_text(text, &format!("{minutes}:{remaining:02}"));
    }
    if let Some(text) = gallery.resources.handles.entity_count_text {
        world.ui_set_text(text, &format!("{entity_count}"));
    }

    if let Some(progress) = gallery.resources.handles.progress_entity {
        gallery.resources.handles.progress_value += delta * 0.1;
        if gallery.resources.handles.progress_value > 1.0 {
            gallery.resources.handles.progress_value = 0.0;
        }
        world.ui_progress_bar_set_value(progress, gallery.resources.handles.progress_value);
    }

    if let Some(pulse) = gallery.resources.handles.pulse_entity {
        let time = uptime_ms as f32 / 1000.0;
        let pulse_alpha = (time * 2.0).sin() * 0.5 + 0.5;
        let pulse_color = blend_color(
            nalgebra_glm::Vec4::new(0.0, 1.0, 0.5, 0.3),
            nalgebra_glm::Vec4::new(0.0, 1.0, 0.5, 1.0),
            pulse_alpha,
        );
        if let Some(color_comp) = world.ui.get_ui_node_color_mut(pulse) {
            color_comp.colors[0] = Some(pulse_color);
        }
    }

    gallery.resources.handles.color_blend_time += delta * 0.3;
    if gallery.resources.handles.color_blend_time > 1.0 {
        gallery.resources.handles.color_blend_time -= 1.0;
    }
    let blended = blend_color(CYAN, MAGENTA, gallery.resources.handles.color_blend_time);
    let viewport = world
        .resources
        .window
        .cached_viewport_size
        .map(|(w, h)| (w as f32, h as f32))
        .unwrap_or((800.0, 600.0));
    world.resources.retained_ui.draw_rect(
        nalgebra_glm::Vec2::new(40.0, viewport.1 - 70.0),
        nalgebra_glm::Vec2::new(160.0, 4.0),
        blended,
    );

    let time = uptime_ms as f32 / 1000.0;
    if let Some(entity) = gallery.resources.handles.effects_pulse {
        let pulse_alpha = (time * 2.5).sin() * 0.5 + 0.5;
        let scale = 0.7 + pulse_alpha * 0.3;
        let pulse_color = nalgebra_glm::Vec4::new(
            0.4 + pulse_alpha * 0.6,
            1.0,
            0.4 + (1.0 - pulse_alpha) * 0.6,
            1.0,
        );
        if let Some(color_comp) = world.ui.get_ui_node_color_mut(entity) {
            color_comp.colors[0] = Some(pulse_color);
        }
        let _ = scale;
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
        world.ui_progress_bar_set_value(entity, gallery.resources.handles.effects_progress_value);
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
