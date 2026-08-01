use crate::components::{
    UiAnimationPhase, UiAnimationType, UiNodeAnimation, UiNodeBlendedTransform, UiNodeShadow,
};
use crate::data::color::blend_state_colors;
use crate::data::state::{STATE_COUNT, UiBase, UiStateTrait};
use nalgebra_glm;
use nalgebra_glm::Vec2;
use nightshade_paint::data::Rect;

pub fn ui_layout_color_blend_system(
    mut runtime: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiRuntime>,
    mut dirty: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiDirty>,
    storage: &mut nightshade::ecs::Storage,
) {
    if !runtime.visible {
        return;
    }

    let mut entities = std::mem::take(&mut runtime.blend_scratch);
    entities.clear();
    entities.extend(storage.query_entities::<(crate::components::UiNodeColor,)>());

    for entity in entities.iter().copied() {
        let (colors, weights) = {
            let color_comp = match storage.get::<crate::components::UiNodeColor>(entity) {
                Some(color) => color.colors,
                None => continue,
            };
            let weight_comp = storage
                .get::<crate::components::UiStateWeights>(entity)
                .map(|state_weights| state_weights.weights)
                .unwrap_or([0.0; STATE_COUNT]);
            (color_comp, weight_comp)
        };

        let blended = blend_state_colors(&colors, &weights);

        let alpha_multiplier = if let Some(node) =
            storage.get::<crate::components::UiLayoutNode>(entity)
            && let Some(animation) = &node.animation
        {
            animation_alpha(animation)
        } else {
            1.0
        };

        if let Some(color_comp) = storage.get_mut::<crate::components::UiNodeColor>(entity) {
            let mut final_color = blended;
            if alpha_multiplier < 1.0 {
                final_color.w *= alpha_multiplier;
            }
            let old = color_comp.computed_color;
            color_comp.computed_color = final_color;
            let diff = old - final_color;
            if diff.x.abs() > 0.001
                || diff.y.abs() > 0.001
                || diff.z.abs() > 0.001
                || diff.w.abs() > 0.001
            {
                dirty.render_dirty = true;
            }
        }
    }

    entities.clear();
    entities.extend(storage.query_entities::<(crate::components::UiNodeShadowStates,)>());

    for entity in entities.iter().copied() {
        let shadows = match storage.get::<crate::components::UiNodeShadowStates>(entity) {
            Some(states) => states.shadows,
            None => continue,
        };
        let weights = storage
            .get::<crate::components::UiStateWeights>(entity)
            .map(|state_weights| state_weights.weights)
            .unwrap_or([0.0; STATE_COUNT]);

        let blended = blend_state_shadows(&shadows, &weights);

        if let Some(slot) = storage.get_mut::<crate::components::UiNodeShadow>(entity) {
            let old = *slot;
            *slot = blended;
            if !shadows_match(&old, &blended) {
                dirty.render_dirty = true;
            }
        }
    }

    runtime.blend_scratch = entities;
}

pub fn ui_layout_transform_blend_system(
    mut runtime: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiRuntime>,
    mut dirty: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiDirty>,
    storage: &mut nightshade::ecs::Storage,
) {
    if !runtime.visible {
        return;
    }

    let mut entities = std::mem::take(&mut runtime.blend_scratch);
    entities.clear();
    entities.extend(storage.query_entities::<(crate::components::UiNodeTransformStates,)>());

    for entity in entities.iter().copied() {
        let states = match storage.get::<crate::components::UiNodeTransformStates>(entity) {
            Some(states) => *states,
            None => continue,
        };
        let weights = storage
            .get::<crate::components::UiStateWeights>(entity)
            .map(|state_weights| state_weights.weights)
            .unwrap_or([0.0; STATE_COUNT]);

        let mut total_weight = 0.0_f32;
        let mut offset = Vec2::zeros();
        let mut scale = 0.0_f32;
        let mut have_explicit_scale = false;
        for (index, &weight) in weights.iter().enumerate() {
            if weight <= 0.0 {
                continue;
            }
            total_weight += weight;
            if let Some(state_offset) = states.offsets[index] {
                offset += state_offset * weight;
            }
            if let Some(state_scale) = states.scales[index] {
                scale += state_scale * weight;
                have_explicit_scale = true;
            } else {
                scale += weight;
            }
        }
        let blended_scale = if have_explicit_scale && total_weight > f32::EPSILON {
            scale / total_weight
        } else {
            1.0
        };
        let radius_override = dominant_state_radius(&states.radii, &weights);

        let blended = UiNodeBlendedTransform {
            offset,
            scale: blended_scale,
            radius_override,
        };

        storage.add_components::<(crate::components::UiNodeBlendedTransform,)>(entity);
        if let Some(slot) = storage.get_mut::<crate::components::UiNodeBlendedTransform>(entity) {
            let old = *slot;
            *slot = blended;
            if !blended_transforms_match(&old, &blended) {
                dirty.render_dirty = true;
            }
        }
    }

    runtime.blend_scratch = entities;
}

