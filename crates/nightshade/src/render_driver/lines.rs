use crate::ecs::world::CORE;
use crate::ecs::world::World;
use crate::render::bounding_volume::extract_frustum_planes;
use crate::render::wgpu::render_configs::{RenderBoundingVolume, RenderLine, RenderNormal};

pub fn gather_lines_data(world: &World, wireframe_enabled: bool) -> Vec<RenderLine> {
    let line_entities = world.ecs.worlds[CORE].query_entities(
        crate::ecs::world::LINES
            | crate::ecs::world::GLOBAL_TRANSFORM
            | crate::ecs::world::VISIBILITY,
    );
    let mut line_data = Vec::new();

    for entity in line_entities {
        if let Some(visibility) = world.get::<crate::ecs::primitives::Visibility>(entity)
            && visibility.visible
            && let Some(lines_component) = world.get::<crate::ecs::lines::components::Lines>(entity)
            && let Some(global_transform) =
                world.get::<crate::ecs::transform::components::GlobalTransform>(entity)
        {
            let transform = global_transform.0;
            let depth_mode: u32 = if lines_component.always_on_top { 1 } else { 0 };
            for line in &lines_component.lines {
                let start_world = transform
                    * nalgebra_glm::Vec4::new(line.start.x, line.start.y, line.start.z, 1.0);
                let end_world =
                    transform * nalgebra_glm::Vec4::new(line.end.x, line.end.y, line.end.z, 1.0);

                line_data.push(RenderLine {
                    start: nalgebra_glm::vec3(start_world.x, start_world.y, start_world.z),
                    end: nalgebra_glm::vec3(end_world.x, end_world.y, end_world.z),
                    color: nalgebra_glm::vec4(
                        line.color.x,
                        line.color.y,
                        line.color.z,
                        line.color.w,
                    ),
                    entity_id: entity.id,
                    depth_mode,
                });
            }
        }
    }

    if wireframe_enabled {
        append_wireframe_lines(world, &mut line_data);
    }

    line_data
}

const WIREFRAME_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn append_wireframe_lines(world: &World, line_data: &mut Vec<RenderLine>) {
    let camera_entity = world.res::<crate::ecs::camera::resources::ActiveCamera>().0;
    let mesh_cache = &world.res::<crate::render::mesh_cache::MeshCache>();

    for entity in world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::RENDER_MESH | crate::ecs::world::GLOBAL_TRANSFORM)
    {
        if Some(entity) == camera_entity {
            continue;
        }

        if let Some(visibility) = world.get::<crate::ecs::primitives::Visibility>(entity)
            && !visibility.visible
        {
            continue;
        }

        if let Some(render_layer) = world.get::<crate::render::render_layer::RenderLayer>(entity)
            && render_layer.0 == crate::render::render_layer::RenderLayer::OVERLAY
        {
            continue;
        }

        let Some(render_mesh) = world.get::<crate::ecs::mesh::components::RenderMesh>(entity)
        else {
            continue;
        };
        let Some(global_transform) =
            world.get::<crate::ecs::transform::components::GlobalTransform>(entity)
        else {
            continue;
        };

        let mesh = {
            let mut found_mesh = None;
            for (name, mesh) in crate::render::mesh_cache::mesh_cache_iter(mesh_cache) {
                if *name == render_mesh.name {
                    found_mesh = Some(mesh);
                    break;
                }
            }
            match found_mesh {
                Some(mesh) => mesh,
                None => continue,
            }
        };

        push_mesh_wireframe_edges(line_data, entity.id, &global_transform.0, mesh);
    }

    let instanced_objects: Vec<(
        nightshade_ecs::Entity,
        crate::render::config::InstancedObjectData,
    )> = world.ecs.worlds[crate::ecs::world::RENDER]
        .query_ref::<&crate::render::config::InstancedObjectData>()
        .iter()
        .map(|(entity, data)| (entity, data.clone()))
        .collect();
    for (entity, instanced) in &instanced_objects {
        let entity = *entity;
        let mesh = {
            let mut found_mesh = None;
            for (name, mesh) in crate::render::mesh_cache::mesh_cache_iter(mesh_cache) {
                if *name == instanced.mesh_name {
                    found_mesh = Some(mesh);
                    break;
                }
            }
            match found_mesh {
                Some(mesh) => mesh,
                None => continue,
            }
        };

        for model in &instanced.world_models {
            let world_matrix: nalgebra_glm::Mat4 = (*model).into();
            push_mesh_wireframe_edges(line_data, entity.id, &world_matrix, mesh);
        }
    }
}

