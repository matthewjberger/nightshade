use crate::data::facility;
use crate::data::{self, Expression, Field, Port, Program, Statement, System};

#[derive(Clone, Debug)]
pub struct Net {
    pub name: String,
    pub requires: Vec<String>,
    pub excludes: Vec<String>,
}

pub fn runs_once(system: &System) -> bool {
    system.ports.iter().any(|port| matches!(port, Port::Once))
}

pub fn reads(system: &System) -> Vec<Net> {
    gather(system, false)
}

pub fn writes(system: &System) -> Vec<Net> {
    gather(system, true)
}

fn gather(system: &System, written: bool) -> Vec<Net> {
    let mut found = Vec::new();
    for port in &system.ports {
        match port {
            Port::Reads(list) if !written => {
                for name in list {
                    found.push(Net {
                        name: root(name),
                        requires: Vec::new(),
                        excludes: Vec::new(),
                    });
                }
            }
            Port::Writes(list) if written => {
                for name in list {
                    found.push(Net {
                        name: root(name),
                        requires: Vec::new(),
                        excludes: Vec::new(),
                    });
                }
            }
            Port::Select(selection) => {
                let mut requires: Vec<String> = selection
                    .columns
                    .iter()
                    .map(|column| column.name.clone())
                    .collect();
                requires.extend(selection.requires.iter().cloned());
                for column in &selection.columns {
                    if column.written == written {
                        found.push(Net {
                            name: column.name.clone(),
                            requires: requires.clone(),
                            excludes: selection.excludes.clone(),
                        });
                    }
                }
            }
            _ => {}
        }
    }
    found
}

fn contracts(system: &System, served: bool) -> Vec<String> {
    let mut found = Vec::new();
    for port in &system.ports {
        match port {
            Port::Needs(list) if !served => found.extend(list.iter().cloned()),
            Port::Provides(list) if served => found.extend(list.iter().cloned()),
            _ => {}
        }
    }
    found
}

fn root(name: &str) -> String {
    name.split('.').next().unwrap_or(name).to_string()
}

fn apart(first: &Net, second: &Net) -> bool {
    first
        .requires
        .iter()
        .any(|name| second.excludes.contains(name))
        || second
            .requires
            .iter()
            .any(|name| first.excludes.contains(name))
}

fn feeds(produced: &[Net], consumed: &[Net]) -> bool {
    produced.iter().any(|source| {
        consumed
            .iter()
            .any(|target| source.name == target.name && !apart(source, target))
    })
}

pub fn order(program: &Program) -> Vec<usize> {
    let count = program.systems.len();
    let written: Vec<Vec<Net>> = program.systems.iter().map(writes).collect();
    let taken: Vec<Vec<Net>> = program.systems.iter().map(reads).collect();

    let mut after: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut waiting = vec![0usize; count];
    let mut edges: Vec<(usize, usize)> = Vec::new();

    for (earlier, produced) in written.iter().enumerate() {
        for (later, consumed) in taken.iter().enumerate() {
            if earlier != later && feeds(produced, consumed) {
                edges.push((earlier, later));
            }
        }
    }

    for (later, system) in program.systems.iter().enumerate() {
        let wanted = contracts(system, false);
        if wanted.is_empty() {
            continue;
        }
        for (earlier, other) in program.systems.iter().enumerate() {
            if earlier == later {
                continue;
            }
            let served = contracts(other, true);
            if wanted.iter().any(|name| served.contains(name)) {
                edges.push((earlier, later));
            }
        }
    }

    for (earlier_name, later_name) in &program.orders {
        let earlier = program
            .systems
            .iter()
            .position(|one| &one.name == earlier_name);
        let later = program
            .systems
            .iter()
            .position(|one| &one.name == later_name);
        if let (Some(earlier), Some(later)) = (earlier, later)
            && earlier != later
        {
            edges.push((earlier, later));
        }
    }

    for (earlier, later) in edges {
        if !after[earlier].contains(&later) {
            after[earlier].push(later);
            waiting[later] += 1;
        }
    }

    let mut ordered = Vec::with_capacity(count);
    let mut placed = vec![false; count];

    while ordered.len() < count {
        let ready = |index: &usize| !placed[*index] && waiting[*index] == 0;
        let next = (0..count)
            .find(|index| ready(index) && runs_once(&program.systems[*index]))
            .or_else(|| (0..count).find(ready));
        let index = match next {
            Some(index) => index,
            None => (0..count).find(|index| !placed[*index]).unwrap_or(0),
        };
        placed[index] = true;
        ordered.push(index);
        let followers = after[index].clone();
        for follower in followers {
            waiting[follower] = waiting[follower].saturating_sub(1);
        }
    }

    ordered
}

