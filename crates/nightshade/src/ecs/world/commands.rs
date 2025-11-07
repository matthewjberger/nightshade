use crate::ecs::world::{Entity, World};

pub type CommandQueue = Vec<WorldCommand>;

#[derive(Debug, Clone)]
pub enum WorldCommand {
    DespawnRecursive { entity: Entity },
}

impl World {
    pub fn queue_command(&mut self, command: WorldCommand) {
        self.resources.command_queue.push(command);
    }
}

pub fn process_commands_system(world: &mut World) {
    let commands = world.resources.command_queue.drain(..).collect::<Vec<_>>();
    for command in commands {
        process_command(world, command);
    }
}

fn process_command(world: &mut World, command: WorldCommand) {
    match command {
        WorldCommand::DespawnRecursive { entity } => {
            let descendents = query_descendents(world, entity);
            despawn_entities_with_cache_cleanup(world, &descendents);
        }
    }
}

fn query_children(world: &World, entity: Entity) -> Vec<Entity> {
    world
        .query_entities(crate::ecs::world::PARENT)
        .filter(|child_entity| {
            world
                .get_parent(*child_entity)
                .is_some_and(|parent| parent.0 == Some(entity))
        })
        .collect()
}

fn query_descendents(world: &World, target_entity: Entity) -> Vec<Entity> {
    let mut descendents = Vec::new();
    let mut stack = vec![target_entity];
    while let Some(entity) = stack.pop() {
        descendents.push(entity);
        for child in query_children(world, entity) {
            stack.push(child);
        }
    }
    descendents
}

fn despawn_entities_with_cache_cleanup(world: &mut World, entities: &[Entity]) {
    for &entity in entities {
        world.resources.children_cache.remove(&entity);

        if let Some(parent_component) = world.get_parent(entity)
            && let Some(parent_entity) = parent_component.0
            && let Some(children) = world.resources.children_cache.get_mut(&parent_entity)
        {
            children.retain(|&e| e != entity);
        }
    }

    world.despawn_entities(entities);
}
