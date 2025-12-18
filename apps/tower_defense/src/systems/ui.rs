use crate::ecs::{GameWorld, TowerType};
use nightshade::ecs::generational_registry::registry_entry_by_name_mut;
use nightshade::prelude::*;

pub fn update_lives_bar(game_world: &mut GameWorld, world: &mut World) {
    if let Some(bar_entity) = game_world.resources.ui_handles.lives_bar {
        let total_hp = (game_world.resources.lives - 1) * game_world.resources.max_hp
            + game_world.resources.current_hp;
        let max_total_hp = game_world.resources.lives * game_world.resources.max_hp;
        let health_percentage = total_hp as f32 / max_total_hp as f32;
        let bar_width = 5.8 * health_percentage;

        if let Some(transform) = world.get_local_transform_mut(bar_entity) {
            transform.translation = nalgebra_glm::vec3(-3.9 + bar_width / 2.0, 1.0, -7.9);
            transform.scale = nalgebra_glm::vec3(bar_width, 0.25, 0.1);
            world.set_local_transform_dirty(bar_entity, LocalTransformDirty);
        }

        if let Some(material_ref) = world.get_material_ref(bar_entity).cloned()
            && let Some(material) = registry_entry_by_name_mut(
                &mut world.resources.material_registry.registry,
                &material_ref.name,
            )
        {
            material.base_color = if health_percentage > 0.5 {
                [0.0, 1.0, 0.0, 1.0]
            } else if health_percentage > 0.25 {
                [1.0, 1.0, 0.0, 1.0]
            } else {
                [1.0, 0.0, 0.0, 1.0]
            };
        }
    }
}

pub fn ui_update_system(game_world: &mut GameWorld, world: &mut World) {
    update_lives_bar(game_world, world);

    if let Some(money_entity) = game_world.resources.ui_handles.money_text
        && let Some(text) = world.get_text(money_entity)
    {
        world.resources.text_cache.set_text(
            text.text_index,
            format!("Money: ${}", game_world.resources.money),
        );
        if let Some(text_mut) = world.get_text_mut(money_entity) {
            text_mut.dirty = true;
        }
    }

    if let Some(lives_entity) = game_world.resources.ui_handles.lives_text
        && let Some(text) = world.get_text(lives_entity)
    {
        world.resources.text_cache.set_text(
            text.text_index,
            format!("Lives: {}", game_world.resources.lives),
        );
        if let Some(text_mut) = world.get_text_mut(lives_entity) {
            text_mut.dirty = true;
        }
    }

    if let Some(hp_entity) = game_world.resources.ui_handles.hp_text
        && let Some(text) = world.get_text(hp_entity)
    {
        world.resources.text_cache.set_text(
            text.text_index,
            format!(
                "HP: {}/{}",
                game_world.resources.current_hp, game_world.resources.max_hp
            ),
        );
        if let Some(text_mut) = world.get_text_mut(hp_entity) {
            text_mut.dirty = true;

            let hp_ratio =
                game_world.resources.current_hp as f32 / game_world.resources.max_hp as f32;
            text_mut.properties.color = if hp_ratio > 0.5 {
                nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0)
            } else if hp_ratio > 0.25 {
                nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0)
            } else {
                nalgebra_glm::vec4(1.0, 0.0, 0.0, 1.0)
            };
        }
    }

    if let Some(wave_entity) = game_world.resources.ui_handles.wave_text
        && let Some(text) = world.get_text(wave_entity)
    {
        world.resources.text_cache.set_text(
            text.text_index,
            format!("Wave: {}", game_world.resources.wave),
        );
        if let Some(text_mut) = world.get_text_mut(wave_entity) {
            text_mut.dirty = true;
        }
    }

    if let Some(wave_announce_entity) = game_world.resources.ui_handles.wave_announce_text {
        if game_world.resources.wave_announce_timer > 0.0 {
            let delta_time =
                world.resources.window.timing.delta_time * game_world.resources.game_speed;
            game_world.resources.wave_announce_timer -= delta_time;

            if let Some(text) = world.get_text(wave_announce_entity) {
                let wave_num = game_world.resources.wave;
                let is_boss_wave = wave_num.is_multiple_of(5);
                let announce_text = if is_boss_wave {
                    format!("BOSS WAVE {}", wave_num)
                } else {
                    format!("WAVE {}", wave_num)
                };
                world
                    .resources
                    .text_cache
                    .set_text(text.text_index, announce_text);
                if let Some(text_mut) = world.get_text_mut(wave_announce_entity) {
                    text_mut.dirty = true;
                }
            }

            if let Some(visibility) = world.get_visibility_mut(wave_announce_entity) {
                visibility.visible = true;
            }
        } else if let Some(visibility) = world.get_visibility_mut(wave_announce_entity) {
            visibility.visible = false;
        }
    }
}

