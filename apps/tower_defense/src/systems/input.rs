use crate::ecs::{GameWorld, TowerType};
use crate::systems::{
    can_place_tower_at, get_grid_position_from_mouse, mark_cell_occupied, sell_tower,
    spawn_money_popup, spawn_tower, update_tower_selection_hud,
};
use nightshade::prelude::*;

pub fn input_system(game_world: &mut GameWorld, world: &mut World) {
    game_world.resources.mouse_grid_pos = get_grid_position_from_mouse(game_world, world);

    if let Some(prev_text) = game_world.resources.hover_tower_text.take() {
        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive { entity: prev_text });
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
            .text_cache
            .add_text(format!("${}", sell_value));
        let text_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | TEXT | VISIBILITY,
            1,
        )[0];

        if let Some(name) = world.get_name_mut(text_entity) {
            *name = Name("Hover Tower Text".to_string());
        }

        if let Some(transform) = world.get_local_transform_mut(text_entity) {
            transform.translation = nalgebra_glm::vec3(grid_x as f32, 2.0, grid_z as f32);
        }

        if let Some(text_component) = world.get_text_mut(text_entity) {
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

    let left_clicked = world
        .resources
        .input
        .mouse
        .state
        .contains(MouseState::LEFT_CLICKED);
    let right_clicked = world
        .resources
        .input
        .mouse
        .state
        .contains(MouseState::RIGHT_CLICKED);

    let mut clicked_tower_hud = false;
    if left_clicked {
        let mouse_pos = world.resources.input.mouse.position;
        let hud_x_min = 10.0;
        let hud_x_max = 200.0;
        let hud_y_start = 10.0;
        let line_height = 28.0;
        let tower_count = TowerType::all().len();

        if mouse_pos.x >= hud_x_min && mouse_pos.x <= hud_x_max {
            for index in 0..tower_count {
                let y_min = hud_y_start + (index as f32 * line_height);
                let y_max = y_min + line_height;
                if mouse_pos.y >= y_min && mouse_pos.y <= y_max {
                    let old_selection = game_world.resources.selected_tower_type;
                    game_world.resources.selected_tower_type = TowerType::all()[index];
                    if old_selection != game_world.resources.selected_tower_type {
                        update_tower_selection_hud(game_world, world);
                    }
                    clicked_tower_hud = true;
                    break;
                }
            }
        }
    }

    if left_clicked
        && !clicked_tower_hud
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

    if right_clicked
        && let Some((grid_x, grid_z)) = game_world.resources.mouse_grid_pos
        && let Some(&tower_entity) = game_world
            .resources
            .towers_by_position
            .get(&(grid_x, grid_z))
    {
        sell_tower(game_world, world, tower_entity, grid_x, grid_z);
    }

    let key_1 = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::Digit1);
    let key_2 = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::Digit2);
    let key_3 = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::Digit3);
    let key_4 = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::Digit4);
    let key_5 = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::Digit5);
    let key_bracket_left = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::BracketLeft);
    let key_bracket_right = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::BracketRight);
    let key_backslash = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::Backslash);

    let old_selection = game_world.resources.selected_tower_type;
    if key_1 {
        game_world.resources.selected_tower_type = TowerType::Basic;
    } else if key_2 {
        game_world.resources.selected_tower_type = TowerType::Frost;
    } else if key_3 {
        game_world.resources.selected_tower_type = TowerType::Cannon;
    } else if key_4 {
        game_world.resources.selected_tower_type = TowerType::Sniper;
    } else if key_5 {
        game_world.resources.selected_tower_type = TowerType::Poison;
    }

    if old_selection != game_world.resources.selected_tower_type {
        update_tower_selection_hud(game_world, world);
    }

    if key_bracket_left {
        game_world.resources.game_speed = (game_world.resources.game_speed - 0.5).max(0.5);
    } else if key_bracket_right {
        game_world.resources.game_speed = (game_world.resources.game_speed + 0.5).min(3.0);
    } else if key_backslash {
        game_world.resources.game_speed = 1.0;
    }
}
