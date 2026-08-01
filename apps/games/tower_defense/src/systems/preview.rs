use crate::commands::Scene;
use crate::components::{IndicatorLines, IndicatorVisible};
use crate::resources::TowerDefenseResources;
use crate::systems::can_place_tower_at;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;

pub fn range_indicator_system(game_world: &mut TowerDefenseResources, scene: &mut Scene<'_, '_>) {
    let entities: Vec<Entity> = scene
        .storage
        .query_entities::<(IndicatorLines,)>()
        .collect();
    let mut lines_to_hide: Vec<Entity> = Vec::new();
    for entity in entities {
        let showing = scene
            .storage
            .get::<IndicatorVisible>(entity)
            .copied()
            .unwrap_or_default();
        if showing.0 {
            scene.storage.set(entity, IndicatorVisible(false));
            if let Some(lines) = scene.storage.get::<IndicatorLines>(entity) {
                lines_to_hide.extend(lines.0.iter().copied());
            }
        }
    }
    for line_entity in lines_to_hide {
        if let Some(visibility) = scene
            .storage
            .get_mut::<nightshade_scene::contract::primitives::Visibility>(line_entity)
        {
            visibility.visible = false;
        }
    }

    if let Some((grid_x, grid_z)) = game_world.mouse_grid_pos
        && let Some(&tower_entity) = game_world.towers_by_position.get(&(grid_x, grid_z))
    {
        let mut lines_to_show: Vec<Entity> = Vec::new();
        if let Some(lines) = scene.storage.get::<IndicatorLines>(tower_entity) {
            lines_to_show.extend(lines.0.iter().copied());
        }
        if !lines_to_show.is_empty() {
            scene.storage.set(tower_entity, IndicatorVisible(true));
        }
        for line_entity in lines_to_show {
            if let Some(visibility) =
                scene
                    .storage
                    .get_mut::<nightshade_scene::contract::primitives::Visibility>(line_entity)
            {
                visibility.visible = true;
            }
        }
    }
}

