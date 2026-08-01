use crate::theme;
use nalgebra_glm::{Vec3, vec3};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::components::{Light, LightType};
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn block(
    commands: &mut Commands,
    materials: &mut MaterialRegistry,
    shape: &str,
    position: Vec3,
    scale: Vec3,
    tint: [f32; 4],
) -> Entity {
    let entity = spawn_mesh_at(commands, shape, position, scale);
    set_material(
        materials,
        commands,
        entity,
        format!("{}_{}", theme::TITLE, entity.id),
        Material {
            base_color: tint,
            roughness: 0.9,
            metallic: 0.0,
            ..Default::default()
        },
    );
    entity
}

pub fn torch(commands: &mut Commands) -> Entity {
    commands.spawn((
        Name("Torch".to_string()),
        LocalTransform {
            translation: vec3(0.0, theme::TORCH_HEIGHT, 0.0),
            ..Default::default()
        },
        GlobalTransform::default(),
        Light {
            light_type: LightType::Spot,
            color: theme::TORCH_COLOR,
            intensity: 0.0,
            range: theme::RANGE,
            inner_cone_angle: theme::INNER,
            outer_cone_angle: theme::OUTER,
            cast_shadows: true,
            shadow_bias: 0.0004,
            ..Default::default()
        },
    ))
}

pub fn floor(commands: &mut Commands, materials: &mut MaterialRegistry) {
    block(
        commands,
        materials,
        "Plane",
        vec3(0.0, 0.0, 0.0),
        vec3(theme::REACH, 1.0, theme::REACH),
        theme::FLOOR_TINT,
    );
}