fn push_mesh_wireframe_edges(
    line_data: &mut Vec<RenderLine>,
    entity_id: u32,
    transform: &nalgebra_glm::Mat4,
    mesh: &crate::render::geometry::Mesh,
) {
    for triangle in mesh.indices.chunks_exact(3) {
        let i0 = triangle[0] as usize;
        let i1 = triangle[1] as usize;
        let i2 = triangle[2] as usize;
        let Some(v0) = mesh.vertices.get(i0) else {
            continue;
        };
        let Some(v1) = mesh.vertices.get(i1) else {
            continue;
        };
        let Some(v2) = mesh.vertices.get(i2) else {
            continue;
        };
        for &(a, b) in &[
            (v0.position, v1.position),
            (v1.position, v2.position),
            (v2.position, v0.position),
        ] {
            let start = transform * nalgebra_glm::Vec4::new(a[0], a[1], a[2], 1.0);
            let end = transform * nalgebra_glm::Vec4::new(b[0], b[1], b[2], 1.0);
            line_data.push(RenderLine {
                start: nalgebra_glm::vec3(start.x, start.y, start.z),
                end: nalgebra_glm::vec3(end.x, end.y, end.z),
                color: nalgebra_glm::vec4(
                    WIREFRAME_COLOR[0],
                    WIREFRAME_COLOR[1],
                    WIREFRAME_COLOR[2],
                    WIREFRAME_COLOR[3],
                ),
                entity_id,
                depth_mode: 1,
            });
        }
    }
}

pub fn gather_bounding_volume_data(
    world: &World,
    show_bounding_volumes: bool,
) -> Option<Vec<RenderBoundingVolume>> {
    let show_selected_bounding_volume = world
        .res::<crate::render::config::DebugDraw>()
        .show_selected_bounding_volume;
    let selected_entity = world
        .res::<crate::ecs::graphics::selection::Selection>()
        .active_entity;
    if !show_bounding_volumes && !show_selected_bounding_volume {
        return None;
    }

    let camera_entity = world.res::<crate::ecs::camera::resources::ActiveCamera>().0;
    let mesh_cache = &world.res::<crate::render::mesh_cache::MeshCache>();
    let mut bv_data = Vec::new();

    for entity in world.ecs.worlds[CORE].query_entities(crate::ecs::world::BOUNDING_VOLUME) {
        if Some(entity) == camera_entity {
            continue;
        }

        if let Some(render_layer) = world.get::<crate::render::render_layer::RenderLayer>(entity)
            && render_layer.0 == crate::render::render_layer::RenderLayer::OVERLAY
        {
            continue;
        }

        let is_selected = selected_entity == Some(entity);

        if !show_bounding_volumes && !is_selected {
            continue;
        }

        let Some(bounding_volume) =
            world.get::<crate::render::bounding_volume::BoundingVolume>(entity)
        else {
            continue;
        };
        let Some(global_transform) =
            world.get::<crate::ecs::transform::components::GlobalTransform>(entity)
        else {
            continue;
        };

        let color = if is_selected {
            [1.0, 0.45, 0.0, 1.0]
        } else {
            [0.0, 1.0, 1.0, 1.0]
        };

        if world.ecs.worlds[CORE].entity_has_components(entity, crate::ecs::world::SKIN)
            && let Some(render_mesh) = world.get::<crate::ecs::mesh::components::RenderMesh>(entity)
            && let Some(mesh) = crate::render::generational_registry::registry_entry_by_name(
                &mesh_cache.registry,
                &render_mesh.name,
            )
            && let Some(vertices) = skinned_overlay_vertices(world, entity, mesh)
            && let Some((minimum, maximum)) = world_position_bounds(&vertices)
        {
            let center = (minimum + maximum) * 0.5;
            let half_extents = (maximum - minimum) * 0.5;
            bv_data.push(RenderBoundingVolume {
                center,
                half_extents,
                orientation: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                transform: nalgebra_glm::Mat4::identity(),
                color: nalgebra_glm::vec4(color[0], color[1], color[2], color[3]),
            });
            continue;
        }

        let obb = &bounding_volume.obb;
        bv_data.push(RenderBoundingVolume {
            center: nalgebra_glm::vec3(obb.center.x, obb.center.y, obb.center.z),
            half_extents: nalgebra_glm::vec3(
                obb.half_extents.x,
                obb.half_extents.y,
                obb.half_extents.z,
            ),
            orientation: nalgebra_glm::vec4(
                obb.orientation.i,
                obb.orientation.j,
                obb.orientation.k,
                obb.orientation.w,
            ),
            transform: global_transform.0,
            color: nalgebra_glm::vec4(color[0], color[1], color[2], color[3]),
        });
    }

    Some(bv_data)
}

