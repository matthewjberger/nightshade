use crate::ecs::{GameWorld, TowerType, UiAction};
use crate::systems::{
    can_place_tower_at, get_grid_position_from_mouse, mark_cell_occupied, sell_tower,
    spawn_money_popup, spawn_tower,
};
use nightshade::prelude::*;

pub fn input_system(game_world: &mut GameWorld, world: &mut World) {
    apply_ui_actions(game_world, world);

    game_world.resources.mouse_grid_pos = get_grid_position_from_mouse(game_world, world);

    if let Some(prev_text) = game_world.resources.hover_tower_text.take() {
        queue_ecs_command(world, EcsCommand::DespawnRecursive { entity: prev_text });
    }

    if let Some((grid_x, grid_z)) = game_world.resources.mouse_grid_pos
        && let Some(&tower_entity) = game_world
            .resources
            .towers_by_position
            .get(&(grid_x, grid_z))
        && let Some(tower) = game_world.get_tower(tower_entity)
    {
        let sell_value = (tower.tower_type.cost() as f32 * 0.7) as u32;

        let text_index = world
            .resources
            .text
            .cache
            .add_text(format!("${}", sell_value));
        let text_entity = spawn_entities(
            world,
            NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | TEXT | VISIBILITY,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(text_entity) {
            *name = Name("Hover Tower Text".to_string());
        }

        if let Some(transform) = world.core.get_local_transform_mut(text_entity) {
            transform.translation = nalgebra_glm::vec3(grid_x as f32, 2.0, grid_z as f32);
        }

        if let Some(text_component) = world.core.get_text_mut(text_entity) {
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

        game_world.resources.hover_tower_text = Some(text_entity);
    }

    let pointer_over_ui = world
        .resources
        .retained_ui
        .interaction
        .hovered_entity
        .is_some();
    let mouse_state = world.resources.input.mouse.state;
    let mouse_position = world.resources.input.mouse.position;
    let left_just_pressed = mouse_state.contains(MouseState::LEFT_JUST_PRESSED);
    let left_just_released = mouse_state.contains(MouseState::LEFT_JUST_RELEASED);
    let right_just_pressed = mouse_state.contains(MouseState::RIGHT_JUST_PRESSED);
    let right_just_released = mouse_state.contains(MouseState::RIGHT_JUST_RELEASED);

    const CLICK_DRAG_THRESHOLD: f32 = 5.0;

    if left_just_pressed && !pointer_over_ui {
        game_world.resources.left_press = Some(mouse_position);
    } else if left_just_pressed {
        game_world.resources.left_press = None;
    }

    if right_just_pressed && !pointer_over_ui {
        game_world.resources.right_press = Some(mouse_position);
    } else if right_just_pressed {
        game_world.resources.right_press = None;
    }

    if left_just_released
        && let Some(press_pos) = game_world.resources.left_press.take()
        && (mouse_position - press_pos).magnitude() < CLICK_DRAG_THRESHOLD
        && !pointer_over_ui
        && let Some((grid_x, grid_z)) = game_world.resources.mouse_grid_pos
        && can_place_tower_at(game_world, grid_x, grid_z)
    {
        let tower_type = game_world.resources.selected_tower_type;
        if game_world.resources.money >= tower_type.cost() {
            let cost = tower_type.cost();
            spawn_tower(game_world, world, grid_x, grid_z, tower_type);
            mark_cell_occupied(game_world, grid_x, grid_z);
            spawn_money_popup(
                game_world,
                world,
                nalgebra_glm::vec3(grid_x as f32, 0.5, grid_z as f32),
                -(cost as i32),
            );
        }
    }

    if right_just_released
        && let Some(press_pos) = game_world.resources.right_press.take()
        && (mouse_position - press_pos).magnitude() < CLICK_DRAG_THRESHOLD
        && !pointer_over_ui
        && let Some((grid_x, grid_z)) = game_world.resources.mouse_grid_pos
        && let Some(&tower_entity) = game_world
            .resources
            .towers_by_position
            .get(&(grid_x, grid_z))
    {
        sell_tower(game_world, world, tower_entity, grid_x, grid_z);
    }

    let keyboard = &world.resources.input.keyboard;
    if keyboard.is_key_pressed(KeyCode::Digit1) {
        game_world.resources.selected_tower_type = TowerType::Basic;
    } else if keyboard.is_key_pressed(KeyCode::Digit2) {
        game_world.resources.selected_tower_type = TowerType::Frost;
    } else if keyboard.is_key_pressed(KeyCode::Digit3) {
        game_world.resources.selected_tower_type = TowerType::Cannon;
    } else if keyboard.is_key_pressed(KeyCode::Digit4) {
        game_world.resources.selected_tower_type = TowerType::Sniper;
    } else if keyboard.is_key_pressed(KeyCode::Digit5) {
        game_world.resources.selected_tower_type = TowerType::Poison;
    }

    if keyboard.is_key_pressed(KeyCode::BracketLeft) {
        game_world.resources.game_speed = (game_world.resources.game_speed - 0.5).max(0.5);
    } else if keyboard.is_key_pressed(KeyCode::BracketRight) {
        game_world.resources.game_speed = (game_world.resources.game_speed + 0.5).min(3.0);
    } else if keyboard.is_key_pressed(KeyCode::Backslash) {
        game_world.resources.game_speed = 1.0;
    }
}

fn apply_ui_actions(game_world: &mut GameWorld, _world: &mut World) {
    let actions = std::mem::take(&mut game_world.resources.ui_interaction.actions);
    for action in actions {
        match action {
            UiAction::SelectTower(tower_type) => {
                game_world.resources.selected_tower_type = tower_type;
            }
            UiAction::DecreaseSpeed => {
                game_world.resources.game_speed = (game_world.resources.game_speed - 0.5).max(0.5);
            }
            UiAction::IncreaseSpeed => {
                game_world.resources.game_speed = (game_world.resources.game_speed + 0.5).min(3.0);
            }
            UiAction::ResetSpeed => {
                game_world.resources.game_speed = 1.0;
            }
        }
    }
}
