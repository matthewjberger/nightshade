use nalgebra_glm;
use nalgebra_glm::Vec2;

use crate::components::ResponsiveOverride;
use crate::resources::UiBreakpoint;

pub fn ui_responsive_apply_system(
    runtime: nightshade::ecs::system_param::Res<crate::resources::RetainedUiRuntime>,
    window: nightshade::ecs::system_param::Res<nightshade_platform::contract::window::Window>,
    mut viewport: nightshade::ecs::system_param::ResMut<crate::resources::UiViewport>,
    mut dirty: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiDirty>,
    storage: &mut nightshade::ecs::Storage,
) {
    if !runtime.visible {
        return;
    }

    let viewport_raw = window.cached_viewport_size;
    let dpi_scale = window.cached_scale_factor.max(0.0001);
    let viewport_size = viewport_raw
        .map(|(width, height)| Vec2::new(width as f32, height as f32))
        .unwrap_or(Vec2::new(800.0, 600.0))
        / dpi_scale;
    let breakpoint = UiBreakpoint::from_width(viewport_size.x);
    *viewport = crate::resources::UiViewport {
        size: viewport_size,
        breakpoint,
    };

    let entities: Vec<nightshade::ecs::Entity> = storage
        .query_entities::<(crate::components::UiLayoutNode,)>()
        .collect();

    let mut any_applied = false;

    for entity in entities {
        let Some(node) = storage.get::<crate::components::UiLayoutNode>(entity) else {
            continue;
        };
        let Some(responsive) = node.responsive.as_ref() else {
            continue;
        };
        if responsive.baseline_captured && responsive.current_applied == Some(breakpoint) {
            continue;
        }

        any_applied = true;
        let mut responsive = responsive.clone();
        if !responsive.baseline_captured {
            responsive.baseline = ResponsiveOverride::capture_from(node);
            responsive.baseline_captured = true;
        }
        let baseline = responsive.baseline;
        let override_value = responsive.overrides[breakpoint.index()];
        responsive.current_applied = Some(breakpoint);

        let Some(node) = storage.get_mut::<crate::components::UiLayoutNode>(entity) else {
            continue;
        };
        baseline.apply_to(node);
        override_value.apply_to(node);
        node.responsive = Some(responsive);
    }

    if any_applied {
        dirty.layout_dirty = true;
    }
}
