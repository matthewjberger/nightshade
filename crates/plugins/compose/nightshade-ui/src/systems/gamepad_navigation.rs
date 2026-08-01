use crate::resources::NavDirection;
use crate::systems::picking::is_descendant_of;
use nightshade_input::contract::{Actions, EngineAction};
use nightshade_paint::data::Rect;

struct NavScope<'a> {
    storage: &'a nightshade::ecs::Storage,
    overlays: &'a crate::resources::RetainedUiOverlays,
}

pub fn ui_gamepad_navigation_system(
    runtime: nightshade::ecs::system_param::Res<crate::resources::RetainedUiRuntime>,
    timing: nightshade::ecs::system_param::Res<crate::resources::RetainedUiTiming>,
    mut nav: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiGamepadNav>,
    mut input: nightshade::ecs::system_param::ResMut<nightshade_platform::contract::input::Input>,
    actions: nightshade::ecs::system_param::Res<Actions<EngineAction>>,
    mut interaction: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiInteraction>,
    mut overlays: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiOverlays>,
    storage: &mut nightshade::ecs::Storage,
) {
    if !runtime.visible {
        return;
    }
    if !nav.enabled {
        return;
    }
    let now = timing.current_time;
    let deadzone = nav.deadzone;
    let initial_delay = nav.initial_repeat_delay;
    let repeat_interval = nav.repeat_interval;

    let direction = read_direction(&actions, deadzone);
    let activate = actions.just_pressed(EngineAction::Primary);
    let cancel = actions.just_pressed(EngineAction::Secondary);
    let prev_tab = actions.just_pressed(EngineAction::NavigateLeft);
    let next_tab = actions.just_pressed(EngineAction::NavigateRight);

    let should_navigate = match direction {
        Some(dir) => {
            if nav.held_direction == Some(dir) {
                if now >= nav.next_repeat_at {
                    nav.next_repeat_at = now + repeat_interval;
                    true
                } else {
                    false
                }
            } else {
                nav.held_direction = Some(dir);
                nav.next_repeat_at = now + initial_delay;
                true
            }
        }
        None => {
            nav.held_direction = None;
            false
        }
    };

    let scope = NavScope {
        storage,
        overlays: &overlays,
    };

    let mut focus = interaction.focused_entity;
    if should_navigate && let Some(dir) = direction {
        focus = focus_in_direction(&scope, focus, dir).or(focus);
    }
    if next_tab {
        focus = cycle_tab_focus(&scope, focus, false).or(focus);
    }
    if prev_tab {
        focus = cycle_tab_focus(&scope, focus, true).or(focus);
    }
    if activate && focus.is_none() {
        focus = collect_focusable(&scope)
            .first()
            .map(|(entity, _, _)| *entity);
    }

    if focus != interaction.focused_entity {
        interaction.focused_entity = focus;
        overlays.focus_ring_visible = true;
    }

    if activate
        && let Some(focused) = focus
        && let Some(node) = storage.get_mut::<crate::components::UiNodeInteraction>(focused)
        && !node.disabled
    {
        node.clicked = true;
    }

    if cancel {
        let keyboard = &mut input.keyboard;
        keyboard
            .just_pressed_keys
            .insert(nightshade_platform::KeyCode::Escape);
        keyboard.keystates.insert(
            nightshade_platform::KeyCode::Escape,
            nightshade_platform::ElementState::Pressed,
        );
        keyboard
            .frame_keys
            .push((nightshade_platform::KeyCode::Escape, true));
    }
}

fn read_direction(actions: &Actions<EngineAction>, deadzone: f32) -> Option<NavDirection> {
    let stick = actions.axis2(EngineAction::Move);
    let stick_x = stick.x;
    let stick_y = stick.y;

    let vertical = if actions.pressed(EngineAction::NavigateUp) || stick_y > deadzone {
        Some(NavDirection::Up)
    } else if actions.pressed(EngineAction::NavigateDown) || stick_y < -deadzone {
        Some(NavDirection::Down)
    } else {
        None
    };

    let horizontal = if actions.pressed(EngineAction::NavigateRight) || stick_x > deadzone {
        Some(NavDirection::Right)
    } else if actions.pressed(EngineAction::NavigateLeft) || stick_x < -deadzone {
        Some(NavDirection::Left)
    } else {
        None
    };

    if vertical.is_some() && horizontal.is_some() {
        if stick_y.abs() >= stick_x.abs() {
            vertical
        } else {
            horizontal
        }
    } else {
        vertical.or(horizontal)
    }
}