pub fn update_tower_selection_hud(game_world: &mut GameWorld, world: &mut World) {
    let selected = game_world.resources.selected_tower_type;
    for (index, tower_type) in TowerType::all().iter().enumerate() {
        if let Some(&entity) = game_world
            .resources
            .ui_handles
            .tower_select_texts
            .get(index)
        {
            let is_selected = *tower_type == selected;
            let base_color = tower_type.color();
            let text_color = if is_selected {
                nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0)
            } else {
                nalgebra_glm::vec4(base_color.x, base_color.y, base_color.z, 0.7)
            };

            if let Some(hud_text) = world.get_hud_text_mut(entity) {
                hud_text.properties.color = text_color;
                hud_text.dirty = true;
            }
        }
    }
}

pub fn tile_hover_system(game_world: &mut GameWorld, world: &mut World) {
    if let Some(last_pos) = game_world.resources.last_hovered_tile
        && let Some(&tile_entity) = game_world.resources.grid_tiles.get(&last_pos)
        && let Some(&original_color) = game_world.resources.tile_original_colors.get(&last_pos)
        && let Some(material_ref) = world.get_material_ref(tile_entity).cloned()
        && let Some(material) = registry_entry_by_name_mut(
            &mut world.resources.material_registry.registry,
            &material_ref.name,
        )
    {
        material.base_color = original_color.into();
    }

    if let Some(current_pos) = game_world.resources.mouse_grid_pos {
        let pos = nalgebra_glm::vec3(current_pos.0 as f32, 0.0, current_pos.1 as f32);

        let start_pos = game_world.resources.path[0];
        let start_half_size = 1.0;
        let is_start = (pos.x - start_pos.x).abs() <= start_half_size
            && (pos.z - start_pos.z).abs() <= start_half_size;

        let end_pos = game_world.resources.path.last().unwrap();
        let end_half_size = 1.0;
        let is_end = (pos.x - end_pos.x).abs() <= end_half_size
            && (pos.z - end_pos.z).abs() <= end_half_size;

        let is_path = game_world.resources.path.windows(2).any(|w| {
            let seg_start = w[0];
            let seg_end = w[1];
            let min_x = seg_start.x.min(seg_end.x);
            let max_x = seg_start.x.max(seg_end.x);
            let min_z = seg_start.z.min(seg_end.z);
            let max_z = seg_start.z.max(seg_end.z);
            pos.x >= min_x && pos.x <= max_x && pos.z >= min_z && pos.z <= max_z
        });

        if !is_start && !is_end && !is_path {
            if let Some(&tile_entity) = game_world.resources.grid_tiles.get(&current_pos)
                && let Some(material_ref) = world.get_material_ref(tile_entity).cloned()
                && let Some(material) = registry_entry_by_name_mut(
                    &mut world.resources.material_registry.registry,
                    &material_ref.name,
                )
            {
                material.base_color = [1.0, 1.0, 0.0, 1.0];
            }
            game_world.resources.last_hovered_tile = Some(current_pos);
        } else {
            game_world.resources.last_hovered_tile = None;
        }
    } else {
        game_world.resources.last_hovered_tile = None;
    }
}
