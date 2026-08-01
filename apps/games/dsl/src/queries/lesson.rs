use crate::data::lesson::Want;
use crate::data::{Program, Statement};
use crate::queries::write;

pub fn met(program: &Program, want: &Want) -> bool {
    match want {
        Want::Group(block, path) => fields(program, block)
            .iter()
            .any(|name| name.starts_with(&format!("{path}."))),
        Want::Values(block, path, count) => {
            let lead = format!("{path}.");
            fields(program, block)
                .iter()
                .filter(|name| name.starts_with(&lead))
                .count()
                >= *count
        }
        Want::Component(name) => program.components.iter().any(|held| held == name),
        Want::System(name) => program.systems.iter().any(|held| &held.name == name),
        Want::Port(name, wanted) => program
            .systems
            .iter()
            .filter(|held| &held.name == name)
            .any(|held| {
                write::sentence(held)
                    .iter()
                    .any(|line| line.contains(wanted))
            }),
        Want::Spawns(name, count) => program
            .systems
            .iter()
            .filter(|held| &held.name == name)
            .any(|held| spawns(&held.body) >= *count),
        Want::Statements(name, count) => program
            .systems
            .iter()
            .filter(|held| &held.name == name)
            .any(|held| held.body.len() >= *count),
        Want::Resource(name) => program.resources.iter().any(|(held, _)| held == name),
    }
}

fn fields(program: &Program, block: &str) -> Vec<String> {
    program
        .configs
        .iter()
        .chain(program.resources.iter())
        .filter(|(name, _)| name == block)
        .flat_map(|(_, fields)| fields.iter().map(|field| field.name.clone()))
        .collect()
}

fn spawns(body: &[Statement]) -> usize {
    let mut held = 0;
    for statement in body {
        match statement {
            Statement::Spawn { .. } => held += 1,
            Statement::When {
                then, otherwise, ..
            } => held += spawns(then) + spawns(otherwise),
            Statement::ForEach { body, .. } => held += spawns(body),
            _ => {}
        }
    }
    held
}