pub fn placement_preview_system(game_world: &mut TowerDefenseResources, scene: &mut Scene<'_, '_>) {
    if let Some(preview) = game_world.preview_entity {
        queue_ecs_command(
            scene.commands,
            EcsCommand::DespawnRecursive { entity: preview },
        );
        game_world.preview_entity = None;
    }

    for entity in &game_world.preview_range_lines {
        queue_ecs_command(
            scene.commands,
            EcsCommand::DespawnRecursive { entity: *entity },
        );
    }
    game_world.preview_range_lines.clear();

    if let Some((grid_x, grid_z)) = game_world.mouse_grid_pos
        && can_place_tower_at(game_world, scene, grid_x, grid_z)
    {
        let tower_type = game_world.selected_tower_type;
        let can_afford = game_world.money >= crate::data::tower(tower_type).cost;
        let line_alpha = if can_afford { 0.5 } else { 0.2 };

        let position = nalgebra_glm::vec3(grid_x as f32, 0.0, grid_z as f32);

        let preview = {
            spawn_mesh(
                scene.commands,
                "Cylinder",
                position,
                nalgebra_glm::vec3(0.4, 0.8, 0.4),
            )
        };

        {
            set_material(
                scene.registry,
                scene.commands,
                preview,
                format!("Preview_{}", preview.id),
                Material {
                    base_color: [
                        crate::data::tower_color(tower_type).x,
                        crate::data::tower_color(tower_type).y,
                        crate::data::tower_color(tower_type).z,
                        1.0,
                    ],
                    ..Default::default()
                },
            );
        };

        game_world.preview_entity = Some(preview);

        let range = crate::data::tower(tower_type).range;
        let segments = 32;
        let tower_color = crate::data::tower_color(tower_type);

        for segment_index in 0..segments {
            let angle1 = (segment_index as f32 / segments as f32) * std::f32::consts::TAU;
            let angle2 = ((segment_index + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let start =
                position + nalgebra_glm::vec3(angle1.cos() * range, 0.1, angle1.sin() * range);
            let end =
                position + nalgebra_glm::vec3(angle2.cos() * range, 0.1, angle2.sin() * range);

            let line_entity = scene.commands.spawn(());

            scene.commands.set(
                line_entity,
                Name(format!("Preview Range Line {}", segment_index)),
            );
            scene.commands.set(
                line_entity,
                LocalTransform {
                    translation: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                    ..Default::default()
                },
            );
            scene.commands.set(line_entity, GlobalTransform::default());

            scene.commands.set(
                line_entity,
                Lines::new(vec![Line {
                    start,
                    end,
                    color: nalgebra_glm::vec4(
                        tower_color.x,
                        tower_color.y,
                        tower_color.z,
                        line_alpha,
                    ),
                }]),
            );

            scene
                .commands
                .set(line_entity, Visibility { visible: true });

            game_world.preview_range_lines.push(line_entity);
        }

        let box_size = 0.5;
        let box_height = 0.9;
        let y_base = -0.5;
        let y_top = y_base + box_height;

        let corners = [
            nalgebra_glm::vec3(box_size, 0.0, box_size),
            nalgebra_glm::vec3(-box_size, 0.0, box_size),
            nalgebra_glm::vec3(-box_size, 0.0, -box_size),
            nalgebra_glm::vec3(box_size, 0.0, -box_size),
        ];

        let mut box_lines = Vec::new();
        let box_color = if can_afford {
            nalgebra_glm::vec4(0.0, 1.0, 0.0, 0.8)
        } else {
            nalgebra_glm::vec4(1.0, 0.0, 0.0, 0.8)
        };

        for index in 0..4 {
            let next_index = (index + 1) % 4;
            box_lines.push(Line {
                start: position + corners[index] + nalgebra_glm::vec3(0.0, y_base, 0.0),
                end: position + corners[next_index] + nalgebra_glm::vec3(0.0, y_base, 0.0),
                color: box_color,
            });
            box_lines.push(Line {
                start: position + corners[index] + nalgebra_glm::vec3(0.0, y_top, 0.0),
                end: position + corners[next_index] + nalgebra_glm::vec3(0.0, y_top, 0.0),
                color: box_color,
            });
            box_lines.push(Line {
                start: position + corners[index] + nalgebra_glm::vec3(0.0, y_base, 0.0),
                end: position + corners[index] + nalgebra_glm::vec3(0.0, y_top, 0.0),
                color: box_color,
            });
        }

        let mark_y = 0.1;
        let mark_color = if can_afford {
            nalgebra_glm::vec4(0.0, 1.0, 0.0, 0.9)
        } else {
            nalgebra_glm::vec4(1.0, 0.0, 0.0, 0.9)
        };

        if can_afford {
            let check_size = 0.2;
            let check_left = -0.15;
            let check_middle = -0.05;
            let check_right = 0.15;

            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(check_left, mark_y, box_size),
                end: position + nalgebra_glm::vec3(check_middle, mark_y - check_size, box_size),
                color: mark_color,
            });
            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(check_middle, mark_y - check_size, box_size),
                end: position
                    + nalgebra_glm::vec3(check_right, mark_y + check_size * 0.5, box_size),
                color: mark_color,
            });

            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(check_left, mark_y, -box_size),
                end: position + nalgebra_glm::vec3(check_middle, mark_y - check_size, -box_size),
                color: mark_color,
            });
            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(check_middle, mark_y - check_size, -box_size),
                end: position
                    + nalgebra_glm::vec3(check_right, mark_y + check_size * 0.5, -box_size),
                color: mark_color,
            });

            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(box_size, mark_y, check_left),
                end: position + nalgebra_glm::vec3(box_size, mark_y - check_size, check_middle),
                color: mark_color,
            });
            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(box_size, mark_y - check_size, check_middle),
                end: position
                    + nalgebra_glm::vec3(box_size, mark_y + check_size * 0.5, check_right),
                color: mark_color,
            });

            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(-box_size, mark_y, check_left),
                end: position + nalgebra_glm::vec3(-box_size, mark_y - check_size, check_middle),
                color: mark_color,
            });
            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(-box_size, mark_y - check_size, check_middle),
                end: position
                    + nalgebra_glm::vec3(-box_size, mark_y + check_size * 0.5, check_right),
                color: mark_color,
            });
        } else {
            let x_size = 0.25;

            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(-x_size, mark_y - x_size, box_size),
                end: position + nalgebra_glm::vec3(x_size, mark_y + x_size, box_size),
                color: mark_color,
            });
            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(x_size, mark_y - x_size, box_size),
                end: position + nalgebra_glm::vec3(-x_size, mark_y + x_size, box_size),
                color: mark_color,
            });

            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(-x_size, mark_y - x_size, -box_size),
                end: position + nalgebra_glm::vec3(x_size, mark_y + x_size, -box_size),
                color: mark_color,
            });
            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(x_size, mark_y - x_size, -box_size),
                end: position + nalgebra_glm::vec3(-x_size, mark_y + x_size, -box_size),
                color: mark_color,
            });

            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(box_size, mark_y - x_size, -x_size),
                end: position + nalgebra_glm::vec3(box_size, mark_y + x_size, x_size),
                color: mark_color,
            });
            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(box_size, mark_y - x_size, x_size),
                end: position + nalgebra_glm::vec3(box_size, mark_y + x_size, -x_size),
                color: mark_color,
            });

            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(-box_size, mark_y - x_size, -x_size),
                end: position + nalgebra_glm::vec3(-box_size, mark_y + x_size, x_size),
                color: mark_color,
            });
            box_lines.push(Line {
                start: position + nalgebra_glm::vec3(-box_size, mark_y - x_size, x_size),
                end: position + nalgebra_glm::vec3(-box_size, mark_y + x_size, -x_size),
                color: mark_color,
            });
        }

        let box_entity = scene.commands.spawn(());

        let box_name = if can_afford {
            "Affordable Box"
        } else {
            "Unaffordable Box"
        };
        scene.commands.set(box_entity, Name(box_name.to_string()));
        scene.commands.set(
            box_entity,
            LocalTransform {
                translation: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                ..Default::default()
            },
        );
        scene.commands.set(box_entity, GlobalTransform::default());
        scene.commands.set(box_entity, Lines::new(box_lines));
        scene.commands.set(box_entity, Visibility { visible: true });

        game_world.preview_range_lines.push(box_entity);
    }
}
