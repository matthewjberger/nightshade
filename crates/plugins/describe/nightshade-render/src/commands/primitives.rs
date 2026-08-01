use crate::commands::mesh_cache::mesh_cache_insert;
use crate::data::generational_registry::registry_add_reference;
use crate::data::procedural_meshes::{
    create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_plane_mesh, create_quad_mesh,
    create_sphere_mesh, create_subdivided_plane_mesh, create_torus_mesh,
};

pub fn register_primitive_mesh(resources: &mut nightshade::ecs::ResourceMap, mesh_name: &str) {
    if !resources
        .res::<crate::resources::mesh_cache::MeshCache>()
        .registry
        .name_to_index
        .contains_key(mesh_name)
    {
        let mesh = match mesh_name {
            "Cube" => Some(create_cube_mesh()),
            "Sphere" => Some(create_sphere_mesh(1.0, 16)),
            "Plane" => Some(create_plane_mesh(2.0)),
            "SubdividedPlane" => Some(create_subdivided_plane_mesh(2.0, 20)),
            "Torus" => Some(create_torus_mesh(1.0, 0.3, 32, 16)),
            "Cylinder" => Some(create_cylinder_mesh(0.5, 1.0, 16)),
            "Cone" => Some(create_cone_mesh(0.5, 1.0, 16)),
            "Quad" => Some(create_quad_mesh(1.0)),
            _ => None,
        };
        if let Some(mesh) = mesh {
            mesh_cache_insert(
                resources.res_mut::<crate::resources::mesh_cache::MeshCache>(),
                mesh_name.to_string(),
                mesh,
            );
        }
    }

    if let Some(&index) = resources
        .res::<crate::resources::mesh_cache::MeshCache>()
        .registry
        .name_to_index
        .get(mesh_name)
    {
        registry_add_reference(
            &mut resources
                .res_mut::<crate::resources::mesh_cache::MeshCache>()
                .registry,
            index,
        );
    }

    if let Some(&index) = resources
        .res::<crate::resources::material_registry::MaterialRegistry>()
        .registry
        .name_to_index
        .get("Default")
    {
        registry_add_reference(
            &mut resources
                .res_mut::<crate::resources::material_registry::MaterialRegistry>()
                .registry,
            index,
        );
    }
}
