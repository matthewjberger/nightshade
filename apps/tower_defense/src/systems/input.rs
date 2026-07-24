use crate::ecs::{Tower, TowerDefenseResources, TowerType, UiAction};
use crate::systems::{
    can_place_tower_at, get_grid_position_from_mouse, mark_cell_occupied, sell_tower,
    spawn_money_popup, spawn_tower,
};
use nightshade::prelude::*;

pub fn input_system(
    mut game_world: ResMut<TowerDefenseResources>,
    input: Res<Input>,
    world: &mut World,
) {
    let game_world = &mut *game_world;
    apply_ui_actions(game_world);

    game_world.mouse_grid_pos = get_grid_position_from_mouse(&input, world);

    if let Some(prev_text) = game_world.hover_tower_text.take() {
        queue_ecs_command(world, EcsCommand::DespawnRecursive { entity: prev_text });
    }

    if let Some((grid_x, grid_z)) = game_world.mouse_grid_pos
        && let Some(&tower_entity) = game_world.towers_by_position.get(&(grid_x, grid_z))
        && let Some(tower) = world.get::<Tower>(tower_entity).copied()
    {
        let sell_value = (tower.tower_type.cost() as f32 * 0.7) as u32;

        let text_index = world
            .res_mut::<TextState>()
            .cache
            .add_text(format!("${}", sell_value));
        let text_entity = spawn_entities(
            world,
            NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | TEXT | VISIBILITY,
            1,
        )[0];

        if let Some(name) = world.get_mut::<nightshade::ecs::primitives::Name>(text_entity) {
            *name = Name("Hover Tower Text".to_string());
        }

        if let Some(transform) =
            world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(text_entity)
        {
            transform.translation = nalgebra_glm::vec3(grid_x as f32, 2.0, grid_z as f32);
        }

        if let Some(text_component) =
            world.get_mut::<nightshade::ecs::text::components::Text>(text_entity)
        {
            text_component.text_index = text_index;
            text_component.properties = TextProperties {
                font_size: 36.0,
                color: nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0),
                alignment: TextAlignment::Center,
                outline_width: 0.08,
                outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                smoothing: 0.1,
                ..Default::default()
            };
            text_component.dirty = true;
        }

        game_world.hover_tower_text = Some(text_entity);
    }

    let pointer_over_ui = world
        .res::<RetainedUiInteraction>()
        .hovered_entity
        .is_some();
    let mouse_state = input.mouse.state;
    let mouse_position = input.mouse.position;
    let left_just_pressed = mouse_state.contains(MouseState::LEFT_JUST_PRESSED);
    let left_just_released = mouse_state.contains(MouseState::LEFT_JUST_RELEASED);
    let right_just_pressed = mouse_state.contains(MouseState::RIGHT_JUST_PRESSED);
    let right_just_released = mouse_state.contains(MouseState::RIGHT_JUST_RELEASED);

    const CLICK_DRAG_THRESHOLD: f32 = 5.0;

    if left_just_pressed && !pointer_over_ui {
        game_world.left_press = Some(mouse_position);
    } else if left_just_pressed {
        game_world.left_press = None;
    }

    if right_just_pressed && !pointer_over_ui {
        game_world.right_press = Some(mouse_position);
    } else if right_just_pressed {
        game_world.right_press = None;
    }

    if left_just_released
        && let Some(press_pos) = game_world.left_press.take()
        && (mouse_position - press_pos).magnitude() < CLICK_DRAG_THRESHOLD
        && !pointer_over_ui
        && let Some((grid_x, grid_z)) = game_world.mouse_grid_pos
        && can_place_tower_at(game_world, world, grid_x, grid_z)
    {
        let tower_type = game_world.selected_tower_type;
        if game_world.money >= tower_type.cost() {
            let cost = tower_type.cost();
            spawn_tower(game_world, world, grid_x, grid_z, tower_type);
            mark_cell_occupied(world, grid_x, grid_z);
            spawn_money_popup(
                world,
                nalgebra_glm::vec3(grid_x as f32, 0.5, grid_z as f32),
                -(cost as i32),
            );
        }
    }

    if right_just_released
        && let Some(press_pos) = game_world.right_press.take()
        && (mouse_position - press_pos).magnitude() < CLICK_DRAG_THRESHOLD
        && !pointer_over_ui
        && let Some((grid_x, grid_z)) = game_world.mouse_grid_pos
        && let Some(&tower_entity) = game_world.towers_by_position.get(&(grid_x, grid_z))
    {
        sell_tower(game_world, world, tower_entity, grid_x, grid_z);
    }

    let keyboard = &input.keyboard;
    if keyboard.is_key_pressed(KeyCode::Digit1) {
        game_world.selected_tower_type = TowerType::Basic;
    } else if keyboard.is_key_pressed(KeyCode::Digit2) {
        game_world.selected_tower_type = TowerType::Frost;
    } else if keyboard.is_key_pressed(KeyCode::Digit3) {
        game_world.selected_tower_type = TowerType::Cannon;
    } else if keyboard.is_key_pressed(KeyCode::Digit4) {
        game_world.selected_tower_type = TowerType::Sniper;
    } else if keyboard.is_key_pressed(KeyCode::Digit5) {
        game_world.selected_tower_type = TowerType::Poison;
    }

    if keyboard.is_key_pressed(KeyCode::BracketLeft) {
        game_world.game_speed = (game_world.game_speed - 0.5).max(0.5);
    } else if keyboard.is_key_pressed(KeyCode::BracketRight) {
        game_world.game_speed = (game_world.game_speed + 0.5).min(3.0);
    } else if keyboard.is_key_pressed(KeyCode::Backslash) {
        game_world.game_speed = 1.0;
    }
}

fn apply_ui_actions(game_world: &mut TowerDefenseResources) {
    let actions = std::mem::take(&mut game_world.ui_interaction.actions);
    for action in actions {
        match action {
            UiAction::SelectTower(tower_type) => {
                game_world.selected_tower_type = tower_type;
            }
            UiAction::DecreaseSpeed => {
                game_world.game_speed = (game_world.game_speed - 0.5).max(0.5);
            }
            UiAction::IncreaseSpeed => {
                game_world.game_speed = (game_world.game_speed + 0.5).min(3.0);
            }
            UiAction::ResetSpeed => {
                game_world.game_speed = 1.0;
            }
        }
    }
}
