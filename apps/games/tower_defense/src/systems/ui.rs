use crate::commands::Scene;
use crate::resources::TowerDefenseResources;
use nightshade::prelude::*;
use nightshade_render::commands::material_registry::material_registry_mutate_id;

pub fn ui_update_system(
    game_world: &mut TowerDefenseResources,
    build: &mut nightshade_ui::commands::builder::UiBuild<'_>,
    delta_time: f32,
) {
    if game_world.wave_announce_timer > 0.0 {
        game_world.wave_announce_timer = (game_world.wave_announce_timer - delta_time).max(0.0);
    }

    crate::commands::update(game_world, build);
}

pub fn tile_hover_system(game_world: &mut TowerDefenseResources, scene: &mut Scene<'_, '_>) {
    if let Some(last_pos) = game_world.last_hovered_tile
        && let Some(&tile_entity) = game_world.grid_tiles.get(&last_pos)
        && let Some(&original_color) = game_world.tile_original_colors.get(&last_pos)
        && let Some(material_ref) = scene
            .storage
            .get::<nightshade_render::components::material::MaterialRef>(tile_entity)
            .cloned()
    {
        material_registry_mutate_id(scene.registry, material_ref.id, |material| {
            material.base_color = original_color.into();
        });
    }

    if let Some(current_pos) = game_world.mouse_grid_pos {
        let pos = nalgebra_glm::vec3(current_pos.0 as f32, 0.0, current_pos.1 as f32);

        let start_pos = game_world.path[0];
        let start_half_size = 1.0;
        let is_start = (pos.x - start_pos.x).abs() <= start_half_size
            && (pos.z - start_pos.z).abs() <= start_half_size;

        let end_pos = game_world.path.last().unwrap();
        let end_half_size = 1.0;
        let is_end = (pos.x - end_pos.x).abs() <= end_half_size
            && (pos.z - end_pos.z).abs() <= end_half_size;

        let is_path = game_world.path.windows(2).any(|w| {
            let seg_start = w[0];
            let seg_end = w[1];
            let min_x = seg_start.x.min(seg_end.x);
            let max_x = seg_start.x.max(seg_end.x);
            let min_z = seg_start.z.min(seg_end.z);
            let max_z = seg_start.z.max(seg_end.z);
            pos.x >= min_x && pos.x <= max_x && pos.z >= min_z && pos.z <= max_z
        });

        if !is_start && !is_end && !is_path {
            if let Some(&tile_entity) = game_world.grid_tiles.get(&current_pos)
                && let Some(material_ref) = scene
                    .storage
                    .get::<nightshade_render::components::material::MaterialRef>(tile_entity)
                    .cloned()
            {
                material_registry_mutate_id(scene.registry, material_ref.id, |material| {
                    material.base_color = [1.0, 1.0, 0.0, 1.0];
                });
            }
            game_world.last_hovered_tile = Some(current_pos);
        } else {
            game_world.last_hovered_tile = None;
        }
    } else {
        game_world.last_hovered_tile = None;
    }
}