pub fn check(program: &Program) -> Result<(), String> {
    for (block, fields) in &program.configs {
        tidy(block, fields)?;
    }
    for (module, name) in &program.shown {
        if !program
            .owner
            .iter()
            .any(|(held, owner)| held == name && owner == module)
        {
            return Err(format!("{module} shows {name}, and declares no such name"));
        }
    }
    let mut known: Vec<String> = program.components.clone();
    known.extend(facility::columns().map(String::from));
    known.extend(program.resources.iter().map(|(name, _)| name.clone()));
    known.extend(program.configs.iter().map(|(name, _)| name.clone()));
    known.extend(program.events.iter().cloned());
    known.extend(facility::blocks().map(|block| block.name.to_string()));
    known.extend(
        program
            .actions
            .iter()
            .map(|(name, _, _)| format!("{}.{name}", data::INPUT_RESOURCE)),
    );
    known.extend(program.assets.iter().map(|(name, _, _, _)| name.clone()));

    for (block, source, _, rows) in &program.assets {
        if !facility::holds_source(source) {
            return Err(format!(
                "{block} draws from {source}, and nothing serves that source"
            ));
        }
        for row in rows {
            if facility::binding_for(&row.kind).is_none() {
                return Err(format!(
                    "{block}.{} is a {}, and nothing serves that kind",
                    row.label, row.kind
                ));
            }
        }
    }

    let mut callable: Vec<String> = program
        .queries
        .iter()
        .map(|(name, _, _)| name.clone())
        .collect();
    callable.extend(program.components.iter().cloned());
    callable.extend(facility::columns().map(String::from));
    callable.extend(facility::functions().map(String::from));

    for system in &program.systems {
        for port in &system.ports {
            for name in named(port) {
                serves(&known, &name, &system.name)?;
                reaches(program, system, &root(&name))?;
            }
        }
        let mut locals: Vec<String> = Vec::new();
        for statement in &system.body {
            watch(program, &known, &callable, &mut locals, statement, system)?;
        }
    }
    Ok(())
}

fn agrees(
    program: &Program,
    system: &System,
    binding: &str,
    value: &Expression,
) -> Result<(), String> {
    let Expression::Field { of, name: label } = value else {
        return Ok(());
    };
    let Expression::Name(block) = of.as_ref() else {
        return Ok(());
    };
    let Some((_, _, _, rows)) = program.assets.iter().find(|(held, _, _, _)| held == block) else {
        return Ok(());
    };
    let Some(row) = rows.iter().find(|row| &row.label == label) else {
        return Err(format!(
            "{} names {block}.{label}, and {block} holds no such asset",
            system.name
        ));
    };
    if facility::binding_for(&row.kind) != Some(binding) {
        return Err(format!(
            "{} spawns {block}.{label} as {binding}, and it is a {}",
            system.name, row.kind
        ));
    }
    Ok(())
}

fn reaches(program: &Program, system: &System, name: &str) -> Result<(), String> {
    let Some((_, owner)) = program.owner.iter().find(|(held, _)| held == name) else {
        return Ok(());
    };
    if owner == &system.module {
        return Ok(());
    }
    if !program
        .visible
        .iter()
        .any(|(from, to)| from == &system.module && to == owner)
    {
        return Err(format!(
            "{} names {name}, which belongs to {owner}, and {} does not use it",
            system.name, system.module
        ));
    }
    if !program
        .shown
        .iter()
        .any(|(held, one)| held == owner && one == name)
    {
        return Err(format!(
            "{} names {name}, and {owner} does not show it",
            system.name
        ));
    }
    Ok(())
}

fn named(port: &Port) -> Vec<String> {
    match port {
        Port::Reads(list) | Port::Writes(list) => list.clone(),
        Port::Select(selection) => {
            let mut found: Vec<String> = selection
                .columns
                .iter()
                .map(|column| column.name.clone())
                .collect();
            found.extend(selection.requires.iter().cloned());
            found.extend(selection.excludes.iter().cloned());
            found
        }
        _ => Vec::new(),
    }
}

fn serves(known: &[String], name: &str, system: &str) -> Result<(), String> {
    if known.iter().any(|one| one == name) {
        return Ok(());
    }
    let stem = root(name);
    if !known.iter().any(|one| one == &stem) {
        return Err(format!(
            "{system} names {stem}, and nothing declares it and nothing serves it"
        ));
    }
    if let Some((_, field)) = name.split_once('.')
        && let Some(block) = facility::block(&stem)
        && !block.fields.contains(&field)
    {
        return Err(format!(
            "{system} names {stem}.{field}, and {stem} has no such field"
        ));
    }
    Ok(())
}

