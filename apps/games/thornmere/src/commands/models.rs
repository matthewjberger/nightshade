use crate::components::{Fit, Look};
use crate::queries::ground;
use crate::resources::Models;
use nightshade::prelude::*;
use nightshade_gltf::Loading;
use nightshade_kit::prelude::{MODEL_COUNT, MODELS, kit_textures, model};
use nightshade_load::{LoadHandle, load_model_with_files};
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name};

pub fn prepare_models(models: &mut Models) {
    models.handles = vec![None; MODEL_COUNT];
    models.kits = vec![None; MODEL_COUNT];
    models.failed = vec![false; MODEL_COUNT];
    models.requested = vec![false; MODEL_COUNT];
    models.settle = vec![0; MODEL_COUNT];
}

pub fn request_model(loading: &mut Loading, index: usize) -> LoadHandle {
    let file = &MODELS[index];
    load_model_with_files(
        loading,
        file.bytes.to_vec(),
        kit_textures(file.kit)
            .iter()
            .map(|(path, bytes)| ((*path).to_string(), bytes.to_vec())),
    )
}

pub fn place_model(
    commands: &mut Commands,
    name: &str,
    position: Vec3,
    yaw: f32,
    scale: f32,
) -> Entity {
    commands.spawn((
        Name(name.to_string()),
        LocalTransform {
            translation: position,
            rotation: nalgebra_glm::quat_angle_axis(yaw, &Vec3::y()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
        Look(model(name)),
        Fit(scale),
    ))
}

pub fn plant_model(
    commands: &mut Commands,
    name: &str,
    spot: Vec2,
    yaw: f32,
    scale: f32,
) -> Entity {
    place_model(commands, name, ground(spot), yaw, scale)
}
