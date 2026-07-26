//! How a capability joins prefab spawning without the prefab knowing it.
//!
//! The same arrangement the scene uses: the prefab record stays plain data, and
//! a capability appends a function pointer here when it is composed. A prefab
//! that names a rigid body still spawns nothing without the physics plugin,
//! which is what composing the plugin is supposed to decide.

use nightshade_ecs::Entity;

use crate::ecs::transform::components::LocalTransform;
use crate::ecs::world::World;

use super::components::PrefabComponents;

/// Spawns one capability's components for a prefab node.
pub type PrefabEntitySpawn = fn(&mut World, Entity, &PrefabComponents, LocalTransform);

/// The prefab's capability table, appended to at composition.
#[derive(Default)]
pub struct PrefabCapabilityHooks {
    pub entity_spawn: Vec<PrefabEntitySpawn>,
}

/// Runs every capability's prefab spawn for `entity`.
pub fn run_entity_spawn(
    world: &mut World,
    entity: Entity,
    components: &PrefabComponents,
    local_transform: LocalTransform,
) {
    let hooks = std::mem::take(&mut world.res_mut::<PrefabCapabilityHooks>().entity_spawn);
    for hook in &hooks {
        hook(world, entity, components, local_transform);
    }
    world.res_mut::<PrefabCapabilityHooks>().entity_spawn = hooks;
}
