use nightshade::ecs::Entity;
use nightshade::prelude::Commands;

pub fn format_entity(entity: Entity) -> String {
    format!("entity_{}_{}", entity.id, entity.generation)
}

pub fn parse_entity(text: &str) -> Option<Entity> {
    let parts: Vec<&str> = text.split('_').collect();
    if parts.len() != 3 || parts[0] != "entity" {
        return None;
    }
    let id: u32 = parts[1].parse().ok()?;
    let generation: u32 = parts[2].parse().ok()?;
    Some(Entity { id, generation })
}

pub type CommandFn<C> = fn(&[&str], &mut Commands, &mut C) -> String;

pub struct Command<C> {
    pub name: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
    pub execute: CommandFn<C>,
}

impl<C> Copy for Command<C> {}

impl<C> Clone for Command<C> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct OutputLine {
    pub text: String,
    pub is_command: bool,
}