fn sphere_in_frustum(
    center: &nalgebra_glm::Vec3,
    radius: f32,
    frustum_planes: &[nalgebra_glm::Vec4; 6],
) -> bool {
    for plane in frustum_planes {
        let distance = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;
        if distance < -radius {
            return false;
        }
    }
    true
}

/// CPU equivalent of the skinned vertex shader: applies morph displacements,
/// then blends each vertex through its joint matrices (joint global transform
/// times inverse bind matrix). Returns world-space position and normal pairs,
/// or `None` when the entity has no usable skin for the mesh.
fn skinned_overlay_vertices(
    world: &World,
    entity: nightshade_ecs::Entity,
    mesh: &crate::render::geometry::Mesh,
) -> Option<Vec<(nalgebra_glm::Vec3, nalgebra_glm::Vec3)>> {
    use crate::ecs::world::{Vec3, Vec4};

    let skin = world.get::<crate::ecs::skin::components::Skin>(entity)?;
    let skin_data = mesh.skin_data.as_ref()?;
    let joint_count = skin.joints.len().min(skin.inverse_bind_matrices.len());
    if joint_count == 0 {
        return None;
    }

    let mut joint_matrices = Vec::with_capacity(joint_count);
    for joint_index in 0..joint_count {
        let joint_global = world
            .get::<crate::ecs::transform::components::GlobalTransform>(skin.joints[joint_index])
            .map(|transform| transform.0)
            .unwrap_or_else(nalgebra_glm::Mat4::identity);
        joint_matrices.push(joint_global * skin.inverse_bind_matrices[joint_index]);
    }

    let morph = mesh
        .morph_targets
        .as_ref()
        .zip(world.get::<crate::ecs::morph::components::MorphWeights>(entity));

    let mut output = Vec::with_capacity(skin_data.skinned_vertices.len());
    for (vertex_index, vertex) in skin_data.skinned_vertices.iter().enumerate() {
        let mut position = Vec3::new(vertex.position[0], vertex.position[1], vertex.position[2]);
        let mut normal = Vec3::new(vertex.normal[0], vertex.normal[1], vertex.normal[2]);

        if let Some((morph_data, morph_weights)) = &morph {
            for (target_index, target) in morph_data.targets.iter().enumerate() {
                let weight = morph_weights.get_weight(target_index);
                if weight.abs() <= 0.0001 {
                    continue;
                }
                if let Some(displacement) = target.position_displacements.get(vertex_index) {
                    position +=
                        Vec3::new(displacement[0], displacement[1], displacement[2]) * weight;
                }
                if let Some(displacements) = &target.normal_displacements
                    && let Some(displacement) = displacements.get(vertex_index)
                {
                    normal += Vec3::new(displacement[0], displacement[1], displacement[2]) * weight;
                }
            }
            if normal.norm_squared() > 1.0e-12 {
                normal = normal.normalize();
            }
        }

        let mut skinned_position = Vec3::zeros();
        let mut skinned_normal = Vec3::zeros();
        for influence in 0..4 {
            let joint_weight = vertex.joint_weights[influence];
            if joint_weight <= 0.0 {
                continue;
            }
            let Some(joint_matrix) = joint_matrices.get(vertex.joint_indices[influence] as usize)
            else {
                continue;
            };
            let transformed = joint_matrix * Vec4::new(position.x, position.y, position.z, 1.0);
            skinned_position +=
                Vec3::new(transformed.x, transformed.y, transformed.z) * joint_weight;
            skinned_normal += nalgebra_glm::mat4_to_mat3(joint_matrix) * normal * joint_weight;
        }
        if skinned_normal.norm_squared() > 1.0e-12 {
            skinned_normal = skinned_normal.normalize();
        }
        output.push((skinned_position, skinned_normal));
    }
    Some(output)
}

