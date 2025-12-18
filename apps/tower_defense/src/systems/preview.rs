use crate::ecs::{GameWorld, RANGE_INDICATOR};
use crate::systems::can_place_tower_at;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

pub fn range_indicator_system(game_world: &mut GameWorld, world: &mut World) {
    let entities: Vec<_> = game_world.query_entities(RANGE_INDICATOR).collect();
    for entity in entities {
        if let Some(indicator) = game_world.get_range_indicator_mut(entity)
            && indicator.visible
        {
            indicator.visible = false;
            for &line_entity in &indicator.line_entities {
                if let Some(visibility) = world.get_visibility_mut(line_entity) {
                    visibility.visible = false;
                }
            }
        }
    }

    if let Some((grid_x, grid_z)) = game_world.resources.mouse_grid_pos
        && let Some(&tower_entity) = game_world
            .resources
            .towers_by_position
            .get(&(grid_x, grid_z))
    {
        let entities: Vec<_> = game_world.query_entities(RANGE_INDICATOR).collect();
        for entity in entities {
            if let Some(indicator) = game_world.get_range_indicator_mut(entity)
                && indicator.tower_entity == tower_entity
            {
                indicator.visible = true;
                for &line_entity in &indicator.line_entities {
                    if let Some(visibility) = world.get_visibility_mut(line_entity) {
                        visibility.visible = true;
                    }
                }
                break;
            }
        }
    }
}

pub fn placement_preview_system(game_world: &mut GameWorld, world: &mut World) {
    if let Some(preview) = game_world.resources.preview_entity {
        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive { entity: preview });
        game_world.resources.preview_entity = None;
    }

    for entity in &game_world.resources.preview_range_lines {
        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive { entity: *entity });
    }
    game_world.resources.preview_range_lines.clear();

    if let Some((grid_x, grid_z)) = game_world.resources.mouse_grid_pos
        && can_place_tower_at(game_world, grid_x, grid_z)
    {
        let tower_type = game_world.resources.selected_tower_type;
        let can_afford = game_world.resources.money >= tower_type.cost();
        let line_alpha = if can_afford { 0.5 } else { 0.2 };

        let position = nalgebra_glm::vec3(grid_x as f32, 0.0, grid_z as f32);

        let preview = spawn_mesh(
            world,
            "Cylinder",
            position,
            nalgebra_glm::vec3(0.4, 0.8, 0.4),
        );

        let material_name = format!("Preview_{}", preview.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            Material {
                base_color: [
                    tower_type.color().x,
                    tower_type.color().y,
                    tower_type.color().z,
                    1.0,
                ],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(preview, MaterialRef::new(material_name));

        game_world.resources.preview_entity = Some(preview);

        let range = tower_type.range();
        let segments = 32;
        let tower_color = tower_type.color();

        for segment_index in 0..segments {
            let angle1 = (segment_index as f32 / segments as f32) * std::f32::consts::TAU;
            let angle2 = ((segment_index + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let start =
                position + nalgebra_glm::vec3(angle1.cos() * range, 0.1, angle1.sin() * range);
            let end =
                position + nalgebra_glm::vec3(angle2.cos() * range, 0.1, angle2.sin() * range);

            let line_entity = world.spawn_entities(
                NAME | LOCAL_TRANSFORM
                    | GLOBAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | LINES
                    | VISIBILITY,
                1,
            )[0];

            world.set_name(
                line_entity,
                Name(format!("Preview Range Line {}", segment_index)),
            );
            world.set_local_transform(
                line_entity,
                LocalTransform {
                    translation: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                    ..Default::default()
                },
            );
            world.set_global_transform(line_entity, GlobalTransform::default());
            world.set_local_transform_dirty(line_entity, LocalTransformDirty);

            world.set_lines(
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

            world.set_visibility(line_entity, Visibility { visible: true });

            game_world.resources.preview_range_lines.push(line_entity);
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

        let box_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | LINES | VISIBILITY,
            1,
        )[0];

        let box_name = if can_afford {
            "Affordable Box"
        } else {
            "Unaffordable Box"
        };
        world.set_name(box_entity, Name(box_name.to_string()));
        world.set_local_transform(
            box_entity,
            LocalTransform {
                translation: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                ..Default::default()
            },
        );
        world.set_global_transform(box_entity, GlobalTransform::default());
        world.set_local_transform_dirty(box_entity, LocalTransformDirty);
        world.set_lines(box_entity, Lines::new(box_lines));
        world.set_visibility(box_entity, Visibility { visible: true });

        game_world.resources.preview_range_lines.push(box_entity);
    }
}
