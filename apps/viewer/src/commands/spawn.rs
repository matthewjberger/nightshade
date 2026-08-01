use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_scene::contract::LocalTransform;
use nightshade_scene::contract::primitives::Name;

use crate::commands::scene::register_spawned;
use crate::data::{LightKind, PrimitiveKind};
use crate::queries::camera::orbit_focus;

pub fn add_primitive_at_focus(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    commands: &mut Commands,
    active_camera: Option<Entity>,
    kind: PrimitiveKind,
) {
    let position = orbit_focus(active_camera, storage);
    let entity = add_primitive(commands, position, kind);
    register_spawned(stage, storage, entity);
}

pub fn add_light_at_focus(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    commands: &mut Commands,
    active_camera: Option<Entity>,
    kind: LightKind,
) {
    let position = orbit_focus(active_camera, storage);
    let entity = add_light(commands, stage.materials, position, kind);
    register_spawned(stage, storage, entity);
}

pub fn add_primitive(commands: &mut Commands, position: Vec3, kind: PrimitiveKind) -> Entity {
    match kind {
        PrimitiveKind::Cube => spawn_cube_at(commands, position),
        PrimitiveKind::Sphere => spawn_sphere_at(commands, position),
        PrimitiveKind::Cylinder => spawn_cylinder_at(commands, position),
        PrimitiveKind::Cone => spawn_cone_at(commands, position),
        PrimitiveKind::Torus => spawn_torus_at(commands, position),
        PrimitiveKind::Plane => spawn_plane_at(commands, position),
    }
}

pub fn add_light(
    commands: &mut Commands,
    registry: &mut MaterialRegistry,
    position: Vec3,
    kind: LightKind,
) -> Entity {
    let color = Vec3::new(1.0, 0.95, 0.8);
    let (name, light_type, intensity, range) = match kind {
        LightKind::Directional => ("Directional Light", LightType::Directional, 3.0, 0.0),
        LightKind::Point => ("Point Light", LightType::Point, 5.0, 10.0),
        LightKind::Spot => ("Spot Light", LightType::Spot, 10.0, 15.0),
    };

    let entity = spawn_sphere_at(commands, position + Vec3::new(0.0, 2.0, 0.0));
    commands.queue(move |storage, _| {
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.scale = Vec3::new(0.15, 0.15, 0.15);
        }
    });
    set_material(
        registry,
        commands,
        entity,
        format!("LightMarker_{}", entity.id),
        Material {
            base_color: [color.x, color.y, color.z, 1.0],
            emissive_factor: [color.x, color.y, color.z],
            emissive_strength: 4.0,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        },
    );
    commands.set(entity, Name(name.to_string()));
    commands.set(
        entity,
        Light {
            light_type,
            color,
            intensity,
            range,
            ..Default::default()
        },
    );
    entity
}