fn world_position_bounds(
    vertices: &[(nalgebra_glm::Vec3, nalgebra_glm::Vec3)],
) -> Option<(nalgebra_glm::Vec3, nalgebra_glm::Vec3)> {
    let mut iterator = vertices.iter();
    let &(first, _) = iterator.next()?;
    let mut minimum = first;
    let mut maximum = first;
    for &(position, _) in iterator {
        minimum = nalgebra_glm::min2(&minimum, &position);
        maximum = nalgebra_glm::max2(&maximum, &position);
    }
    Some((minimum, maximum))
}

pub fn gather_normal_data(world: &World, show_normals: bool) -> Option<Vec<RenderNormal>> {
    let normal_line_length = world
        .res::<crate::render::config::DebugDraw>()
        .normal_line_length;
    let normal_line_color = world
        .res::<crate::render::config::DebugDraw>()
        .normal_line_color;
    if !show_normals {
        return None;
    }

    let camera_matrices = crate::ecs::camera::queries::query_active_camera_matrices(world)?;

    let view_proj = camera_matrices.projection * camera_matrices.view;
    let frustum_planes = extract_frustum_planes(&view_proj);

    let camera_entity = world.res::<crate::ecs::camera::resources::ActiveCamera>().0;
    let mesh_cache = &world.res::<crate::render::mesh_cache::MeshCache>();
    let mut normal_data = Vec::new();

    for entity in world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::RENDER_MESH | crate::ecs::world::GLOBAL_TRANSFORM)
    {
        if Some(entity) == camera_entity {
            continue;
        }

        if let Some(render_layer) = world.get::<crate::render::render_layer::RenderLayer>(entity)
            && render_layer.0 == crate::render::render_layer::RenderLayer::OVERLAY
        {
            continue;
        }

        let Some(render_mesh) = world.get::<crate::ecs::mesh::components::RenderMesh>(entity)
        else {
            continue;
        };

        let Some(global_transform) =
            world.get::<crate::ecs::transform::components::GlobalTransform>(entity)
        else {
            continue;
        };

        let is_skinned =
            world.ecs.worlds[CORE].entity_has_components(entity, crate::ecs::world::SKIN);

        if !is_skinned
            && let Some(bounding_volume) =
                world.get::<crate::render::bounding_volume::BoundingVolume>(entity)
        {
            let world_center = global_transform.0
                * nalgebra_glm::Vec4::new(
                    bounding_volume.obb.center.x,
                    bounding_volume.obb.center.y,
                    bounding_volume.obb.center.z,
                    1.0,
                );
            let world_center =
                nalgebra_glm::Vec3::new(world_center.x, world_center.y, world_center.z);

            let scale = nalgebra_glm::length(&nalgebra_glm::Vec3::new(
                global_transform.0[(0, 0)],
                global_transform.0[(1, 0)],
                global_transform.0[(2, 0)],
            ));
            let world_radius = bounding_volume.sphere_radius * scale;

            if !sphere_in_frustum(&world_center, world_radius, &frustum_planes) {
                continue;
            }
        }

        let mesh = {
            let mut found_mesh = None;
            for (name, mesh) in crate::render::mesh_cache::mesh_cache_iter(mesh_cache) {
                if *name == render_mesh.name {
                    found_mesh = Some(mesh);
                    break;
                }
            }
            match found_mesh {
                Some(m) => m,
                None => continue,
            }
        };

        if is_skinned && let Some(vertices) = skinned_overlay_vertices(world, entity, mesh) {
            for (position, normal) in vertices {
                normal_data.push(RenderNormal {
                    position,
                    normal,
                    transform: nalgebra_glm::Mat4::identity(),
                    color: nalgebra_glm::vec4(
                        normal_line_color[0],
                        normal_line_color[1],
                        normal_line_color[2],
                        normal_line_color[3],
                    ),
                    length: normal_line_length,
                });
            }
            continue;
        }

        for vertex in &mesh.vertices {
            normal_data.push(RenderNormal {
                position: nalgebra_glm::vec3(
                    vertex.position[0],
                    vertex.position[1],
                    vertex.position[2],
                ),
                normal: nalgebra_glm::vec3(vertex.normal[0], vertex.normal[1], vertex.normal[2]),
                transform: global_transform.0,
                color: nalgebra_glm::vec4(
                    normal_line_color[0],
                    normal_line_color[1],
                    normal_line_color[2],
                    normal_line_color[3],
                ),
                length: normal_line_length,
            });
        }
    }

    Some(normal_data)
}
