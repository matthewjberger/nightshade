use nalgebra_glm::{Vec3, vec3};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_kit::prelude::*;
use nightshade_load::{LoadHandle, load_model_with_files};
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn ask_for(loading: &mut nightshade_gltf::Loading, name: &str) -> LoadHandle {
    let file = &MODELS[model(name)];
    load_model_with_files(
        loading,
        file.bytes.to_vec(),
        kit_textures(file.kit)
            .iter()
            .map(|(path, bytes)| ((*path).to_string(), bytes.to_vec())),
    )
}

pub fn lay_the_ground(
    commands: &mut Commands,
    materials: &mut MaterialRegistry,
    reach: f32,
    tint: [f32; 4],
) -> Entity {
    let entity = spawn_mesh_at(
        commands,
        "Cube",
        vec3(0.0, -0.5, 0.0),
        vec3(reach * 2.0, 1.0, reach * 2.0),
    );
    set_material(
        materials,
        commands,
        entity,
        "kit_ground".to_string(),
        Material {
            base_color: tint,
            roughness: 0.95,
            metallic: 0.0,
            ..Default::default()
        },
    );
    entity
}

pub fn stand(storage: &mut nightshade::ecs::Storage, entity: Entity, yaw: f32, squeeze: f32) {
    if let Some(transform) = storage.get_mut::<nightshade_scene::contract::LocalTransform>(entity) {
        transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y_axis());
        transform.scale = vec3(1.0, 1.0, squeeze);
    }
}
