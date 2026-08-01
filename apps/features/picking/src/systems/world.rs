use crate::components::{DebugVolume, DemoMesh};
use crate::resources::PickingResources;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn spin_meshes(
    picking: Res<PickingResources>,
    time: Res<Time>,
    storage: &mut nightshade::ecs::Storage,
) {
    let delta = time.delta_time.max(0.0);
    if picking.spin_speed.abs() < f32::EPSILON || delta <= 0.0 {
        return;
    }
    let rotation =
        nalgebra_glm::quat_angle_axis(picking.spin_speed * delta, &Vec3::new(0.0, 1.0, 0.0));
    for entity in storage.query_entities::<(DemoMesh,)>().collect::<Vec<_>>() {
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.rotation = rotation * transform.rotation;
        }
    }

    let followers: Vec<(Entity, Entity)> = storage
        .query_ref::<(&DebugVolume,)>()
        .iter()
        .map(|(entity, (volume,))| (entity, volume.mesh))
        .collect();
    for (volume_entity, mesh_entity) in followers {
        let Some(translation) = storage
            .get::<GlobalTransform>(mesh_entity)
            .map(|global| global.translation())
        else {
            continue;
        };
        if let Some(transform) = storage.get_mut::<LocalTransform>(volume_entity) {
            transform.translation = translation;
        }
    }
}

pub fn sync_debug_volumes(
    picking: Res<PickingResources>,
    volumes: Query<(&mut Visibility,), With<DebugVolume>>,
) {
    volumes.for_each(|_entity, (visibility,)| {
        visibility.visible = picking.show_debug_volumes;
    });
}