fn watch(
    program: &Program,
    known: &[String],
    callable: &[String],
    locals: &mut Vec<String>,
    statement: &Statement,
    system: &System,
) -> Result<(), String> {
    match statement {
        Statement::Spawn { bindings } => {
            for (name, value) in bindings {
                let held = program.components.iter().any(|one| one == name)
                    || facility::holds_column(name)
                    || facility::holds_binding(name);
                if !held {
                    return Err(format!(
                        "{} spawns with {name}, and nothing declares it and nothing serves it",
                        system.name
                    ));
                }
                reaches(program, system, name)?;
                agrees(program, system, name, value)?;
                reachable(callable, value, &system.name)?;
            }
        }
        Statement::Increase { place, value } => {
            reaches(program, system, &place.name)?;
            reachable(callable, value, &system.name)?;
        }
        Statement::Let { name, value } => {
            reachable(callable, value, &system.name)?;
            locals.push(name.clone());
        }
        Statement::Keep { name } | Statement::Recall { name } => serves(known, name, &system.name)?,
        Statement::Emit { value, .. } | Statement::Send { value, .. } => {
            reachable(callable, value, &system.name)?
        }
        Statement::Add { place, value } | Statement::Assign { place, value } => {
            let local = place.subject.is_none()
                && place.field.is_none()
                && locals.iter().any(|one| one == &place.name);
            if !local {
                serves(known, &place.name, &system.name)?;
                reaches(program, system, &root(&place.name))?;
            }
            reachable(callable, value, &system.name)?
        }
        Statement::Remove { subject } => reachable(callable, subject, &system.name)?,
        Statement::When {
            condition,
            then,
            otherwise,
        } => {
            reachable(callable, condition, &system.name)?;
            for inner in then.iter().chain(otherwise.iter()) {
                watch(program, known, callable, locals, inner, system)?;
            }
        }
        Statement::ForEach { name, over, body } => {
            reachable(callable, over, &system.name)?;
            locals.push(name.clone());
            for inner in body {
                watch(program, known, callable, locals, inner, system)?;
            }
        }
        Statement::Break | Statement::Stop => {}
    }
    Ok(())
}

fn reachable(callable: &[String], value: &Expression, system: &str) -> Result<(), String> {
    match value {
        Expression::Call { name, arguments } => {
            if !callable.iter().any(|one| one == name) {
                return Err(format!(
                    "{system} calls {name}, and nothing holds that name"
                ));
            }
            for argument in arguments {
                reachable(callable, argument, system)?;
            }
        }
        Expression::Field { of, .. }
        | Expression::Unary { value: of, .. }
        | Expression::IsSome(of) => reachable(callable, of, system)?,
        Expression::Index { of, at } => {
            reachable(callable, of, system)?;
            reachable(callable, at, system)?;
        }
        Expression::Binary { left, right, .. } => {
            reachable(callable, left, system)?;
            reachable(callable, right, system)?;
        }
        Expression::Tuple(parts) | Expression::List(parts) => {
            for part in parts {
                reachable(callable, part, system)?;
            }
        }
        Expression::Record(fields) => {
            for (_, part) in fields {
                reachable(callable, part, system)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn tidy(block: &str, fields: &[Field]) -> Result<(), String> {
    let mut levels: Vec<(String, Vec<String>)> = Vec::new();
    for field in fields {
        let parts: Vec<&str> = field.name.split('.').collect();
        for at in 0..parts.len() {
            let path = parts[..at].join(".");
            if !levels.iter().any(|(held, _)| held == &path) {
                levels.push((path.clone(), Vec::new()));
            }
            let held = levels
                .iter_mut()
                .find(|(one, _)| one == &path)
                .map(|(_, held)| held)
                .expect("the level is there");
            let next = parts[at].to_string();
            if !held.contains(&next) {
                held.push(next);
            }
        }
    }
    for (path, held) in levels {
        if held.len() > data::GROUP_LIMIT {
            let place = if path.is_empty() {
                block.to_string()
            } else {
                format!("{block}.{path}")
            };
            return Err(format!(
                "{place} holds {} values together, and the most one group holds is {}, so put them in named groups",
                held.len(),
                data::GROUP_LIMIT
            ));
        }
    }
    Ok(())
}

pub fn owned(one: &Program) -> Vec<String> {
    let mut held: Vec<String> = Vec::new();
    held.extend(one.components.iter().cloned());
    held.extend(one.events.iter().cloned());
    held.extend(one.resources.iter().map(|(name, _)| name.clone()));
    held.extend(one.configs.iter().map(|(name, _)| name.clone()));
    held.extend(one.queries.iter().map(|(name, _, _)| name.clone()));
    held.extend(one.assets.iter().map(|(name, _, _, _)| name.clone()));
    held
}
