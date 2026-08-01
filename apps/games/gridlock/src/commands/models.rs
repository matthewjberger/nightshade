use crate::components::{Cell, Fit, Look, Prop};
use crate::resources::Models;
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_gltf::Loading;
use nightshade_kit::prelude::{MODEL_COUNT, MODELS, kit_textures};
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

pub fn transform_at(position: Vec3, yaw: f32) -> LocalTransform {
    LocalTransform {
        translation: position,
        rotation: nalgebra_glm::quat_angle_axis(yaw, &Vec3::y()),
        scale: Vec3::new(1.0, 1.0, 1.0),
    }
}

pub fn place_piece(
    commands: &mut Commands,
    model: usize,
    position: Vec3,
    yaw: f32,
    scale: f32,
) -> Entity {
    commands.spawn((
        Name(MODELS[model].name.to_string()),
        transform_at(position, yaw),
        GlobalTransform::default(),
        Look(model),
        Fit(scale),
    ))
}

pub fn place_model(
    storage: &mut Storage,
    model: usize,
    position: Vec3,
    yaw: f32,
    scale: f32,
    prop: Option<usize>,
    cell: [i32; 2],
) -> Entity {
    match prop {
        Some(prop) => storage.spawn((
            Name(MODELS[model].name.to_string()),
            transform_at(position, yaw),
            GlobalTransform::default(),
            Look(model),
            Fit(scale),
            Cell(cell),
            Prop(prop),
        )),
        None => storage.spawn((
            Name(MODELS[model].name.to_string()),
            transform_at(position, yaw),
            GlobalTransform::default(),
            Look(model),
            Fit(scale),
            Cell(cell),
        )),
    }
}