fn blended_transforms_match(left: &UiNodeBlendedTransform, right: &UiNodeBlendedTransform) -> bool {
    let eps = 0.001;
    (left.offset - right.offset).norm() < eps
        && (left.scale - right.scale).abs() < eps
        && match (left.radius_override, right.radius_override) {
            (Some(x), Some(y)) => (x - y).abs() < eps,
            (None, None) => true,
            _ => false,
        }
}

fn dominant_state_radius(
    radii: &[Option<f32>; STATE_COUNT],
    weights: &[f32; STATE_COUNT],
) -> Option<f32> {
    let mut best_index = None;
    let mut best_weight = -1.0;
    for index in 0..STATE_COUNT {
        if radii[index].is_none() {
            continue;
        }
        let weight = weights[index];
        if weight > best_weight {
            best_weight = weight;
            best_index = Some(index);
        }
    }
    best_index.and_then(|index| radii[index])
}

fn shadows_match(left: &UiNodeShadow, right: &UiNodeShadow) -> bool {
    let eps = 0.001;
    (left.color - right.color).norm() < eps
        && (left.offset - right.offset).norm() < eps
        && (left.blur - right.blur).abs() < eps
        && (left.spread - right.spread).abs() < eps
}

fn blend_state_shadows(
    shadows: &[Option<UiNodeShadow>; STATE_COUNT],
    weights: &[f32; STATE_COUNT],
) -> UiNodeShadow {
    let base = shadows[UiBase::INDEX].unwrap_or_default();

    let non_base_weight: f32 = (1..STATE_COUNT)
        .filter(|&index| shadows[index].is_some())
        .map(|index| weights[index].clamp(0.0, 1.0))
        .sum();
    if non_base_weight < f32::EPSILON {
        return base;
    }

    let base_weight = (1.0 - non_base_weight).max(0.0);
    let total = base_weight + non_base_weight;
    let mut blended = UiNodeShadow {
        color: base.color * base_weight,
        offset: base.offset * base_weight,
        blur: base.blur * base_weight,
        spread: base.spread * base_weight,
    };

    for index in 1..STATE_COUNT {
        let weight = weights[index].clamp(0.0, 1.0);
        if weight < f32::EPSILON {
            continue;
        }
        let Some(shadow) = shadows[index] else {
            continue;
        };
        blended.color += shadow.color * weight;
        blended.offset += shadow.offset * weight;
        blended.blur += shadow.blur * weight;
        blended.spread += shadow.spread * weight;
    }

    if total > f32::EPSILON {
        blended.color /= total;
        blended.offset /= total;
        blended.blur /= total;
        blended.spread /= total;
    }
    blended
}

fn animation_alpha(animation: &UiNodeAnimation) -> f32 {
    let uses_fade = matches!(
        match animation.phase {
            UiAnimationPhase::IntroPlaying => animation.intro,
            UiAnimationPhase::OutroPlaying => animation.outro,
            _ => return 1.0,
        },
        Some(UiAnimationType::Fade)
    );
    let progress = animation_progress_eased(animation);

    if uses_fade {
        match animation.phase {
            UiAnimationPhase::IntroPlaying => progress,
            UiAnimationPhase::OutroPlaying => 1.0 - progress,
            _ => 1.0,
        }
    } else {
        1.0
    }
}

