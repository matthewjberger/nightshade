use crate::components::{ThemeEffect, ThemeShadow, UiNodeContent, UiNodeEffect, UiNodeShadow};
use crate::resources::theme::UiTheme;

fn resolve_shadow(theme: &UiTheme, role: ThemeShadow) -> Option<UiNodeShadow> {
    Some(match role {
        ThemeShadow::PanelShadow => theme.panel_shadow,
        ThemeShadow::ButtonShadow => theme.button_shadow,
        ThemeShadow::ButtonShadowHover => theme.button_shadow_hover,
        ThemeShadow::ButtonShadowPressed => theme.button_shadow_pressed,
        ThemeShadow::ModalShadow => theme.modal_shadow,
        ThemeShadow::FocusGlow => theme.focus_glow?,
    })
}

fn resolve_effect(theme: &UiTheme, role: ThemeEffect) -> UiNodeEffect {
    let preset = match role {
        ThemeEffect::PanelEffect => theme.panel_effect,
        ThemeEffect::ButtonEffect => theme.button_effect,
        ThemeEffect::InputEffect => theme.input_effect,
    };
    let (kind, params) = preset.encode();
    UiNodeEffect { kind, params }
}

pub fn ui_theme_transition_tick_system(
    runtime: nightshade::ecs::system_param::Res<crate::resources::RetainedUiRuntime>,
    time: nightshade::ecs::system_param::Res<nightshade_platform::contract::time::Time>,
    mut theme_state: nightshade::ecs::system_param::ResMut<crate::resources::ThemeState>,
) {
    if !runtime.visible {
        return;
    }
    theme_state.tick_transition(time.delta_time);
}

pub fn ui_theme_apply_system(
    runtime: nightshade::ecs::system_param::Res<crate::resources::RetainedUiRuntime>,
    mut theme_state: nightshade::ecs::system_param::ResMut<crate::resources::ThemeState>,
    storage: &mut nightshade::ecs::Storage,
) {
    if !runtime.visible {
        return;
    }

    let generation = theme_state.generation;
    if generation == theme_state.last_applied_generation {
        return;
    }
    theme_state.last_applied_generation = generation;

    let theme = theme_state.active_theme().clone();

    let entities: Vec<nightshade::ecs::Entity> = storage
        .query_entities::<(crate::components::UiThemeBinding,)>()
        .collect();

    for entity in entities {
        let binding = match storage.get::<crate::components::UiThemeBinding>(entity) {
            Some(binding) => *binding,
            None => continue,
        };

        if let Some(node_color) = storage.get_mut::<crate::components::UiNodeColor>(entity) {
            for (state_index, role) in binding.color_roles.iter().enumerate() {
                if let Some(role) = role {
                    node_color.colors[state_index] = Some(role.resolve(&theme));
                }
            }
        }

        if let Some(border_role) = binding.border_color_role
            && let Some(content) = storage.get_mut::<crate::components::UiNodeContent>(entity)
            && let UiNodeContent::Rect { border_color, .. } = content
        {
            *border_color = border_role.resolve(&theme);
        }

        let has_any_shadow = binding.shadow_roles.iter().any(|role| role.is_some());
        if has_any_shadow {
            let mut state_shadows = [None; crate::data::state::STATE_COUNT];
            for (state_index, role) in binding.shadow_roles.iter().enumerate() {
                if let Some(role) = role {
                    state_shadows[state_index] = resolve_shadow(&theme, *role);
                }
            }
            storage.add_components::<(crate::components::UiNodeShadowStates,)>(entity);
            if let Some(slot) = storage.get_mut::<crate::components::UiNodeShadowStates>(entity) {
                slot.shadows = state_shadows;
            }
            let base_shadow = state_shadows[0].unwrap_or_default();
            storage.add_components::<(crate::components::UiNodeShadow,)>(entity);
            if let Some(slot) = storage.get_mut::<crate::components::UiNodeShadow>(entity) {
                *slot = base_shadow;
            }
        }

        if let Some(role) = binding.effect_role {
            let effect = resolve_effect(&theme, role);
            storage.add_components::<(crate::components::UiNodeEffect,)>(entity);
            if let Some(slot) = storage.get_mut::<crate::components::UiNodeEffect>(entity) {
                *slot = effect;
            }
        }
    }
}