fn collect_focusable(scope: &NavScope<'_>) -> Vec<(nightshade::ecs::Entity, Rect, Option<i32>)> {
    let mut focusable = Vec::new();
    for entity in scope
        .storage
        .query_entities::<(crate::components::UiNodeInteraction,)>()
    {
        let Some(interaction) = scope
            .storage
            .get::<crate::components::UiNodeInteraction>(entity)
        else {
            continue;
        };
        if interaction.disabled {
            continue;
        }
        let Some(node) = scope.storage.get::<crate::components::UiLayoutNode>(entity) else {
            continue;
        };
        if !node.visible {
            continue;
        }
        if !crate::commands::widgets::world_layout::ui_chain_visible(
            &mut |entity| {
                scope
                    .storage
                    .get::<crate::components::UiLayoutNode>(entity)
                    .map(|node| node.visible)
            },
            &mut |entity| {
                scope
                    .storage
                    .get::<nightshade::ecs::ChildOf>(entity)
                    .map(|child_of| child_of.0)
            },
            entity,
        ) {
            continue;
        }
        let rect = node.computed_rect;
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            continue;
        }
        focusable.push((entity, rect, interaction.tab_index));
    }
    scope_to_active_overlay(scope, &mut focusable);
    focusable
}

fn scope_to_active_overlay(
    scope: &NavScope<'_>,
    focusable: &mut Vec<(nightshade::ecs::Entity, Rect, Option<i32>)>,
) {
    if let Some(modal) = scope.overlays.active_modal {
        focusable.retain(|(entity, _, _)| is_descendant_of(scope.storage, *entity, modal));
    } else if !scope.overlays.popup_entities.is_empty() {
        let popups = &scope.overlays.popup_entities;
        focusable.retain(|(entity, _, _)| {
            popups
                .iter()
                .any(|popup| is_descendant_of(scope.storage, *entity, *popup))
        });
    }
}

fn focus_in_direction(
    scope: &NavScope<'_>,
    current: Option<nightshade::ecs::Entity>,
    direction: NavDirection,
) -> Option<nightshade::ecs::Entity> {
    let focusable = collect_focusable(scope);
    if focusable.is_empty() {
        return None;
    }
    match current {
        Some(entity) => match focusable.iter().find(|(e, _, _)| *e == entity) {
            Some((_, rect, _)) => find_neighbor(&focusable, entity, *rect, direction)
                .or_else(|| fallback_endpoint(&focusable, direction)),
            None => fallback_endpoint(&focusable, direction),
        },
        None => fallback_endpoint(&focusable, direction),
    }
}

fn find_neighbor(
    focusable: &[(nightshade::ecs::Entity, Rect, Option<i32>)],
    current: nightshade::ecs::Entity,
    current_rect: Rect,
    direction: NavDirection,
) -> Option<nightshade::ecs::Entity> {
    let current_center = current_rect.center();
    let mut best: Option<(nightshade::ecs::Entity, f32)> = None;
    for (entity, rect, _) in focusable {
        if *entity == current {
            continue;
        }
        let center = rect.center();
        let delta = center - current_center;
        let in_direction = match direction {
            NavDirection::Up => delta.y < -1.0,
            NavDirection::Down => delta.y > 1.0,
            NavDirection::Left => delta.x < -1.0,
            NavDirection::Right => delta.x > 1.0,
        };
        if !in_direction {
            continue;
        }
        let (primary, secondary) = match direction {
            NavDirection::Up | NavDirection::Down => (delta.y.abs(), delta.x.abs()),
            NavDirection::Left | NavDirection::Right => (delta.x.abs(), delta.y.abs()),
        };
        let score = primary + secondary * 2.5;
        match best {
            Some((_, current_score)) if score >= current_score => {}
            _ => best = Some((*entity, score)),
        }
    }
    best.map(|(entity, _)| entity)
}

fn fallback_endpoint(
    focusable: &[(nightshade::ecs::Entity, Rect, Option<i32>)],
    direction: NavDirection,
) -> Option<nightshade::ecs::Entity> {
    focusable
        .iter()
        .min_by(|a, b| {
            let key_a = endpoint_key(direction, a.1.center());
            let key_b = endpoint_key(direction, b.1.center());
            key_a
                .partial_cmp(&key_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, _, _)| *entity)
}

fn endpoint_key(direction: NavDirection, center: nalgebra_glm::Vec2) -> f32 {
    match direction {
        NavDirection::Up => -center.y,
        NavDirection::Down => center.y,
        NavDirection::Left => -center.x,
        NavDirection::Right => center.x,
    }
}

fn cycle_tab_focus(
    scope: &NavScope<'_>,
    current: Option<nightshade::ecs::Entity>,
    reverse: bool,
) -> Option<nightshade::ecs::Entity> {
    let mut tabbable: Vec<(nightshade::ecs::Entity, i32)> = collect_focusable(scope)
        .into_iter()
        .filter_map(|(entity, _, tab_index)| tab_index.map(|index| (entity, index)))
        .collect();
    tabbable.sort_by_key(|(_, index)| *index);
    if tabbable.is_empty() {
        return None;
    }
    let current_index =
        current.and_then(|entity| tabbable.iter().position(|(stored, _)| *stored == entity));
    let next_index = if reverse {
        match current_index {
            Some(0) => tabbable.len() - 1,
            Some(index) => index - 1,
            None => tabbable.len() - 1,
        }
    } else {
        match current_index {
            Some(index) => (index + 1) % tabbable.len(),
            None => 0,
        }
    };
    Some(tabbable[next_index].0)
}