fn animation_progress_eased(animation: &UiNodeAnimation) -> f32 {
    let progress = animation.progress.clamp(0.0, 1.0);
    match animation.phase {
        UiAnimationPhase::IntroPlaying => ease_out_cubic(progress),
        UiAnimationPhase::OutroPlaying => ease_in_cubic(progress),
        _ => 1.0,
    }
}

fn ease_out_cubic(progress: f32) -> f32 {
    1.0 - (1.0 - progress).powi(3)
}

fn ease_in_cubic(progress: f32) -> f32 {
    progress * progress * progress
}

pub(super) fn apply_animation_to_rect(rect: Rect, animation: &UiNodeAnimation) -> Rect {
    let anim_type = match animation.phase {
        UiAnimationPhase::IntroPlaying => animation.intro,
        UiAnimationPhase::OutroPlaying => animation.outro,
        _ => return rect,
    };

    let anim_type = match anim_type {
        Some(progress) => progress,
        None => return rect,
    };

    let progress = animation_progress_eased(animation);

    match animation.phase {
        UiAnimationPhase::IntroPlaying => apply_intro_transform(rect, anim_type, progress),
        UiAnimationPhase::OutroPlaying => apply_outro_transform(rect, anim_type, progress),
        _ => rect,
    }
}

fn apply_intro_transform(rect: Rect, anim_type: UiAnimationType, progress: f32) -> Rect {
    match anim_type {
        UiAnimationType::Fade => rect,
        UiAnimationType::SlideLeft => {
            let offset = rect.width() * (1.0 - progress);
            Rect::new(
                rect.min.x - offset,
                rect.min.y,
                rect.max.x - offset,
                rect.max.y,
            )
        }
        UiAnimationType::SlideRight => {
            let offset = rect.width() * (1.0 - progress);
            Rect::new(
                rect.min.x + offset,
                rect.min.y,
                rect.max.x + offset,
                rect.max.y,
            )
        }
        UiAnimationType::SlideUp => {
            let offset = rect.height() * (1.0 - progress);
            Rect::new(
                rect.min.x,
                rect.min.y - offset,
                rect.max.x,
                rect.max.y - offset,
            )
        }
        UiAnimationType::SlideDown => {
            let offset = rect.height() * (1.0 - progress);
            Rect::new(
                rect.min.x,
                rect.min.y + offset,
                rect.max.x,
                rect.max.y + offset,
            )
        }
        UiAnimationType::Scale => {
            let center = rect.center();
            let half_w = rect.width() * 0.5 * progress;
            let half_h = rect.height() * 0.5 * progress;
            Rect::new(
                center.x - half_w,
                center.y - half_h,
                center.x + half_w,
                center.y + half_h,
            )
        }
    }
}

fn apply_outro_transform(rect: Rect, anim_type: UiAnimationType, progress: f32) -> Rect {
    match anim_type {
        UiAnimationType::Fade => rect,
        UiAnimationType::SlideLeft => {
            let offset = rect.width() * progress;
            Rect::new(
                rect.min.x - offset,
                rect.min.y,
                rect.max.x - offset,
                rect.max.y,
            )
        }
        UiAnimationType::SlideRight => {
            let offset = rect.width() * progress;
            Rect::new(
                rect.min.x + offset,
                rect.min.y,
                rect.max.x + offset,
                rect.max.y,
            )
        }
        UiAnimationType::SlideUp => {
            let offset = rect.height() * progress;
            Rect::new(
                rect.min.x,
                rect.min.y - offset,
                rect.max.x,
                rect.max.y - offset,
            )
        }
        UiAnimationType::SlideDown => {
            let offset = rect.height() * progress;
            Rect::new(
                rect.min.x,
                rect.min.y + offset,
                rect.max.x,
                rect.max.y + offset,
            )
        }
        UiAnimationType::Scale => {
            let center = rect.center();
            let scale = 1.0 - progress;
            let half_w = rect.width() * 0.5 * scale;
            let half_h = rect.height() * 0.5 * scale;
            Rect::new(
                center.x - half_w,
                center.y - half_h,
                center.x + half_w,
                center.y + half_h,
            )
        }
    }
}
