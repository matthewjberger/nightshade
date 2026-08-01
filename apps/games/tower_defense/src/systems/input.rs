use crate::commands::Scene;
use crate::data::{Ask, TowerType, UiAction};
use crate::resources::TowerDefenseResources;
use crate::systems::{
    can_place_tower_at, get_grid_position_from_mouse, mark_cell_occupied, sell_tower,
    spawn_money_popup, spawn_tower,
};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_text::prelude::*;

pub fn input_system(
    game_world: &mut TowerDefenseResources,
    input: &Input,
    actions: &Actions<Ask>,
    view: &crate::systems::MouseView<'_>,
    pointer_over_ui: bool,
    scene: &mut Scene<'_, '_>,
) {
    apply_ui_actions(game_world);

    game_world.mouse_grid_pos = get_grid_position_from_mouse(input, view, scene);

    if let Some(prev_text) = game_world.hover_tower_text.take() {
        queue_ecs_command(
            scene.commands,
            EcsCommand::DespawnRecursive { entity: prev_text },
        );
    }

    if let Some((grid_x, grid_z)) = game_world.mouse_grid_pos
        && let Some(&tower_entity) = game_world.towers_by_position.get(&(grid_x, grid_z))
        && let Some(tower_type) = scene.storage.get::<TowerType>(tower_entity).copied()
    {
        let sell_value = (crate::data::tower(tower_type).cost as f32 * 0.7) as u32;

        let text_index = scene.text_state.cache.add_text(format!("${}", sell_value));
        let text_entity = scene.commands.spawn(());

        scene
            .commands
            .set(text_entity, Name("Hover Tower Text".to_string()));
        scene.commands.set(
            text_entity,
            LocalTransform {
                translation: nalgebra_glm::vec3(grid_x as f32, 2.0, grid_z as f32),
                ..Default::default()
            },
        );
        scene.commands.set(text_entity, GlobalTransform::default());
        scene.commands.set(
            text_entity,
            nightshade_scene::contract::primitives::Visibility { visible: true },
        );
        scene.commands.set(
            text_entity,
            nightshade_text::contract::Text {
                text_index,
                properties: TextProperties {
                    font_size: 36.0,
                    color: nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0),
                    alignment: TextAlignment::Center,
                    outline_width: 0.08,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.1,
                    ..Default::default()
                },
                dirty: true,
                ..Default::default()
            },
        );

        game_world.hover_tower_text = Some(text_entity);
    }

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
        && can_place_tower_at(game_world, scene, grid_x, grid_z)
    {
        let tower_type = game_world.selected_tower_type;
        if game_world.money >= crate::data::tower(tower_type).cost {
            let cost = crate::data::tower(tower_type).cost;
            spawn_tower(game_world, scene, grid_x, grid_z, tower_type);
            mark_cell_occupied(scene, grid_x, grid_z);
            spawn_money_popup(
                scene,
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
        sell_tower(game_world, scene, tower_entity, grid_x, grid_z);
    }

    if actions.pressed(Ask::PickBasic) {
        game_world.selected_tower_type = TowerType::Basic;
    } else if actions.pressed(Ask::PickFrost) {
        game_world.selected_tower_type = TowerType::Frost;
    } else if actions.pressed(Ask::PickCannon) {
        game_world.selected_tower_type = TowerType::Cannon;
    } else if actions.pressed(Ask::PickSniper) {
        game_world.selected_tower_type = TowerType::Sniper;
    } else if actions.pressed(Ask::PickPoison) {
        game_world.selected_tower_type = TowerType::Poison;
    }

    if actions.pressed(Ask::SlowDown) {
        game_world.game_speed = (game_world.game_speed - 0.5).max(0.5);
    } else if actions.pressed(Ask::SpeedUp) {
        game_world.game_speed = (game_world.game_speed + 0.5).min(3.0);
    } else if actions.pressed(Ask::ResetSpeed) {
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
